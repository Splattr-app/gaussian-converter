use std::io::BufReader;

use crate::{ConversionError, GaussianSplat, Importer, Scene};
use ply_rs::parser::Parser;
use ply_rs::ply;

use super::helpers::{
  get_f32_property, get_spherical_harmonics_res, property_to_f32, property_to_u8, property_to_u32,
};

pub struct PlyImporter;

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
