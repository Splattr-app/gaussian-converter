use std::io::BufReader;

use crate::{ConversionError, GaussianSplat, Importer, Scene};
use ply_rs::parser::Parser;
use ply_rs::ply;

use super::helpers::{
  get_f32_property, get_spherical_harmonics_res, property_to_f32, property_to_u8, property_to_u32,
};

pub struct PlyImporter;

/// Unpacks an 11-10-11 bit value (x in MSB) into normalized [0,1] floats.
fn unpack_111011(v: u32) -> (f32, f32, f32) {
  (
    ((v >> 21) & 0x7FF) as f32 / 2047.0,
    ((v >> 11) & 0x3FF) as f32 / 1023.0,
    (v & 0x7FF) as f32 / 2047.0,
  )
}

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
    let sh_data = payload.get("sh");

    let mut splats = Vec::with_capacity(vertices.len());

    for (i, vert) in vertices.iter().enumerate() {
      let chunk_index = i / CHUNK_SIZE;
      if chunk_index >= chunks.len() {
        return Err(parse_error("Vertex index out of bounds for chunks."));
      }
      let chunk = &chunks[chunk_index];

      let f = |c: &ply::DefaultElement, k: &str| property_to_f32(&c[k]);
      let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

      // --- Position: x=11 (MSB), y=10, z=11 (LSB) ---
      let p = property_to_u32(&vert["packed_position"]);
      let (nx, ny, nz) = unpack_111011(p);
      let position = [
        lerp(f(chunk, "min_x"), f(chunk, "max_x"), nx),
        lerp(f(chunk, "min_y"), f(chunk, "max_y"), ny),
        lerp(f(chunk, "min_z"), f(chunk, "max_z"), nz),
      ];

      // --- Scale: chunk bounds are in log space, same layout as position ---
      let p = property_to_u32(&vert["packed_scale"]);
      let (sx, sy, sz) = unpack_111011(p);
      let scale = [
        lerp(f(chunk, "min_scale_x"), f(chunk, "max_scale_x"), sx),
        lerp(f(chunk, "min_scale_y"), f(chunk, "max_scale_y"), sy),
        lerp(f(chunk, "min_scale_z"), f(chunk, "max_scale_z"), sz),
      ];

      // --- Color (R:G:B:A, R in MSB). RGB = f_dc * SH_C0 + 0.5, A = sigmoid(opacity) ---
      const SH_C0: f32 = 0.28209479177387814;
      let packed_color = property_to_u32(&vert["packed_color"]);
      let r = ((packed_color >> 24) & 0xFF) as f32 / 255.0;
      let g = ((packed_color >> 16) & 0xFF) as f32 / 255.0;
      let b = ((packed_color >> 8) & 0xFF) as f32 / 255.0;
      let a = (packed_color & 0xFF) as f32 / 255.0;
      let spherical_harmonics_dc = [
        (lerp(f(chunk, "min_r"), f(chunk, "max_r"), r) - 0.5) / SH_C0,
        (lerp(f(chunk, "min_g"), f(chunk, "max_g"), g) - 0.5) / SH_C0,
        (lerp(f(chunk, "min_b"), f(chunk, "max_b"), b) - 0.5) / SH_C0,
      ];
      let a = a.clamp(1.0 / 255.0 * 0.5, 1.0 - 1.0 / 255.0 * 0.5);
      let opacity = (a / (1.0 - a)).ln();

      // --- Rotation: 2-bit largest index, then three 10-bit values (w,x,y,z order) ---
      let packed_rotation = property_to_u32(&vert["packed_rotation"]);
      let sqrt2 = std::f32::consts::SQRT_2;
      let comp = |shift: u32| (((packed_rotation >> shift) & 1023) as f32 / 1023.0 - 0.5) * sqrt2;
      let (ca, cb, cc) = (comp(20), comp(10), comp(0));
      let m = (1.0 - (ca * ca + cb * cb + cc * cc)).max(0.0).sqrt();
      let rotation = match packed_rotation >> 30 {
        0 => [m, ca, cb, cc],
        1 => [ca, m, cb, cc],
        2 => [ca, cb, m, cc],
        _ => [ca, cb, cc, m],
      };

      // --- Spherical harmonics (rest), optional ---
      let mut spherical_harmonics_rest = vec![0.0f32; 45];
      if let Some(sh_data) = sh_data {
        let sh_element = &sh_data[i];
        for (j, out) in spherical_harmonics_rest.iter_mut().enumerate() {
          if let Some(prop) = sh_element.get(&format!("f_rest_{}", j)) {
            let sh_u8 = property_to_u8(prop);
            *out = ((sh_u8 as f32 + 0.5) / 256.0 - 0.5) * 8.0;
          }
        }
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
