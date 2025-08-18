use std::io::BufWriter;
use std::{env, fs::File, io::BufReader};

use converter_core::formats::ply::PlyBinaryExporter;
use converter_core::formats::spz::SpzImporter;
use converter_core::{Exporter, Importer};

fn main() {
  let cwd = env::current_dir().unwrap();
  println!("Current working directory: {}", cwd.display());

  let file = File::open("./converter_core/hornedlizard.spz").unwrap();
  let mut reader = BufReader::new(file);

  let result = SpzImporter::import(&mut reader);

  assert!(
    result.is_ok(),
    "Importer should successfully parse valid PLY data"
  );
}

#[test]
fn convert_spz_to_ply() -> Result<(), Box<dyn std::error::Error>> {
  const FILENAME: &str = "hornedlizard";

  let file = File::open(format!("./{}.spz", FILENAME))?;
  let mut reader = BufReader::new(file);

  let scene = SpzImporter::import(&mut reader)?;

  let file: File = File::create(format!("./{}_converted.ply", FILENAME))?;
  let mut writer = BufWriter::new(file);

  let _ = PlyBinaryExporter::export(&scene, &mut writer);

  println!("Converted scene {}.spz", FILENAME);

  Ok(())
}
