use crate::{ConversionError, Exporter, GaussianSplat, Importer, Scene};
use std::convert::TryInto;

pub struct SplatImporter;
pub struct SplatExporter;

const SPLAT_SIZE_BYTES: usize = 32;
const SH_C0: f32 = 0.28209479177387814;

/// Helper function to read a little-endian f32 from a slice.
fn read_f32_le(slice: &[u8]) -> f32 {
  f32::from_le_bytes(slice.try_into().unwrap())
}

// --- IMPORTER ---

impl Importer for SplatImporter {
  fn import(reader: &mut impl std::io::Read) -> Result<Scene, ConversionError> {
    let mut buffer = Vec::new();
    reader
      .read_to_end(&mut buffer)
      .map_err(|e| ConversionError::ParseError {
        format: "SPLAT".to_string(),
        message: e.to_string(),
      })?;

    if buffer.len() % SPLAT_SIZE_BYTES != 0 {
      return Err(ConversionError::ParseError {
        format: "SPLAT".to_string(),
        message: format!(
          "Invalid file size. Expected multiple of {}.",
          SPLAT_SIZE_BYTES
        ),
      });
    }

    let num_splats = buffer.len() / SPLAT_SIZE_BYTES;
    let mut splats = Vec::with_capacity(num_splats);

    for chunk in buffer.chunks_exact(SPLAT_SIZE_BYTES) {
      // Position (Bytes 0-11)
      let position = [
        read_f32_le(&chunk[0..4]),
        read_f32_le(&chunk[4..8]),
        read_f32_le(&chunk[8..12]),
      ];

      // Scale (Bytes 12-23) - Convert from linear to log scale
      let scale = [
        read_f32_le(&chunk[12..16]).ln(),
        read_f32_le(&chunk[16..20]).ln(),
        read_f32_le(&chunk[20..24]).ln(),
      ];

      // Color (Bytes 24-27) - RGBA u8 to SH f32
      let color_rgba = &chunk[24..28];
      let spherical_harmonics_dc = [
        (color_rgba[0] as f32 / 255.0 - 0.5) / SH_C0,
        (color_rgba[1] as f32 / 255.0 - 0.5) / SH_C0,
        (color_rgba[2] as f32 / 255.0 - 0.5) / SH_C0,
      ];

      // Opacity (Byte 27) - u8 to inverse sigmoid
      let normalized_opacity = (color_rgba[3] as f32 / 255.0).clamp(1e-6, 1.0 - 1e-6);
      let opacity = (normalized_opacity / (1.0 - normalized_opacity)).ln();

      // Rotation (Bytes 28-31) - u8 to normalized quaternion f32
      let rot_u8 = &chunk[28..32];
      let mut rot_f32 = [
        (rot_u8[0] as f32 / 255.0) * 2.0 - 1.0, // x
        (rot_u8[1] as f32 / 255.0) * 2.0 - 1.0, // y
        (rot_u8[2] as f32 / 255.0) * 2.0 - 1.0, // z
        (rot_u8[3] as f32 / 255.0) * 2.0 - 1.0, // w
      ];

      let len =
        (rot_f32[0].powi(2) + rot_f32[1].powi(2) + rot_f32[2].powi(2) + rot_f32[3].powi(2)).sqrt();
      if len > 0.0 {
        for v in &mut rot_f32 {
          *v /= len;
        }
      }

      // Re-order from [x, y, z, w] to our struct's [w, x, y, z]
      let rotation = [rot_f32[3], rot_f32[0], rot_f32[1], rot_f32[2]];

      splats.push(GaussianSplat {
        position,
        scale,
        rotation,
        spherical_harmonics_dc,
        opacity,
        normal: [0.0, 0.0, 0.0],
        spherical_harmonics_rest: vec![0.0; 45],
      });
    }

    Ok(Scene { splats })
  }
}

// --- EXPORTER ---

impl Exporter for SplatExporter {
  fn export(scene: &Scene, writer: &mut impl std::io::Write) -> Result<(), ConversionError> {
    let map_io_err = |e: std::io::Error| ConversionError::WriteError {
      format: "SPLAT".to_string(),
      message: e.to_string(),
    };

    for splat in &scene.splats {
      // Position
      for &p in &splat.position {
        writer.write_all(&p.to_le_bytes()).map_err(map_io_err)?;
      }

      // Scale - Convert from log scale to linear
      for &s in &splat.scale {
        writer
          .write_all(&s.exp().to_le_bytes())
          .map_err(map_io_err)?;
      }

      // Color - Convert from SH f32 to RGBA u8
      let r = ((splat.spherical_harmonics_dc[0] * SH_C0 + 0.5) * 255.0)
        .round()
        .clamp(0.0, 255.0) as u8;
      let g = ((splat.spherical_harmonics_dc[1] * SH_C0 + 0.5) * 255.0)
        .round()
        .clamp(0.0, 255.0) as u8;
      let b = ((splat.spherical_harmonics_dc[2] * SH_C0 + 0.5) * 255.0)
        .round()
        .clamp(0.0, 255.0) as u8;

      // Opacity - Apply sigmoid and convert to u8
      let sigmoid = 1.0 / (1.0 + (-splat.opacity).exp());
      let a = (sigmoid * 255.0).round().clamp(0.0, 255.0) as u8;
      writer.write_all(&[r, g, b, a]).map_err(map_io_err)?;

      // Rotation - Convert from [w, x, y, z] f32 to quantized [x, y, z, w] u8
      let mut rot = splat.rotation; // [w, x, y, z]
      let len = (rot[1].powi(2) + rot[2].powi(2) + rot[3].powi(2) + rot[0].powi(2)).sqrt();
      if len > 0.0 {
        for v in &mut rot {
          *v /= len;
        }
      }

      let rot_u8 = [
        (((rot[1] + 1.0) / 2.0) * 255.0).round().clamp(0.0, 255.0) as u8, // x
        (((rot[2] + 1.0) / 2.0) * 255.0).round().clamp(0.0, 255.0) as u8, // y
        (((rot[3] + 1.0) / 2.0) * 255.0).round().clamp(0.0, 255.0) as u8, // z
        (((rot[0] + 1.0) / 2.0) * 255.0).round().clamp(0.0, 255.0) as u8, // w
      ];
      writer.write_all(&rot_u8).map_err(map_io_err)?;
    }

    Ok(())
  }
}
