use ply_rs::ply;

/// Get a float property from a PLY element, returning 0.0 if not found or invalid
pub fn get_f32_property(vert: &ply::DefaultElement, key: &str) -> f32 {
  vert.get(key).map_or(0.0, |prop| property_to_f32(prop))
}

/// Convert a PLY property to f32, returning 0.0 for unsupported types
pub fn property_to_f32(prop: &ply::Property) -> f32 {
  match prop {
    ply::Property::Float(f) => *f,
    _ => 0.0, // fallback for unsupported property types
  }
}

/// Convert a PLY property to u32, returning 0 for unsupported types
pub fn property_to_u32(prop: &ply::Property) -> u32 {
  match prop {
    ply::Property::UInt(i) => *i,
    _ => 0, // Fallback
  }
}

/// Convert a PLY property to u8, returning 0 for unsupported types
pub fn property_to_u8(prop: &ply::Property) -> u8 {
  match prop {
    ply::Property::UChar(c) => *c,
    _ => 0, // Fallback
  }
}

/// Extract spherical harmonics coefficients from a PLY element
pub fn get_spherical_harmonics_res(vert: &ply::DefaultElement) -> Vec<f32> {
  let base_key = "f_rest_";
  let mut output: Vec<f32> = Vec::with_capacity(45); // Pre-allocate with capacity

  for i in 0..45 {
    let key = format!("{}{}", base_key, i);
    // Use the safe getter for each spherical harmonic coefficient
    output.push(get_f32_property(vert, &key));
  }

  output
}

/// Helper function to write a slice of f32 values as little-endian bytes
pub fn write_f32_slice<W: std::io::Write>(writer: &mut W, data: &[f32]) -> std::io::Result<()> {
  for &val in data {
    writer.write_all(&val.to_le_bytes())?;
  }
  Ok(())
}
