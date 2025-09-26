use crate::{ConversionError, Exporter, Scene};

use super::super::helpers::write_f32_slice;

pub struct PlyBinaryExporter;

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
      write_f32_slice(writer, &splat.position).map_err(map_io_err)?;
      write_f32_slice(writer, &splat.normal).map_err(map_io_err)?;
      write_f32_slice(writer, &splat.spherical_harmonics_dc).map_err(map_io_err)?;
      write_f32_slice(writer, &splat.spherical_harmonics_rest).map_err(map_io_err)?;
      writer
        .write_all(&splat.opacity.to_le_bytes())
        .map_err(map_io_err)?;
      write_f32_slice(writer, &splat.scale).map_err(map_io_err)?;
      write_f32_slice(writer, &splat.rotation).map_err(map_io_err)?;
    }

    Ok(())
  }
}
