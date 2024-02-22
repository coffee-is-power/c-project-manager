use std::{io::ErrorKind, path::PathBuf};

use serde::{Deserialize, Serialize};
pub mod errors;
fn default_src_folder() -> PathBuf {
    "src".into()
}
fn default_include_folder() -> PathBuf {
    "include".into()
}
#[derive(Deserialize, Serialize, Default, PartialEq, Eq)]
pub enum PackageKind {
    #[default]
    #[serde(rename = "exe")]
    Executable,
    #[serde(rename = "staticlib")]
    StaticLibrary,
    #[serde(rename = "dynlib")]
    DynamicLibrary,
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
    /// The folder that contains all the public header files
    ///
    /// This folder will be added to the include folders
    /// When compiling this package, and dependent packages
    #[serde(default = "default_include_folder")]
    pub include_folder: PathBuf,
    /// Adds additional flags for the compiler when compiling this package
    #[serde(default)]
    pub additional_compiler_flags: Vec<String>,
    /// Adds additional flags for the compiler when linking the source files
    /// of this package
    #[serde(default)]
    pub additional_linker_flags: Vec<String>,
    /// Links with math library when set to `true`
    #[serde(default)]
    pub enable_math_library: bool,
    /// Links with pthread library when set to `true`
    #[serde(default)]
    pub enable_pthread_library: bool,
    /// Disables linking with the std library when set to `true`
    #[serde(default)]
    pub disable_std_library: bool,

    /// Sets the output artifact of this package (e.g. lib or exe)
    #[serde(default)]
    pub kind: PackageKind,
}

#[derive(Deserialize, Serialize)]
pub struct Workspace {
    /// Paths to the children packages of this workspace
    pub members: Vec<PathBuf>,
}
#[derive(Deserialize, Serialize)]
pub struct Manifest {
    pub workspace: Option<Workspace>,
    pub package: Option<Package>,
}

impl Manifest {
    pub fn init_manifest(package_name: String) -> String {
        format!(
            r#"[package]
name = "{package_name}"
version = "0.1.0"
# src_folder = "src"
# include_folder = "include"
# additional_compiler_flags = [...]
# additional_linker_flags = [...]
# enable_pthread_library = false
# enable_math_library = false
# disable_std_library = false
# kind = "exe" or "lib"
"#
        )
    }
    pub fn load_manifest_from_file_path(
        file_path: impl Into<PathBuf>,
    ) -> Result<Self, errors::ManifestLoadError> {
        let manifest_string_content;
        match std::fs::read_to_string(file_path.into()) {
            Ok(content) => manifest_string_content = content,
            Err(e) => {
                return match e.kind() {
                    ErrorKind::NotFound => Err(errors::ManifestLoadError::NotFound),
                    _ => Err(errors::ManifestLoadError::IOError(e)),
                }
            }
        };
        match toml::from_str(&manifest_string_content) {
            Ok(result) => Ok(result),
            Err(e) => Err(errors::ManifestLoadError::Invalid(e)),
        }
    }
    pub fn load_manifest_from_project_path(
        project_path: impl Into<PathBuf>,
    ) -> Result<Self, errors::ManifestLoadError> {
        let mut manifest_path = project_path.into();
        manifest_path.push("cpm.toml");
        Self::load_manifest_from_file_path(manifest_path)
    }
}
