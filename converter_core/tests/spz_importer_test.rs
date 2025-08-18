use std::{fs::File, io::BufReader};

#[cfg(test)]
use converter_core::Importer;
use converter_core::formats::spz::SpzImporter;

#[test]
fn spz_importer_success() {
  let file = File::open("./hornedlizard.spz").unwrap();
  let mut reader = BufReader::new(file);

  let result = SpzImporter::import(&mut reader);

  assert!(
    result.is_ok(),
    "Importer should successfully parse valid PLY data"
  );
}
