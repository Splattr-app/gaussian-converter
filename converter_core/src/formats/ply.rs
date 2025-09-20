use std::io::BufReader;

use crate::{ConversionError, Exporter, GaussianSplat, Importer, Scene};
use ply_rs::parser::Parser;
use ply_rs::ply;

pub struct PlyImporter;
pub struct PlyASCIIExporter;
pub struct PlyBinaryExporter;
pub struct PlyCompressedExporter;

// const SH_C0: f32 = 0.28209479177387814;
fn get_f32_property(vert: &ply::DefaultElement, key: &str) -> f32 {
  vert.get(key).map_or(0.0, |prop| property_to_f32(prop))
}

fn property_to_f32(prop: &ply::Property) -> f32 {
  match prop {
    ply::Property::Float(f) => *f,
    _ => 0.0, // fallback for unsupported property types
  }
}

fn get_spherical_harmonics_res(vert: &ply::DefaultElement) -> Vec<f32> {
  let base_key = "f_rest_";
  let mut output: Vec<f32> = Vec::with_capacity(45); // Pre-allocate with capacity

  for i in 0..45 {
    let key = format!("{}{}", base_key, i);
    // Use the safe getter for each spherical harmonic coefficient
    output.push(get_f32_property(vert, &key));
  }

  output
}
// fn write_f32_slice_to_vec(vec: &mut Vec<u8>, data: &[f32]) {
//   for &val in data {
//     vec.extend_from_slice(&val.to_le_bytes());
//   }
// }

impl Importer for PlyImporter {
  fn import(reader: &mut impl std::io::Read) -> Result<Scene, ConversionError> {
    let parser = Parser::<ply::DefaultElement>::new();
    let mut reader = std::io::BufReader::new(reader);

    // Read header and check if it's compressed or uncompressed ply
    let header = parser
      .read_header(&mut reader)
      .map_err(|e| ConversionError::ParseError {
        format: "PLY".to_string(),
        message: format!("Failed to read header: {}", e),
      })?;

    // if includes "chunk" it's compressed
    if header.elements.contains_key("chunk") {
      return Self::import_compressed(&header, reader);
    } else {
      return Self::import_uncompressed(&header, reader);
    }
  }
}

impl PlyImporter {
  fn import_uncompressed(
    header: &ply::Header,
    mut reader: BufReader<&mut impl std::io::Read>,
  ) -> Result<Scene, ConversionError> {
    let parser = Parser::<ply::DefaultElement>::new();

    let payload =
      parser
        .read_payload(&mut reader, header)
        .map_err(|e| ConversionError::ParseError {
          format: "PLY".to_string(),
          message: e.to_string(),
        })?;

    let verticies = payload
      .get("vertex")
      .ok_or_else(|| ConversionError::ParseError {
        format: "PLY".to_string(),
        message: "Missing \"vertex\" element in PLY file".to_string(),
      })?;

    let mut splats: Vec<GaussianSplat> = Vec::with_capacity(verticies.len());

    for vert in verticies {
      let splat = GaussianSplat {
        position: [
          get_f32_property(vert, "x"),
          get_f32_property(vert, "y"),
          get_f32_property(vert, "z"),
        ],
        normal: [
          get_f32_property(vert, "nx"),
          get_f32_property(vert, "ny"),
          get_f32_property(vert, "nz"),
        ],
        spherical_harmonics_dc: [
          get_f32_property(vert, "f_dc_0"),
          get_f32_property(vert, "f_dc_1"),
          get_f32_property(vert, "f_dc_2"),
        ],
        spherical_harmonics_rest: get_spherical_harmonics_res(vert),
        opacity: get_f32_property(vert, "opacity"),
        scale: [
          get_f32_property(vert, "scale_0"),
          get_f32_property(vert, "scale_1"),
          get_f32_property(vert, "scale_2"),
        ],
        rotation: if vert.contains_key("rot_0") {
          [
            get_f32_property(vert, "rot_0"), // w
            get_f32_property(vert, "rot_1"), // x
            get_f32_property(vert, "rot_2"), // y
            get_f32_property(vert, "rot_3"), // z
          ]
        } else {
          [1.0, 0.0, 0.0, 0.0] // Identity: w=1, x=0, y=0, z=0
        },
      };
      splats.push(splat);
    }

    Ok(Scene { splats })
  }

  fn import_compressed(
    header: &ply::Header,
    mut reader: BufReader<&mut impl std::io::Read>,
  ) -> Result<Scene, ConversionError> {
    const CHUNK_SIZE: usize = 256;

    let parse_error = |message: &str| ConversionError::ParseError {
      format: "PLY (Compressed)".to_string(),
      message: message.to_string(),
    };

    let parser = Parser::<ply::DefaultElement>::new();
    let payload = parser
      .read_payload(&mut reader, header)
      .map_err(|e| parse_error(&format!("Failed to read payload: {}", e)))?;

    let chunks = payload
      .get("chunk")
      .ok_or_else(|| parse_error("Missing 'chunk' element"))?;
    let vertices = payload
      .get("vertex")
      .ok_or_else(|| parse_error("Missing 'vertex' element"))?;
    let sh_data = payload
      .get("sh")
      .ok_or_else(|| parse_error("Missing 'sh' element"))?;

    let mut splats = Vec::with_capacity(vertices.len());

    for (i, vert) in vertices.iter().enumerate() {
      let chunk_index = i / CHUNK_SIZE;
      if chunk_index >= chunks.len() {
        return Err(parse_error("Vertex index out of bounds for chunks."));
      }
      let chunk = &chunks[chunk_index];

      // --- Dequantize Position --- (No change)
      let packed_position = property_to_u32(&vert["packed_position"]);
      let ix = packed_position & 0x3FF;
      let iy = (packed_position >> 10) & 0x7FF;
      let iz = (packed_position >> 21) & 0x7FF;
      let min_x = property_to_f32(&chunk["min_x"]);
      let max_x = property_to_f32(&chunk["max_x"]);
      let min_y = property_to_f32(&chunk["min_y"]);
      let max_y = property_to_f32(&chunk["max_y"]);
      let min_z = property_to_f32(&chunk["min_z"]);
      let max_z = property_to_f32(&chunk["max_z"]);
      let position = [
        min_x + (ix as f32 / 1023.0) * (max_x - min_x),
        min_y + (iy as f32 / 2047.0) * (max_y - min_y),
        min_z + (iz as f32 / 2047.0) * (max_z - min_z),
      ];

      // --- Dequantize Scale ---
      let packed_scale = property_to_u32(&vert["packed_scale"]);
      let isx = packed_scale & 0x3FF;
      let isy = (packed_scale >> 10) & 0x7FF;
      let isz = (packed_scale >> 21) & 0x7FF;
      let min_scale_x = property_to_f32(&chunk["min_scale_x"]);
      let max_scale_x = property_to_f32(&chunk["max_scale_x"]);
      let min_scale_y = property_to_f32(&chunk["min_scale_y"]);
      let max_scale_y = property_to_f32(&chunk["max_scale_y"]);
      let min_scale_z = property_to_f32(&chunk["min_scale_z"]);
      let max_scale_z = property_to_f32(&chunk["max_scale_z"]);

      // Dequantize to linear scale
      let linear_scale_x = min_scale_x + (isx as f32 / 1023.0) * (max_scale_x - min_scale_x);
      let linear_scale_y = min_scale_y + (isy as f32 / 2047.0) * (max_scale_y - min_scale_y);
      let linear_scale_z = min_scale_z + (isz as f32 / 2047.0) * (max_scale_z - min_scale_z);

      // Convert to log scale for the struct, clamping to prevent ln(0) or ln(<0)
      let scale = [
        linear_scale_x.max(1e-8).ln(),
        linear_scale_y.max(1e-8).ln(),
        linear_scale_z.max(1e-8).ln(),
      ];

      // --- Dequantize Color (DC) and Opacity ---
      let packed_color = property_to_u32(&vert["packed_color"]);
      let r_u8 = (packed_color & 0xFF) as u8;
      let g_u8 = ((packed_color >> 8) & 0xFF) as u8;
      let b_u8 = ((packed_color >> 16) & 0xFF) as u8;
      let a_u8 = ((packed_color >> 24) & 0xFF) as u8;
      let min_r = property_to_f32(&chunk["min_r"]);
      let max_r = property_to_f32(&chunk["max_r"]);
      let min_g = property_to_f32(&chunk["min_g"]);
      let max_g = property_to_f32(&chunk["max_g"]);
      let min_b = property_to_f32(&chunk["min_b"]);
      let max_b = property_to_f32(&chunk["max_b"]);
      let spherical_harmonics_dc = [
        min_r + (r_u8 as f32 / 255.0) * (max_r - min_r),
        min_g + (g_u8 as f32 / 255.0) * (max_g - min_g),
        min_b + (b_u8 as f32 / 255.0) * (max_b - min_b),
      ];
      let normalized_opacity = (a_u8 as f32 / 255.0).clamp(1e-6, 1.0 - 1e-6);
      let opacity = (normalized_opacity / (1.0 - normalized_opacity)).ln();

      // --- Dequantize Rotation ---
      let packed_rotation = property_to_u32(&vert["packed_rotation"]);
      let rx_u8 = (packed_rotation & 0xFF) as u8;
      let ry_u8 = ((packed_rotation >> 8) & 0xFF) as u8;
      let rz_u8 = ((packed_rotation >> 16) & 0xFF) as u8;
      let rw_u8 = ((packed_rotation >> 24) & 0xFF) as u8;
      let mut rot_f32 = [
        (rx_u8 as f32 / 255.0) * 2.0 - 1.0,
        (ry_u8 as f32 / 255.0) * 2.0 - 1.0,
        (rz_u8 as f32 / 255.0) * 2.0 - 1.0,
        (rw_u8 as f32 / 255.0) * 2.0 - 1.0,
      ];
      let len =
        (rot_f32[0].powi(2) + rot_f32[1].powi(2) + rot_f32[2].powi(2) + rot_f32[3].powi(2)).sqrt();
      if len > 0.0 {
        for v in &mut rot_f32 {
          *v /= len;
        }
      }
      let rotation = [rot_f32[3], rot_f32[0], rot_f32[1], rot_f32[2]];

      // --- Dequantize Spherical Harmonics (Rest) ---
      let sh_element = &sh_data[i];
      let mut spherical_harmonics_rest = Vec::with_capacity(45);
      for j in 0..45 {
        let key = format!("f_rest_{}", j);
        let sh_u8 = property_to_u8(&sh_element[&key]);
        // Inverse of the new exporter formula: (nvalue * 256).floor()
        // We approximate the inverse by dividing by 256.
        let nvalue = sh_u8 as f32 / 256.0;
        let sh_f32 = (nvalue - 0.5) * 8.0;
        spherical_harmonics_rest.push(sh_f32);
      }

      splats.push(GaussianSplat {
        position,
        scale,
        rotation,
        spherical_harmonics_dc,
        opacity,
        spherical_harmonics_rest,
        normal: [0.0; 3],
      });
    }

    Ok(Scene { splats })
  }
}

impl Exporter for PlyASCIIExporter {
  fn export(scene: &Scene, writer: &mut impl std::io::Write) -> Result<(), ConversionError> {
    // A helper closure to map I/O errors to custom ConversionError type.
    let map_io_err = |e: std::io::Error| ConversionError::WriteError {
      format: "PLY".to_string(),
      message: e.to_string(),
    };

    // Write PLY header
    writeln!(writer, "ply").map_err(map_io_err)?;
    writeln!(writer, "format ascii 1.0").map_err(map_io_err)?;
    writeln!(writer, "element vertex {}", scene.splats.len()).map_err(map_io_err)?;
    writeln!(writer, "property float x").map_err(map_io_err)?;
    writeln!(writer, "property float y").map_err(map_io_err)?;
    writeln!(writer, "property float z").map_err(map_io_err)?;
    writeln!(writer, "property float nx").map_err(map_io_err)?;
    writeln!(writer, "property float ny").map_err(map_io_err)?;
    writeln!(writer, "property float nz").map_err(map_io_err)?;
    writeln!(writer, "property float f_dc_0").map_err(map_io_err)?;
    writeln!(writer, "property float f_dc_1").map_err(map_io_err)?;
    writeln!(writer, "property float f_dc_2").map_err(map_io_err)?;
    for i in 0..45 {
      writeln!(writer, "property float f_rest_{}", i).map_err(map_io_err)?;
    }
    writeln!(writer, "property float opacity").map_err(map_io_err)?;
    writeln!(writer, "property float scale_0").map_err(map_io_err)?;
    writeln!(writer, "property float scale_1").map_err(map_io_err)?;
    writeln!(writer, "property float scale_2").map_err(map_io_err)?;
    writeln!(writer, "property float rot_0").map_err(map_io_err)?;
    writeln!(writer, "property float rot_1").map_err(map_io_err)?;
    writeln!(writer, "property float rot_2").map_err(map_io_err)?;
    writeln!(writer, "property float rot_3").map_err(map_io_err)?;
    writeln!(writer, "end_header").map_err(map_io_err)?;

    // Write body
    for splat in &scene.splats {
      // Position, Normal, DC
      write!(
        writer,
        "{} {} {} {} {} {} {} {} {} ",
        splat.position[0],
        splat.position[1],
        splat.position[2],
        splat.normal[0],
        splat.normal[1],
        splat.normal[2],
        splat.spherical_harmonics_dc[0],
        splat.spherical_harmonics_dc[1],
        splat.spherical_harmonics_dc[2]
      )
      .map_err(map_io_err)?;

      // Rest of Spherical Harmonics
      for val in &splat.spherical_harmonics_rest {
        write!(writer, "{} ", val).map_err(map_io_err)?;
      }

      // Opacity, Scale, Rotation + newline
      writeln!(
        writer,
        "{} {} {} {} {} {} {} {}",
        splat.opacity,
        splat.scale[0],
        splat.scale[1],
        splat.scale[2],
        splat.rotation[0],
        splat.rotation[1],
        splat.rotation[2],
        splat.rotation[3]
      )
      .map_err(map_io_err)?;
    }

    Ok(())
  }
}

impl Exporter for PlyBinaryExporter {
  fn export(scene: &Scene, writer: &mut impl std::io::Write) -> Result<(), ConversionError> {
    let map_io_err = |e: std::io::Error| ConversionError::WriteError {
      format: "PLY (Binary)".to_string(),
      message: e.to_string(),
    };

    // Write header
    writeln!(writer, "ply").map_err(map_io_err)?;
    writeln!(writer, "format binary_little_endian 1.0").map_err(map_io_err)?;
    writeln!(writer, "element vertex {}", scene.splats.len()).map_err(map_io_err)?;
    writeln!(writer, "property float x").map_err(map_io_err)?;
    writeln!(writer, "property float y").map_err(map_io_err)?;
    writeln!(writer, "property float z").map_err(map_io_err)?;
    writeln!(writer, "property float nx").map_err(map_io_err)?;
    writeln!(writer, "property float ny").map_err(map_io_err)?;
    writeln!(writer, "property float nz").map_err(map_io_err)?;
    writeln!(writer, "property float f_dc_0").map_err(map_io_err)?;
    writeln!(writer, "property float f_dc_1").map_err(map_io_err)?;
    writeln!(writer, "property float f_dc_2").map_err(map_io_err)?;
    for i in 0..45 {
      writeln!(writer, "property float f_rest_{}", i).map_err(map_io_err)?;
    }
    writeln!(writer, "property float opacity").map_err(map_io_err)?;
    writeln!(writer, "property float scale_0").map_err(map_io_err)?;
    writeln!(writer, "property float scale_1").map_err(map_io_err)?;
    writeln!(writer, "property float scale_2").map_err(map_io_err)?;
    writeln!(writer, "property float rot_0").map_err(map_io_err)?;
    writeln!(writer, "property float rot_1").map_err(map_io_err)?;
    writeln!(writer, "property float rot_2").map_err(map_io_err)?;
    writeln!(writer, "property float rot_3").map_err(map_io_err)?;
    writeln!(writer, "end_header").map_err(map_io_err)?;

    // Write body
    for splat in &scene.splats {
      // Use the generic helper function instead of a closure
      Self::write_f32_slice(writer, &splat.position).map_err(map_io_err)?;
      Self::write_f32_slice(writer, &splat.normal).map_err(map_io_err)?;
      Self::write_f32_slice(writer, &splat.spherical_harmonics_dc).map_err(map_io_err)?;
      Self::write_f32_slice(writer, &splat.spherical_harmonics_rest).map_err(map_io_err)?;
      writer
        .write_all(&splat.opacity.to_le_bytes())
        .map_err(map_io_err)?;
      Self::write_f32_slice(writer, &splat.scale).map_err(map_io_err)?;
      Self::write_f32_slice(writer, &splat.rotation).map_err(map_io_err)?;
    }

    Ok(())
  }
}

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

    // --- Write PLY Header ---
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

    // --- Prepare binary buffers that exactly match the JS layout ---
    // chunk_f32: num_chunks * 18 floats
    let mut chunk_f32 = vec![0f32; num_chunks * 18];
    // vertex_u32: num_splats * 4 u32 fields (packed_position, rotation, scale, color)
    let mut vertex_u32 = vec![0u32; num_splats * 4];
    // sh bytes: only if has_sh_data
    let mut sh_bytes = if has_sh_data {
      vec![0u8; num_splats * 45]
    } else {
      Vec::new()
    };

    // iterate chunks in the same order as JS (and write output at the same offsets)
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

      // --- store chunk floats into chunk_f32 at i * 18 ---
      let base = i * 18;
      chunk_f32[base + 0] = min_pos[0];
      chunk_f32[base + 1] = min_pos[1];
      chunk_f32[base + 2] = min_pos[2];
      chunk_f32[base + 3] = max_pos[0];
      chunk_f32[base + 4] = max_pos[1];
      chunk_f32[base + 5] = max_pos[2];
      chunk_f32[base + 6] = min_scl_lin[0].max(1e-8); // clamp linear bounds to avoid ln(0)
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

      // number of real splats in this chunk
      // let num = chunk_splats_orig.len();

      // fill vertex_u32 at offset = i*CHUNK_SIZE*4
      let vertex_chunk_offset = i * CHUNK_SIZE * 4;
      for (j, splat) in chunk_splats_orig.iter().enumerate() {
        let write_idx = vertex_chunk_offset + j * 4;
        vertex_u32[write_idx + 0] =
          Self::quantize_pack_position(&splat.position, &min_pos, &max_pos);
        vertex_u32[write_idx + 1] = Self::quantize_pack_rotation(&splat.rotation);
        // quantize_pack_scale expects log_scale and linear min/max (we kept that signature)
        vertex_u32[write_idx + 2] =
          Self::quantize_pack_scale(&splat.scale, &min_scl_lin, &max_scl_lin);
        vertex_u32[write_idx + 3] = Self::pack_color_opacity(splat, &min_col, &max_col);

        if has_sh_data {
          let sh_off = (i * CHUNK_SIZE + j) * 45;
          let packed = Self::quantize_sh_rest(&splat.spherical_harmonics_rest);
          sh_bytes[sh_off..sh_off + 45].copy_from_slice(&packed);
        }
      }

      // Note: we DO NOT write the padded entries into vertex_u32. This matches
      // the JS exporter which leaves entries beyond num filled only when other chunks write there.
      // vertex_u32 is sized to num_splats * 4, so last chunk won't overflow.
    }

    // --- Write the binary buffers in the same order as the header ---
    // chunk_f32 -> floats in little-endian
    for &f in &chunk_f32 {
      writer.write_all(&f.to_le_bytes()).map_err(map_io_err)?;
    }

    // vertex_u32 -> u32 little-endian
    for &v in &vertex_u32 {
      writer.write_all(&v.to_le_bytes()).map_err(map_io_err)?;
    }

    // sh bytes if present
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
        min[i] = min[i].min(s.scale[i]);
        max[i] = max[i].max(s.scale[i]);
      }
    }
    (min, max)
  }

  fn calculate_color_bounds(splats: &[GaussianSplat]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for s in splats {
      for i in 0..3 {
        min[i] = min[i].min(s.spherical_harmonics_dc[i]);
        max[i] = max[i].max(s.spherical_harmonics_dc[i]);
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
      (norm * (bits - 1) as f32)
        .round()
        .clamp(0.0, (bits - 1) as f32) as u32
    };
    let ix = quantize(pos[0], min[0], max[0], 1024);
    let iy = quantize(pos[1], min[1], max[1], 2048);
    let iz = quantize(pos[2], min[2], max[2], 2048);
    ix | (iy << 10) | (iz << 21)
  }

  fn quantize_pack_scale(log_scale: &[f32; 3], min: &[f32; 3], max: &[f32; 3]) -> u32 {
    let quantize = |val: f32, min_v: f32, max_v: f32, bits: u32| -> u32 {
      if (max_v - min_v).abs() < 1e-8 {
        return 0;
      }
      let norm = (val - min_v) / (max_v - min_v);
      (norm * (bits - 1) as f32)
        .round()
        .clamp(0.0, (bits - 1) as f32) as u32
    };
    let isx = quantize(log_scale[0], min[0], max[0], 1024);
    let isy = quantize(log_scale[1], min[1], max[1], 2048);
    let isz = quantize(log_scale[2], min[2], max[2], 2048);
    isx | (isy << 10) | (isz << 21)
  }

  // Rotation packing is correct.
  fn quantize_pack_rotation(rot: &[f32; 4]) -> u32 {
    let mut norm_rot = *rot;
    let len =
      (norm_rot[0].powi(2) + norm_rot[1].powi(2) + norm_rot[2].powi(2) + norm_rot[3].powi(2))
        .sqrt();
    if len > 1e-8 {
      for v in &mut norm_rot {
        *v /= len;
      }
    }
    let to_u8 = |v: f32| (((v.clamp(-1.0, 1.0) + 1.0) / 2.0) * 255.0).round() as u32;
    let x = to_u8(norm_rot[1]);
    let y = to_u8(norm_rot[2]);
    let z = to_u8(norm_rot[3]);
    let w = to_u8(norm_rot[0]);
    x | (y << 8) | (z << 16) | (w << 24)
  }

  fn pack_color_opacity(splat: &GaussianSplat, min: &[f32; 3], max: &[f32; 3]) -> u32 {
    let quantize_color = |val: f32, min_v: f32, max_v: f32| -> u32 {
      if (max_v - min_v).abs() < 1e-8 {
        return 0;
      }
      let norm = (val - min_v) / (max_v - min_v);
      (norm * 255.0).round().clamp(0.0, 255.0) as u32
    };

    let r = quantize_color(splat.spherical_harmonics_dc[0], min[0], max[0]);
    let g = quantize_color(splat.spherical_harmonics_dc[1], min[1], max[1]);
    let b = quantize_color(splat.spherical_harmonics_dc[2], min[2], max[2]);

    let sigmoid = 1.0 / (1.0 + (-splat.opacity).exp());
    let a = (sigmoid * 255.0).round().clamp(0.0, 255.0) as u32;

    // Pack as RGBA to match the importer's expectation.
    r | (g << 8) | (b << 16) | (a << 24)
  }

  // SH (Rest) packing is correct.
  fn quantize_sh_rest(sh: &[f32]) -> [u8; 45] {
    let mut packed = [0u8; 45];
    for i in 0..45 {
      let val = sh.get(i).copied().unwrap_or(0.0);
      let nvalue = (val / 8.0) + 0.5;
      packed[i] = (nvalue * 256.0).floor().clamp(0.0, 255.0) as u8;
    }
    packed
  }
}

impl PlyBinaryExporter {
  /// Helper function to write a slice of f32 values as little-endian bytes.
  /// This function is generic over the `Write` trait.
  fn write_f32_slice<W: std::io::Write>(writer: &mut W, data: &[f32]) -> std::io::Result<()> {
    for &val in data {
      writer.write_all(&val.to_le_bytes())?;
    }
    Ok(())
  }
}

fn property_to_u32(prop: &ply::Property) -> u32 {
  match prop {
    ply::Property::UInt(i) => *i,
    _ => 0, // Fallback
  }
}

fn property_to_u8(prop: &ply::Property) -> u8 {
  match prop {
    ply::Property::UChar(c) => *c,
    _ => 0, // Fallback
  }
}
