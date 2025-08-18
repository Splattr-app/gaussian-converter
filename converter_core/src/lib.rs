pub mod error;
pub mod formats;
pub mod helpers;

pub use error::ConversionError;

#[derive(Debug, Clone)]
pub struct GaussianSplat {
  pub position: [f32; 3], // An array of 3 elements, each of type f32 (32-bit floating point), X Y Z
  pub normal: [f32; 3],
  pub spherical_harmonics_dc: [f32; 3],
  pub spherical_harmonics_rest: Vec<f32>,
  pub opacity: f32,
  pub scale: [f32; 3],
  pub rotation: [f32; 4], // Quaternion (w, x, y, z)
}

#[derive(Debug, Clone)]
pub struct Scene {
  pub splats: Vec<GaussianSplat>,
}

/// A trait for any object that can read a byte stream and produce a `Scene`.
/// The `Read` trait is used to support streaming from files, network sockets, etc.
pub trait Importer {
  fn import(reader: &mut impl std::io::Read) -> Result<Scene, ConversionError>;
}

/// A trait for any object that can write a `Scene` to a byte stream.
/// The `Write` trait is used to support streaming to files, etc.
pub trait Exporter {
  fn export(scene: &Scene, writer: &mut impl std::io::Write) -> Result<(), ConversionError>;
}
