#[cfg(test)]
use std::io::BufWriter;
#[cfg(test)]
use std::{fs::File, io::BufReader};

#[cfg(test)]
use converter_core::formats::{
  csv::{CsvExporter, CsvImporter},
  ply::{PlyBinaryExporter, PlyImporter},
  splat::{SplatExporter, SplatImporter},
  spz::{SpzImporter, SpzV2Exporter},
};

#[cfg(test)]
use converter_core::{Exporter, Importer};

fn main() {}

#[test]
fn convert_spz_to_ply() -> Result<(), Box<dyn std::error::Error>> {
  const FILENAME: &str = "hornedlizard";

  let file = File::open(format!("./test_data/{}.spz", FILENAME))?;
  let mut reader = BufReader::new(file);

  let scene = SpzImporter::import(&mut reader)?;

  let file: File = File::create(format!("./test_data/{}_converted.ply", FILENAME))?;
  let mut writer = BufWriter::new(file);

  let _ = PlyBinaryExporter::export(&scene, &mut writer);

  println!("Converted scene {}.ply", FILENAME);

  Ok(())
}

#[test]
fn convert_ply_to_spz() -> Result<(), Box<dyn std::error::Error>> {
  const FILENAME: &str = "ball_refined";

  let file = File::open(format!("./test_data/{}.ply", FILENAME))?;
  let mut reader = BufReader::new(file);

  let scene = PlyImporter::import(&mut reader)?;

  let file: File = File::create(format!("./test_data/{}_converted.spz", FILENAME))?;
  let mut writer = BufWriter::new(file);

  let _ = SpzV2Exporter::export(&scene, &mut writer);

  println!("Converted scene {}.spz", FILENAME);

  Ok(())
}

#[test]
fn convert_ply_to_splat() -> Result<(), Box<dyn std::error::Error>> {
  const FILENAME: &str = "ball_refined";

  let file = File::open(format!("./test_data/{}.ply", FILENAME))?;
  let mut reader = BufReader::new(file);

  let scene = PlyImporter::import(&mut reader)?;

  let file: File = File::create(format!("./test_data/{}_converted.splat", FILENAME))?;
  let mut writer = BufWriter::new(file);

  let _ = SplatExporter::export(&scene, &mut writer);

  println!("Converted scene {}.splat", FILENAME);

  Ok(())
}

#[test]
fn convert_ply_to_ply() -> Result<(), Box<dyn std::error::Error>> {
  const FILENAME: &str = "ball_refined";

  let file = File::open(format!("./test_data/{}.ply", FILENAME))?;
  let mut reader = BufReader::new(file);

  let scene = PlyImporter::import(&mut reader)?;

  let file: File = File::create(format!("./test_data/{}_converted.ply", FILENAME))?;
  let mut writer = BufWriter::new(file);

  let _ = PlyBinaryExporter::export(&scene, &mut writer);

  println!("Converted scene {}.ply", FILENAME);

  Ok(())
}

#[test]
fn convert_splat_to_spz() -> Result<(), Box<dyn std::error::Error>> {
  const FILENAME: &str = "baby_yoda";

  let file = File::open(format!("./test_data/{}.splat", FILENAME))?;
  let mut reader = BufReader::new(file);

  let scene = SplatImporter::import(&mut reader)?;

  let file: File = File::create(format!("./test_data/{}_converted.spz", FILENAME))?;
  let mut writer = BufWriter::new(file);

  let _ = SpzV2Exporter::export(&scene, &mut writer);

  println!("Converted scene {}.splat", FILENAME);

  Ok(())
}

#[test]
fn convert_splat_to_csv() -> Result<(), Box<dyn std::error::Error>> {
  const FILENAME: &str = "baby_yoda";

  let file = File::open(format!("./test_data/{}.splat", FILENAME))?;
  let mut reader = BufReader::new(file);

  let scene = SplatImporter::import(&mut reader)?;

  let file: File = File::create(format!("./test_data/{}_converted.csv", FILENAME))?;
  let mut writer = BufWriter::new(file);

  let _ = CsvExporter::export(&scene, &mut writer);

  println!("Converted scene {}.splat", FILENAME);

  Ok(())
}

#[test]
fn convert_csv_to_ply() -> Result<(), Box<dyn std::error::Error>> {
  const FILENAME: &str = "baby_yoda";

  let file = File::open(format!("./test_data/{}.csv", FILENAME))?;
  let mut reader = BufReader::new(file);

  let scene = CsvImporter::import(&mut reader)?;

  let file: File = File::create(format!("./test_data/{}_converted.ply", FILENAME))?;
  let mut writer = BufWriter::new(file);

  let _ = PlyBinaryExporter::export(&scene, &mut writer);

  println!("Converted scene {}.csv", FILENAME);

  Ok(())
}
