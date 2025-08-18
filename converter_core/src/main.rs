use std::io::BufWriter;
use std::{env, fs::File, io::BufReader};

use converter_core::formats::ply::{PlyASCIIExporter, PlyBinaryExporter, PlyImporter};
use converter_core::formats::splat::{SplatExporter, SplatImporter};
use converter_core::formats::spz::{SpzImporter, SpzV2Exporter};
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

  println!("Converted scene {}.ply", FILENAME);

  Ok(())
}

#[test]
fn convert_ply_to_spz() -> Result<(), Box<dyn std::error::Error>> {
  const FILENAME: &str = "ball_refined";

  let file = File::open(format!("./{}.ply", FILENAME))?;
  let mut reader = BufReader::new(file);

  let scene = PlyImporter::import(&mut reader)?;

  let file: File = File::create(format!("./{}_converted.spz", FILENAME))?;
  let mut writer = BufWriter::new(file);

  let _ = SpzV2Exporter::export(&scene, &mut writer);

  println!("Converted scene {}.spz", FILENAME);

  Ok(())
}

#[test]
fn convert_ply_to_splat() -> Result<(), Box<dyn std::error::Error>> {
  const FILENAME: &str = "ball_refined";

  let file = File::open(format!("./{}.ply", FILENAME))?;
  let mut reader = BufReader::new(file);

  let scene = PlyImporter::import(&mut reader)?;

  let file: File = File::create(format!("./{}_converted.splat", FILENAME))?;
  let mut writer = BufWriter::new(file);

  let _ = SplatExporter::export(&scene, &mut writer);

  println!("Converted scene {}.splat", FILENAME);

  Ok(())
}

#[test]
fn convert_splat_to_spz() -> Result<(), Box<dyn std::error::Error>> {
  const FILENAME: &str = "baby_yoda";

  let file = File::open(format!("./{}.splat", FILENAME))?;
  let mut reader = BufReader::new(file);

  let scene = SplatImporter::import(&mut reader)?;

  let file: File = File::create(format!("./{}_converted.spz", FILENAME))?;
  let mut writer = BufWriter::new(file);

  let _ = SpzV2Exporter::export(&scene, &mut writer);

  println!("Converted scene {}.spz", FILENAME);

  Ok(())
}
