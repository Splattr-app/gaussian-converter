use crate::{ConversionError, Exporter, GaussianSplat, Importer, Scene};
use ply_rs::parser::Parser;
use ply_rs::ply;

pub struct PlyImporter;
pub struct PlyASCIIExporter;
pub struct PlyBinaryExporter;

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
          property_to_f32(&vert["rot_0"]), // w
          property_to_f32(&vert["rot_1"]), // x
          property_to_f32(&vert["rot_2"]), // y
          property_to_f32(&vert["rot_3"]), // z
        ],
      };
      splats.push(splat);
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
