use converter_core::{
  formats::{
    csv::{CsvExporter, CsvImporter},
    ply::{PlyASCIIExporter, PlyBinaryExporter, PlyImporter},
    splat::{SplatExporter, SplatImporter},
    spz::{SpzImporter, SpzV2Exporter},
  },
  Exporter, Importer,
};
use serde::Serialize;
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use uuid::Uuid;

#[derive(Serialize)]
struct ConversionResult {
  path: String,
  size: u64, // File size in bytes
}

#[tauri::command]
async fn convert_to_temp_file(
  input_path: String,
  source_format: &str,
  target_format: &str,
) -> Result<ConversionResult, String> {
  let temp_dir = env::temp_dir();
  let unique_id = Uuid::new_v4().to_string();

  let temp_file_name = format!(
    "{}.{}",
    unique_id,
    target_format.split('_').last().unwrap_or("bin")
  );
  let temp_file_path = temp_dir.join(temp_file_name);

  let input_file = File::open(&input_path).map_err(|e| e.to_string())?;
  let mut reader = BufReader::new(input_file);

  let scene = match source_format {
    "ply" => PlyImporter::import(&mut reader),
    "spz" => SpzImporter::import(&mut reader),
    "csv" => CsvImporter::import(&mut reader),
    "splat" => SplatImporter::import(&mut reader),
    _ => Err(converter_core::ConversionError::UnsupportedFormat),
  }
  .map_err(|e| e.to_string())?;

  let output_file = File::create(&temp_file_path).map_err(|e| e.to_string())?;
  let mut writer = BufWriter::new(output_file);

  match target_format {
    "ascii_ply" => PlyASCIIExporter::export(&scene, &mut writer),
    "binary_ply" => PlyBinaryExporter::export(&scene, &mut writer),
    "spz_v2" => SpzV2Exporter::export(&scene, &mut writer),
    "csv" => CsvExporter::export(&scene, &mut writer),
    "splat" => SplatExporter::export(&scene, &mut writer),
    _ => Err(converter_core::ConversionError::UnsupportedFormat),
  }
  .map_err(|e| e.to_string())?;

  let metadata = fs::metadata(&temp_file_path).map_err(|e| e.to_string())?;

  Ok(ConversionResult {
    path: temp_file_path.to_string_lossy().into_owned(),
    size: metadata.len(),
  })
}

#[tauri::command]
async fn save_converted_file(temp_path: String, final_path: String) -> Result<(), String> {
  fs::rename(temp_path, final_path).map_err(|e| e.to_string())?;
  Ok(())
}

#[derive(Serialize)]
struct FileMetadata {
  name: String,
  size: u64,
}

#[tauri::command]
async fn get_file_metadata(path: String) -> Result<FileMetadata, String> {
  let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
  let name = Path::new(&path)
    .file_name()
    .and_then(|s| s.to_str())
    .unwrap_or("unknown")
    .to_string();

  Ok(FileMetadata {
    name,
    size: metadata.len(),
  })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![
      convert_to_temp_file,
      save_converted_file,
      get_file_metadata
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
