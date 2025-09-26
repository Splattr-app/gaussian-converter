use crate::{ConversionError, Exporter, GaussianSplat, Scene};

pub struct PlyCompressedExporter;

impl Exporter for PlyCompressedExporter {
  fn export(scene: &Scene, writer: &mut impl std::io::Write) -> Result<(), ConversionError> {
    const CHUNK_SIZE: usize = 256;

    let map_io_err = |e: std::io::Error| ConversionError::WriteError {
      format: "PLY (Compressed)".to_string(),
      message: e.to_string(),
    };

    let num_splats = scene.splats.len();
    if num_splats == 0 {
      return Ok(());
    }
    let num_chunks = (num_splats + CHUNK_SIZE - 1) / CHUNK_SIZE;

    // Detect whether we have SH data (same as before).
    let has_sh_data = if let Some(first_splat) = scene.splats.first() {
      !first_splat.spherical_harmonics_rest.is_empty()
        && first_splat
          .spherical_harmonics_rest
          .iter()
          .any(|&v| v.abs() > 1e-6)
    } else {
      false
    };

    // --- Stage 1: Write PLY header that advertises chunk/vertex/sh layout ---
    // The header follows the compressed PLY spec in compressed_ply_docs.md. We use a
    // binary_little_endian body and declare three elements: `chunk`, `vertex`, and
    // optional `sh` if higher-order SH data exist.
    writeln!(writer, "ply").map_err(map_io_err)?;
    writeln!(writer, "format binary_little_endian 1.0").map_err(map_io_err)?;
    writeln!(writer, "element chunk {}", num_chunks).map_err(map_io_err)?;
    writeln!(writer, "property float min_x").map_err(map_io_err)?;
    writeln!(writer, "property float min_y").map_err(map_io_err)?;
    writeln!(writer, "property float min_z").map_err(map_io_err)?;
    writeln!(writer, "property float max_x").map_err(map_io_err)?;
    writeln!(writer, "property float max_y").map_err(map_io_err)?;
    writeln!(writer, "property float max_z").map_err(map_io_err)?;
    writeln!(writer, "property float min_scale_x").map_err(map_io_err)?;
    writeln!(writer, "property float min_scale_y").map_err(map_io_err)?;
    writeln!(writer, "property float min_scale_z").map_err(map_io_err)?;
    writeln!(writer, "property float max_scale_x").map_err(map_io_err)?;
    writeln!(writer, "property float max_scale_y").map_err(map_io_err)?;
    writeln!(writer, "property float max_scale_z").map_err(map_io_err)?;
    writeln!(writer, "property float min_r").map_err(map_io_err)?;
    writeln!(writer, "property float min_g").map_err(map_io_err)?;
    writeln!(writer, "property float min_b").map_err(map_io_err)?;
    writeln!(writer, "property float max_r").map_err(map_io_err)?;
    writeln!(writer, "property float max_g").map_err(map_io_err)?;
    writeln!(writer, "property float max_b").map_err(map_io_err)?;

    writeln!(writer, "element vertex {}", num_splats).map_err(map_io_err)?;
    writeln!(writer, "property uint packed_position").map_err(map_io_err)?;
    writeln!(writer, "property uint packed_rotation").map_err(map_io_err)?;
    writeln!(writer, "property uint packed_scale").map_err(map_io_err)?;
    writeln!(writer, "property uint packed_color").map_err(map_io_err)?;

    if has_sh_data {
      writeln!(writer, "element sh {}", num_splats).map_err(map_io_err)?;
      for i in 0..45 {
        writeln!(writer, "property uchar f_rest_{}", i).map_err(map_io_err)?;
      }
    }
    writeln!(writer, "end_header").map_err(map_io_err)?;

    // --- Stage 2: Prepare contiguous binary buffers per spec ---
    // chunk_f32: num_chunks * 18 floats (bounds for position, scale, color)
    let mut chunk_f32 = vec![0f32; num_chunks * 18];
    // vertex_u32: num_splats * 4 u32 fields (packed_position, rotation, scale, color)
    let mut vertex_u32 = vec![0u32; num_splats * 4];
    // sh bytes: only if has_sh_data
    let mut sh_bytes = if has_sh_data {
      vec![0u8; num_splats * 45]
    } else {
      Vec::new()
    };

    // --- Stage 3: Iterate chunks, compute per-chunk bounds, and pack vertices ---
    // We compute min/max for position, linear-scale, and DC color per chunk. Then we
    // quantize and bit-pack vertex attributes according to the bit allocation in the
    // spec (position/scale: x=11 MSB, y=10, z=11 LSB; rotation: 2+10+10+10; color: R:G:B:A).
    for (i, chunk_splats_orig) in scene.splats.chunks(CHUNK_SIZE).enumerate() {
      // padded_chunk for computing bounds (same as before)
      let mut padded_chunk = chunk_splats_orig.to_vec();
      if padded_chunk.len() < CHUNK_SIZE && !padded_chunk.is_empty() {
        let last_splat = padded_chunk.last().unwrap().clone();
        padded_chunk.resize(CHUNK_SIZE, last_splat);
      }

      let (min_pos, max_pos) = Self::calculate_position_bounds(&padded_chunk);
      // calculate_scale_bounds should return LINEAR min/max (not log). Keep the
      // implementation you already switched to (exp of stored log values).
      let (min_scl_lin, max_scl_lin) = Self::calculate_scale_bounds(&padded_chunk);
      let (min_col, max_col) = Self::calculate_color_bounds(&padded_chunk);

      // --- Stage 3a: Store per-chunk bounds into `chunk_f32` ---
      let base = i * 18;
      chunk_f32[base + 0] = min_pos[0];
      chunk_f32[base + 1] = min_pos[1];
      chunk_f32[base + 2] = min_pos[2];
      chunk_f32[base + 3] = max_pos[0];
      chunk_f32[base + 4] = max_pos[1];
      chunk_f32[base + 5] = max_pos[2];
      // Clamp linear scale bounds to avoid degenerate intervals.
      chunk_f32[base + 6] = min_scl_lin[0].max(1e-8);
      chunk_f32[base + 7] = min_scl_lin[1].max(1e-8);
      chunk_f32[base + 8] = min_scl_lin[2].max(1e-8);
      chunk_f32[base + 9] = max_scl_lin[0].max(1e-8);
      chunk_f32[base + 10] = max_scl_lin[1].max(1e-8);
      chunk_f32[base + 11] = max_scl_lin[2].max(1e-8);
      chunk_f32[base + 12] = min_col[0];
      chunk_f32[base + 13] = min_col[1];
      chunk_f32[base + 14] = min_col[2];
      chunk_f32[base + 15] = max_col[0];
      chunk_f32[base + 16] = max_col[1];
      chunk_f32[base + 17] = max_col[2];

      // --- Stage 3b: Pack vertices for this chunk into `vertex_u32` ---
      let vertex_chunk_offset = i * CHUNK_SIZE * 4;
      for (j, splat) in chunk_splats_orig.iter().enumerate() {
        let write_idx = vertex_chunk_offset + j * 4;
        // Position: x=11 MSB, y=10, z=11 LSB.
        vertex_u32[write_idx + 0] =
          Self::quantize_pack_position(&splat.position, &min_pos, &max_pos);
        vertex_u32[write_idx + 1] = Self::quantize_pack_rotation(&splat.rotation);
        // Scale: inputs are log-space in the struct; exporter packs linear (exp) into x=11, y=10, z=11.
        vertex_u32[write_idx + 2] =
          Self::quantize_pack_scale(&splat.scale, &min_scl_lin, &max_scl_lin);
        // Color: pack as R:G:B:A (R is MSB, A is LSB) per spec.
        vertex_u32[write_idx + 3] = Self::pack_color_opacity(splat, &min_col, &max_col);

        if has_sh_data {
          let sh_off = (i * CHUNK_SIZE + j) * 45;
          let packed = Self::quantize_sh_rest(&splat.spherical_harmonics_rest);
          sh_bytes[sh_off..sh_off + 45].copy_from_slice(&packed);
        }
      }

      // We skip writing any padded entries beyond the real splat count. `vertex_u32` is sized
      // to exactly `num_splats * 4`, so the final chunk will not overflow.
    }

    // --- Stage 4: Write binary data in the same order as the header ---
    // First: `chunk_f32` as f32 little-endian values.
    for &f in &chunk_f32 {
      writer.write_all(&f.to_le_bytes()).map_err(map_io_err)?;
    }

    // Second: `vertex_u32` as u32 little-endian values.
    for &v in &vertex_u32 {
      writer.write_all(&v.to_le_bytes()).map_err(map_io_err)?;
    }

    // Finally: optional `sh` bytes if present.
    if has_sh_data {
      writer.write_all(&sh_bytes).map_err(map_io_err)?;
    }

    Ok(())
  }
}

// Helper implementation for PlyCompressedExporter
impl PlyCompressedExporter {
  // Bounding box for position remains the same.
  fn calculate_position_bounds(splats: &[GaussianSplat]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for s in splats {
      for i in 0..3 {
        min[i] = min[i].min(s.position[i]);
        max[i] = max[i].max(s.position[i]);
      }
    }
    (min, max)
  }

  fn calculate_scale_bounds(splats: &[GaussianSplat]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for s in splats {
      for i in 0..3 {
        let scale = s.scale[i].exp();
        min[i] = min[i].min(scale);
        max[i] = max[i].max(scale);
      }
    }
    (min, max)
  }

  fn calculate_color_bounds(splats: &[GaussianSplat]) -> ([f32; 3], [f32; 3]) {
    const SH_C0: f32 = 0.28209479177387814;
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for s in splats {
      for i in 0..3 {
        let val = s.spherical_harmonics_dc[i] * SH_C0 + 0.5;
        min[i] = min[i].min(val);
        max[i] = max[i].max(val);
      }
    }
    (min, max)
  }

  fn quantize_pack_position(pos: &[f32; 3], min: &[f32; 3], max: &[f32; 3]) -> u32 {
    let quantize = |val: f32, min_v: f32, max_v: f32, bits: u32| -> u32 {
      if (max_v - min_v).abs() < 1e-8 {
        return 0;
      }
      let norm = (val - min_v) / (max_v - min_v);
      let max_quant_val = ((1 << bits) - 1) as f32;
      (norm * max_quant_val).round().clamp(0.0, max_quant_val) as u32
    };
    let ix = quantize(pos[0], min[0], max[0], 11);
    let iy = quantize(pos[1], min[1], max[1], 10);
    let iz = quantize(pos[2], min[2], max[2], 11);
    // Pack with x in the most significant 11 bits, y in the middle 10 bits, z in the least 11 bits.
    (ix << 21) | (iy << 11) | iz
  }

  fn quantize_pack_scale(log_scale: &[f32; 3], min: &[f32; 3], max: &[f32; 3]) -> u32 {
    let quantize = |val: f32, min_v: f32, max_v: f32, bits: u32| -> u32 {
      if (max_v - min_v).abs() < 1e-8 {
        return 0;
      }
      let norm = (val - min_v) / (max_v - min_v);
      let max_quant_val = ((1 << bits) - 1) as f32;
      (norm * max_quant_val).round().clamp(0.0, max_quant_val) as u32
    };
    let isx = quantize(log_scale[0].exp(), min[0], max[0], 11);
    let isy = quantize(log_scale[1].exp(), min[1], max[1], 10);
    let isz = quantize(log_scale[2].exp(), min[2], max[2], 11);
    // Pack with x in the most significant 11 bits, y in the middle 10 bits, z in the least 11 bits.
    (isx << 21) | (isy << 11) | isz
  }

  fn quantize_pack_rotation(rot: &[f32; 4]) -> u32 {
    // Largest-component quaternion compression (2 + 10 + 10 + 10 bits):
    // 1) Normalize quaternion.
    // 2) Identify index of the component with largest absolute value (stored in top 2 bits).
    // 3) For the remaining three components, map from [-1/sqrt(2), +1/sqrt(2)] to [0, 1023]
    //    and store as 10-bit integers.
    let mut norm_rot = *rot;
    let len =
      (norm_rot[0].powi(2) + norm_rot[1].powi(2) + norm_rot[2].powi(2) + norm_rot[3].powi(2))
        .sqrt();
    if len > 1e-8 {
      for v in &mut norm_rot {
        *v /= len;
      }
    }

    let mut max_val = 0.0;
    let mut max_idx = 0;
    for (i, &val) in norm_rot.iter().enumerate() {
      if val.abs() > max_val {
        max_val = val.abs();
        max_idx = i;
      }
    }

    let mut components = [0u32; 3];
    let mut component_idx = 0;
    for (i, &val) in norm_rot.iter().enumerate() {
      if i != max_idx {
        const NORM_FACTOR: f32 = 0.70710678118; // 1 / sqrt(2)
        let normalized = (val / NORM_FACTOR + 1.0) / 2.0;
        let quantized = (normalized * 1023.0).round().clamp(0.0, 1023.0);
        components[component_idx] = quantized as u32;
        component_idx += 1;
      }
    }

    (max_idx as u32) << 30 | components[2] << 20 | components[1] << 10 | components[0]
  }

  fn pack_color_opacity(splat: &GaussianSplat, min: &[f32; 3], max: &[f32; 3]) -> u32 {
    const SH_C0: f32 = 0.28209479177387814;
    let quantize_color = |val: f32, min_v: f32, max_v: f32| -> u32 {
      if (max_v - min_v).abs() < 1e-8 {
        return 0;
      }
      let norm = (val - min_v) / (max_v - min_v);
      (norm * 255.0).round().clamp(0.0, 255.0) as u32
    };

    let r = quantize_color(
      splat.spherical_harmonics_dc[0] * SH_C0 + 0.5,
      min[0],
      max[0],
    );
    let g = quantize_color(
      splat.spherical_harmonics_dc[1] * SH_C0 + 0.5,
      min[1],
      max[1],
    );
    let b = quantize_color(
      splat.spherical_harmonics_dc[2] * SH_C0 + 0.5,
      min[2],
      max[2],
    );

    let sigmoid = 1.0 / (1.0 + (-splat.opacity).exp());
    let a = (sigmoid * 255.0).round().clamp(0.0, 255.0) as u32;

    // Pack as R (MSB) : G : B : A (LSB) per spec.
    (r << 24) | (g << 16) | (b << 8) | a
  }

  // SH (Rest) packing is correct.
  fn quantize_sh_rest(sh: &[f32]) -> [u8; 45] {
    // Map each coefficient from roughly [-4, +4] to [0, 255] using:
    // n = (val / 8) + 0.5; byte = floor(n * 256).
    let mut packed = [0u8; 45];
    for i in 0..45 {
      let val = sh.get(i).copied().unwrap_or(0.0);
      let nvalue = (val / 8.0) + 0.5;
      packed[i] = (nvalue * 256.0).floor().clamp(0.0, 255.0) as u8;
    }
    packed
  }
}
