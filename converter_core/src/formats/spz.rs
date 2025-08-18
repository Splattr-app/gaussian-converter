use crate::{ConversionError, Exporter, GaussianSplat, Importer, Scene};
use spz_rs::{UnpackedGaussian, load_packed_gaussians_from_spz_buffer};

pub struct SpzImporter;
pub struct SpzV2Exporter;

fn unpack_sh_rest(unpacked: &UnpackedGaussian) -> Vec<f32> {
  let mut sh_rest = vec![0.0f32; 45]; // preallocate all 45 slots

  for i in 0..15 {
    sh_rest[i] = unpacked.sh_r[i] as f32;
    sh_rest[i + 15] = unpacked.sh_g[i] as f32;
    sh_rest[i + 30] = unpacked.sh_b[i] as f32;
  }

  sh_rest
}

impl Importer for SpzImporter {
  fn import(reader: &mut impl std::io::Read) -> Result<Scene, ConversionError> {
    let packed_gaussians = load_packed_gaussians_from_spz_buffer(reader)?;

    let mut splats: Vec<GaussianSplat> = Vec::with_capacity(packed_gaussians.num_points);

    for i in 0..packed_gaussians.num_points {
      let unpacked_gaussian = packed_gaussians.unpack(i);

      let splat = GaussianSplat {
        position: [
          unpacked_gaussian.position[0],
          unpacked_gaussian.position[1],
          unpacked_gaussian.position[2],
        ],
        normal: [0f32, 0f32, 0f32],
        spherical_harmonics_dc: unpacked_gaussian.color,
        spherical_harmonics_rest: unpack_sh_rest(&unpacked_gaussian),
        opacity: unpacked_gaussian.alpha,
        scale: unpacked_gaussian.scale,
        rotation: unpacked_gaussian.rotation,
      };

      splats.push(splat);
    }

    Ok(Scene { splats })
  }
}

impl Exporter for SpzV2Exporter {
  fn export(scene: &Scene, writer: &mut impl std::io::Write) -> Result<(), ConversionError> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    let map_io_err = |e: std::io::Error| ConversionError::WriteError {
      format: "SPZ".to_string(),
      message: e.to_string(),
    };

    // Header fields
    let sh_degree: u8 = 3; // 15 coeffs/channel
    let version: u32 = 2; // fixed-point 3-byte coordinates
    let flags: u8 = 0;

    let num_points = scene.splats.len();
    let num_points_u32 = num_points as u32;

    // Choose fractional_bits adaptively to maximize precision without overflow
    // Store signed 24-bit fixed-point (range [-8388608, 8388607]).
    // Need: max(|coord|) * 2^fractional_bits <= 8388607
    let mut max_abs = 0f32;
    for s in &scene.splats {
      // magnitude doesn't change with sign flips, so use raw positions
      for &c in &s.position {
        let a = c.abs();
        if a > max_abs {
          max_abs = a;
        }
      }
    }

    // If everything is at the origin, just pick a high fractional precision.
    let max_val = if max_abs.is_finite() { max_abs } else { 0.0 };
    let max_fixed_mag: f32 = 8_388_607.0; // 2^23 - 1

    // fb_max_safe = floor(log2(max_fixed_mag / max_val))
    // Clamp to a reasonable range [4..20] to avoid dumb extremes.
    let fb_from_range = if max_val <= 1e-12 {
      20i32
    } else {
      ((max_fixed_mag / max_val).log2().floor() as i32).max(0)
    };
    let fractional_bits_i32 = fb_from_range.clamp(4, 20);
    let fractional_bits: u8 = fractional_bits_i32 as u8;

    let scale_pos = (1u32 << fractional_bits) as f32; // multiplier to convert position -> fixed24

    // helpers
    #[inline]
    fn clamp_u8i(v: i32) -> u8 {
      if v < 0 {
        0
      } else if v > 255 {
        255
      } else {
        v as u8
      }
    }
    #[inline]
    fn clamp_u8f(v: f32) -> u8 {
      if !v.is_finite() {
        0
      } else {
        let r = v.round() as i32;
        clamp_u8i(r)
      }
    }

    // Preallocate component buffers (contiguous, cache-friendly)
    let mut positions: Vec<u8> = Vec::with_capacity(num_points * 9); // 3 coords * 3 bytes
    let mut alphas: Vec<u8> = Vec::with_capacity(num_points); // 1 byte
    let mut colors: Vec<u8> = Vec::with_capacity(num_points * 3); // 3 bytes
    let mut scales: Vec<u8> = Vec::with_capacity(num_points * 3); // 3 bytes
    let mut rotations: Vec<u8> = Vec::with_capacity(num_points * 3); // 3 bytes (x,y,z only)
    let mut sh: Vec<u8> = Vec::with_capacity(num_points * 45); // 15*3 bytes

    const COLOR_SCALE: f32 = 0.15;

    for splat in &scene.splats {
      // Positions
      let coords = [splat.position[0], splat.position[1], splat.position[2]];
      for &coord in &coords {
        // fixed24 = round(coord * scale_pos), then store little-endian 3 bytes
        let fixed = (coord * scale_pos).round() as i32;
        positions.push((fixed & 0xff) as u8);
        positions.push(((fixed >> 8) & 0xff) as u8);
        positions.push(((fixed >> 16) & 0xff) as u8);
      }

      // Alpha (linear 0..1 -> 0..255), avoids sigmoid squashing detail
      let a = if splat.opacity.is_finite() {
        splat.opacity.clamp(0.0, 1.0)
      } else {
        0.0
      };
      alphas.push((a * 255.0).round().clamp(0.0, 255.0) as u8);

      // Color DC
      for &c in &splat.spherical_harmonics_dc {
        let packed = ((c * COLOR_SCALE) + 0.5) * 255.0;
        colors.push(clamp_u8f(packed));
      }

      // Scales
      for &s in &splat.scale {
        let q = ((s + 10.0) * 16.0).round() as i32;
        scales.push(clamp_u8i(q));
      }

      // Rotation: store x,y,z as bytes
      let x = splat.rotation[1];
      let y = splat.rotation[2];
      let z = splat.rotation[3];
      rotations.push(clamp_u8f((x + 1.0) * 127.5));
      rotations.push(clamp_u8f((y + 1.0) * 127.5));
      rotations.push(clamp_u8f((z + 1.0) * 127.5));

      // SH rest (15 per channel; interleaved r,g,b per coefficient)
      // inverse of: unquantize_sh(x) = (x - 128) / 128
      for i in 0..15 {
        let r = *splat.spherical_harmonics_rest.get(i).unwrap_or(&0.0);
        let g = *splat.spherical_harmonics_rest.get(i + 15).unwrap_or(&0.0);
        let b = *splat.spherical_harmonics_rest.get(i + 30).unwrap_or(&0.0);
        sh.push(clamp_u8f(r * 128.0 + 128.0));
        sh.push(clamp_u8f(g * 128.0 + 128.0));
        sh.push(clamp_u8f(b * 128.0 + 128.0));
      }
    }

    // Build header (same as before, but with our adaptive fractional_bits)
    let magic: u32 = 0x5053474e;
    let mut header_bytes: [u8; 16] = [0; 16];
    header_bytes[0..4].copy_from_slice(&magic.to_le_bytes());
    header_bytes[4..8].copy_from_slice(&version.to_le_bytes());
    header_bytes[8..12].copy_from_slice(&num_points_u32.to_le_bytes());
    header_bytes[12] = sh_degree;
    header_bytes[13] = fractional_bits;
    header_bytes[14] = flags;
    header_bytes[15] = 0; // reserved

    // Gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());

    encoder.write_all(&header_bytes).map_err(map_io_err)?;
    encoder.write_all(&positions).map_err(map_io_err)?;
    encoder.write_all(&alphas).map_err(map_io_err)?;
    encoder.write_all(&colors).map_err(map_io_err)?;
    encoder.write_all(&scales).map_err(map_io_err)?;
    encoder.write_all(&rotations).map_err(map_io_err)?;
    encoder.write_all(&sh).map_err(map_io_err)?;

    let output = encoder.finish().map_err(map_io_err)?;
    writer.write_all(&output).map_err(map_io_err)?;
    Ok(())
  }
}
