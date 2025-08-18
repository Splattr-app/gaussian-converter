use std::{fs::File, io::BufReader};

#[cfg(test)]
use converter_core::Importer;
use converter_core::formats::splat::SplatImporter;

#[test]
fn splat_importer_success() {
  let file = File::open("./baby_yoda.splat").unwrap();
  let mut reader = BufReader::new(file);

  let result = SplatImporter::import(&mut reader);

  assert!(
    result.is_ok(),
    "SplatImporter should successfully parse valid SPLAT data"
  );
}
