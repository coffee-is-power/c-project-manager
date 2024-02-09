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
}
impl Package {
    pub fn binary_path(&self) -> Result<PathBuf, String> {
        let mut binary_output_path = PathBuf::new();
        binary_output_path.push("target");
        binary_output_path.push("binaries");
        std::fs::create_dir_all(&binary_output_path)
            .map_err(|err| format!("failed to create binaries folder: {err}"))?;
        binary_output_path.push(format!("{}-{}", self.name, self.version.to_string()));
        Ok(binary_output_path)
    }
}
#[derive(Deserialize, Serialize)]
pub struct Manifest {
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
"#
        )
    }
}
