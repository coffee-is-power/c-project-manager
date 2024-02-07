use std::path::PathBuf;

use serde::{Deserialize, Serialize};
fn default_src_folder() -> PathBuf {
    "src".into()
}
fn default_include_folder() -> PathBuf {
    "include".into()
}
#[derive(Deserialize, Serialize)]
pub struct Package {
    /// This name of this package
    /// It must be in **snake_case**
    pub name: String,
    /// Semver version of the package
    pub version: semver::Version,
    /// The folder that contains all the source files
    /// to compile
    #[serde(default = "default_src_folder")]
    pub src_folder: PathBuf,
    /// The folder that contains all the source files
    /// to compile
    #[serde(default = "default_include_folder")]
    pub include_folder: PathBuf,
}

#[derive(Deserialize, Serialize)]
pub struct Manifest {
    pub package: Option<Package>,
}
