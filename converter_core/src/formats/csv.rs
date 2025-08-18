use crate::{ConversionError, Exporter, GaussianSplat, Importer, Scene};
use csv::{ReaderBuilder, WriterBuilder};
use std::io::{Read, Write};

pub struct CsvImporter;
pub struct CsvExporter;

// --- Exporter ---

impl Exporter for CsvExporter {
  fn export(scene: &Scene, writer: &mut impl Write) -> Result<(), ConversionError> {
    let mut wtr = WriterBuilder::new().from_writer(writer);

    // Header

    // Create a Vec<String> to ensure ownership of all header names.
    let mut headers: Vec<String> = vec![
      "x", "y", "z", "nx", "ny", "nz", "f_dc_0", "f_dc_1", "f_dc_2", "opacity", "scale_0",
      "scale_1", "scale_2", "rot_0", "rot_1", "rot_2", "rot_3",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    for i in 0..45 {
      headers.push(format!("f_rest_{}", i));
    }

    wtr
      .write_record(&headers)
      .map_err(|e| ConversionError::WriteError {
        format: "CSV".to_string(),
        message: e.to_string(),
      })?;

    // Write splat data
    for splat in &scene.splats {
      let mut record = Vec::new();

      record.push(splat.position[0].to_string());
      record.push(splat.position[1].to_string());
      record.push(splat.position[2].to_string());

      record.push(splat.normal[0].to_string());
      record.push(splat.normal[1].to_string());
      record.push(splat.normal[2].to_string());

      record.push(splat.spherical_harmonics_dc[0].to_string());
      record.push(splat.spherical_harmonics_dc[1].to_string());
      record.push(splat.spherical_harmonics_dc[2].to_string());

      record.push(splat.opacity.to_string());

      record.push(splat.scale[0].to_string());
      record.push(splat.scale[1].to_string());
      record.push(splat.scale[2].to_string());

      record.push(splat.rotation[0].to_string());
      record.push(splat.rotation[1].to_string());
      record.push(splat.rotation[2].to_string());
      record.push(splat.rotation[3].to_string());

      // Pad SH Rest data to ensure consistent column count
      let mut sh_rest_padded = splat.spherical_harmonics_rest.clone();
      sh_rest_padded.resize(45, 0.0);
      for val in sh_rest_padded {
        record.push(val.to_string());
      }

      wtr
        .write_record(&record)
        .map_err(|e| ConversionError::WriteError {
          format: "CSV".to_string(),
          message: e.to_string(),
        })?;
    }

    wtr.flush().map_err(|e| ConversionError::WriteError {
      format: "CSV".to_string(),
      message: e.to_string(),
    })?;
    Ok(())
  }
}

// --- Importer ---

impl Importer for CsvImporter {
  fn import(reader: &mut impl Read) -> Result<Scene, ConversionError> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(reader);
    let mut splats = Vec::new();

    let parse_f32 = |field: &str, col_name: &str| -> Result<f32, ConversionError> {
      field
        .parse::<f32>()
        .map_err(|_| ConversionError::ParseError {
          format: "CSV".to_string(),
          message: format!(
            "Could not parse field '{}' as f32 for column {}",
            field, col_name
          ),
        })
    };

    for result in rdr.records() {
      let record = result.map_err(|e| ConversionError::ParseError {
        format: "CSV".to_string(),
        message: e.to_string(),
      })?;

      if record.len() < 17 + 45 {
        return Err(ConversionError::ParseError {
          format: "CSV".to_string(),
          message: "Row has insufficient columns.".to_string(),
        });
      }

      let mut sh_rest = Vec::with_capacity(45);
      for i in 0..45 {
        sh_rest.push(parse_f32(&record[17 + i], &format!("f_rest_{}", i))?);
      }

      let splat = GaussianSplat {
        position: [
          parse_f32(&record[0], "x")?,
          parse_f32(&record[1], "y")?,
          parse_f32(&record[2], "z")?,
        ],
        normal: [
          parse_f32(&record[3], "nx")?,
          parse_f32(&record[4], "ny")?,
          parse_f32(&record[5], "nz")?,
        ],
        spherical_harmonics_dc: [
          parse_f32(&record[6], "f_dc_0")?,
          parse_f32(&record[7], "f_dc_1")?,
          parse_f32(&record[8], "f_dc_2")?,
        ],
        opacity: parse_f32(&record[9], "opacity")?,
        scale: [
          parse_f32(&record[10], "scale_0")?,
          parse_f32(&record[11], "scale_1")?,
          parse_f32(&record[12], "scale_2")?,
        ],
        rotation: [
          parse_f32(&record[13], "rot_0")?,
          parse_f32(&record[14], "rot_1")?,
          parse_f32(&record[15], "rot_2")?,
          parse_f32(&record[16], "rot_3")?,
        ],
        spherical_harmonics_rest: sh_rest,
      };
      splats.push(splat);
    }

    Ok(Scene { splats })
  }
}
