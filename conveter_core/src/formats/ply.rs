use crate::{ConversionError, GaussianSplat, Importer, Scene};
use ply_rs::parser::Parser;
use ply_rs::ply;

pub struct PlyImporter;
pub struct PlyExporter;

fn property_to_f32(prop: &ply::Property) -> f32 {
  match prop {
    ply::Property::Float(f) => *f,
    _ => 0.0, // fallback for unsupported property types
  }
}

fn get_spherical_harmonics_res(vert: &ply::DefaultElement) -> Vec<f32> {
  let base_key = "f_rest_";
  let mut output: Vec<f32> = Vec::new();

  for i in 0..45 {
    let key = format!("{}{}", base_key, i);
    let value = vert.get(&key).map_or(0.0, |prop| property_to_f32(prop));
    output.push(value);
  }

  output
}

impl Importer for PlyImporter {
  fn import(reader: &mut impl std::io::Read) -> Result<Scene, ConversionError> {
    let parser = Parser::<ply::DefaultElement>::new();
    let ply = parser
      .read_ply(reader)
      .map_err(|e| ConversionError::ParseError {
        format: "PLY".to_string(),
        message: e.to_string(),
      })?;

    let verticies = ply
      .payload
      .get("vertex")
      .ok_or_else(|| ConversionError::ParseError {
        format: "PLY".to_string(),
        message: "Missing \"vertex\" element in PLY file".to_string(),
      })?;

    let mut splats: Vec<GaussianSplat> = Vec::with_capacity(verticies.len());

    for vert in verticies {
      let splat = GaussianSplat {
        position: [
          property_to_f32(&vert["x"]),
          property_to_f32(&vert["y"]),
          property_to_f32(&vert["z"]),
        ],
        normal: [
          property_to_f32(&vert["nx"]),
          property_to_f32(&vert["ny"]),
          property_to_f32(&vert["nz"]),
        ],
        spherical_harmonics_dc: [
          property_to_f32(&vert["f_dc_0"]),
          property_to_f32(&vert["f_dc_1"]),
          property_to_f32(&vert["f_dc_2"]),
        ],
        spherical_harmonics_rest: get_spherical_harmonics_res(vert),
        opacity: property_to_f32(&vert["opacity"]),
        scale: [
          property_to_f32(&vert["scale_0"]),
          property_to_f32(&vert["scale_1"]),
          property_to_f32(&vert["scale_2"]),
        ],
        rotation: [
          property_to_f32(&vert["rot_0"]),
          property_to_f32(&vert["rot_1"]),
          property_to_f32(&vert["rot_2"]),
          property_to_f32(&vert["rot_3"]),
        ],
      };
      splats.push(splat);
    }

    Ok(Scene { splats })
  }
}
