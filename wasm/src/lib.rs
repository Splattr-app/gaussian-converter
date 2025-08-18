use wasm_bindgen::prelude::*;

use converter_core::{
  ConversionError, Exporter, Importer, Scene,
  formats::{
    csv::{CsvExporter, CsvImporter},
    ply::{PlyBinaryExporter, PlyImporter},
    splat::{SplatExporter, SplatImporter},
    spz::{SpzImporter, SpzV2Exporter},
  },
};

#[wasm_bindgen]
pub fn convert(
  input_data: &[u8],
  source_format: &str,
  target_format: &str,
) -> Result<Vec<u8>, JsValue> {
  run_conversion(input_data, source_format, target_format)
    .map_err(|err| JsValue::from_str(&err.to_string()))
}

fn run_conversion(
  input_data: &[u8],
  source_format: &str,
  target_format: &str,
) -> Result<Vec<u8>, ConversionError> {
  // Import data from input bytes

  // Use `&[u8]` as a reader. It implements `std::io::Read` directly.
  let mut reader = input_data;

  let scene: Scene = match source_format {
    "ply" => PlyImporter::import(&mut reader)?,
    "spz" => SpzImporter::import(&mut reader)?,
    "csv" => CsvImporter::import(&mut reader)?,
    "splat" => SplatImporter::import(&mut reader)?,
    _ => return Err(ConversionError::UnsupportedFormat),
  };

  // Export the scene into a new byte vector

  // `Vec<u8>` can be used as a writer. It implements `std::io::Write`.
  let mut writer = Vec::new();

  match target_format {
    "ply" => PlyBinaryExporter::export(&scene, &mut writer)?,
    "spz" => SpzV2Exporter::export(&scene, &mut writer)?,
    "csv" => CsvExporter::export(&scene, &mut writer)?,
    "splat" => SplatExporter::export(&scene, &mut writer)?,
    _ => return Err(ConversionError::UnsupportedFormat),
  };

  Ok(writer)
}
