use crate::{ConversionError, Exporter, Scene};

pub struct PlyASCIIExporter;

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
