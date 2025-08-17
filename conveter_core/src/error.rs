use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
  #[error("I/O error: {0}")]
  Io(#[from] std::io::Error),

  #[error("Failed to parse {format}: {message}")]
  ParseError { format: String, message: String },

  #[error("Failed to write {format}: {message}")]
  WriteError { format: String, message: String },

  #[error("Unsupported format")]
  UnsupportedFormat,

  #[error("An unknown error occurred")]
  Unknown,
}
