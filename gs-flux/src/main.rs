use clap::Parser;
use converter_core::{
  ConversionError, Exporter, Importer, Scene,
  formats::{
    csv::{CsvExporter, CsvImporter},
    ply::{PlyASCIIExporter, PlyBinaryExporter, PlyImporter},
    splat::{SplatExporter, SplatImporter},
    spz::{SpzImporter, SpzV2Exporter},
  },
};
use indicatif::{ProgressBar, ProgressStyle};
use std::{
  fs::File,
  io::{BufReader, BufWriter},
  path::Path,
  process, thread,
  time::{Duration, Instant},
};

const SUPPORTED_FORMATS: [&str; 4] = ["ply", "spz", "splat", "csv"];

#[derive(Parser, Debug)]
#[command(name = "GS-Flux", version, about = "Convert gaussian splatting files")]
struct Params {
  /// Source file
  #[arg(value_parser = validate_input_path)]
  source_file: String,

  /// The target filename
  #[arg(value_parser = validate_output_path)]
  output_file: String,

  /// Output encoding (Valid only for .ply)
  #[arg(long, value_parser = ["ascii", "binary"])]
  encoding: Option<String>,
  //
  // Output version (no need to implement yet, just something for the future.)
  // #[arg(long)]
  // version: Option<u32>,
}

fn main() {
  let params = Params::parse();

  let input_ext = Path::new(&params.source_file)
    .extension()
    .and_then(|ext| ext.to_str())
    .unwrap_or("")
    .to_lowercase();

  let output_ext = Path::new(&params.output_file)
    .extension()
    .and_then(|ext| ext.to_str())
    .unwrap_or("")
    .to_lowercase();

  // Start timer
  let start = Instant::now();

  let spinner = ProgressBar::new_spinner();
  spinner.set_style(
    ProgressStyle::default_spinner()
      .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
      .template("{spinner:.blue} {msg}")
      .unwrap(),
  );
  spinner.set_message("Converting...");
  spinner.enable_steady_tick(Duration::from_millis(100));

  let conversion_thread = thread::spawn({
    let source_file = params.source_file.clone();
    let output_file = params.output_file.clone();
    let encoding = params.encoding.clone();
    move || {
      convert(
        &source_file,
        &input_ext,
        &output_file,
        &output_ext,
        encoding.as_deref(),
      )
    }
  });

  let result = conversion_thread.join().unwrap();

  // Calculate elapsed time
  let elapsed = start.elapsed();
  let elapsed_secs = elapsed.as_secs_f32();

  match result {
    Ok(_) => {
      let finish_message = format!("✔ Done in {:.2}s", elapsed_secs);
      spinner.set_style(ProgressStyle::default_spinner().template("{msg}").unwrap());
      spinner.finish_with_message(finish_message);
    }
    Err(e) => {
      spinner.abandon_with_message("✖ Conversion failed");
      eprintln!("\nError: {:?}", e);
      process::exit(1);
    }
  }
}

fn validate_extension(s: &str) -> Result<(), String> {
  let extension = s
    .rsplit('.')
    .next()
    .ok_or("File must have an extension")?
    .to_lowercase();

  if !SUPPORTED_FORMATS.contains(&extension.as_str()) {
    return Err(format!(
      "File must have one of the following extensions: {}",
      SUPPORTED_FORMATS
        .iter()
        .map(|ext| format!(".{}", ext))
        .collect::<Vec<_>>()
        .join(", ")
    ));
  }

  Ok(())
}

fn validate_input_path(s: &str) -> Result<String, String> {
  validate_extension(s)?;

  let path = Path::new(s);
  if !path.is_file() {
    return Err("Invalid input: file does not exist".to_string());
  }

  Ok(s.to_string())
}

fn validate_output_path(s: &str) -> Result<String, String> {
  validate_extension(s)?;

  let path = Path::new(s);
  let parent = path.parent().unwrap_or_else(|| Path::new(""));

  if !parent.as_os_str().is_empty() && !parent.exists() {
    return Err(format!(
      "Invalid output: parent folder '{}' does not exist",
      parent.display()
    ));
  }

  Ok(s.to_string())
}

fn convert(
  input_file_path: &str,
  input_file_type: &str,
  output_file_path: &str,
  output_file_type: &str,
  encoding: Option<&str>,
  // version: Option<u32>,
) -> Result<(), ConversionError> {
  let file = File::open(input_file_path)?;
  let mut reader = BufReader::new(file);

  let scene: Scene = match input_file_type {
    "ply" => PlyImporter::import(&mut reader)?,
    "spz" => SpzImporter::import(&mut reader)?,
    "csv" => CsvImporter::import(&mut reader)?,
    "splat" => SplatImporter::import(&mut reader)?,
    _ => return Err(ConversionError::UnsupportedFormat),
  };

  let file: File = File::create(output_file_path)?;
  let mut writer = BufWriter::new(file);

  match output_file_type {
    "ply" => match encoding.unwrap_or("binary".into()) {
      "ascii" => PlyASCIIExporter::export(&scene, &mut writer)?,
      "binary" => PlyBinaryExporter::export(&scene, &mut writer)?,
      other => {
        return Err(ConversionError::ParseError {
          format: "PLY".to_string(),
          message: format!("Unsupported PLY encoding: {}", other),
        });
      }
    },
    "spz" => SpzV2Exporter::export(&scene, &mut writer)?,
    "csv" => CsvExporter::export(&scene, &mut writer)?,
    "splat" => SplatExporter::export(&scene, &mut writer)?,
    _ => return Err(ConversionError::UnsupportedFormat),
  };

  Ok(())
}
