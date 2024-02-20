use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManifestLoadError {
    #[error("failed to read manifest file: not found")]
    NotFound,
    #[error("failed to read manifest file: IO Error: {0}")]
    IOError(std::io::Error),
    #[error("invalid manifest file")]
    Invalid(toml::de::Error),
}
