#[cfg(os = "windows")]
pub const EXECUTABLE_EXTENSION: &str = "exe";

#[cfg(not(os = "windows"))]
pub const EXECUTABLE_EXTENSION: &str = "";

#[cfg(os = "windows")]
pub const STATIC_LIB_EXTENSION: &str = "lib";

#[cfg(not(os = "windows"))]
pub const STATIC_LIB_EXTENSION: &str = "a";

#[cfg(os = "windows")]
pub const DYN_LIB_EXTENSION: &str = "dll";

#[cfg(not(os = "windows"))]
pub const DYN_LIB_EXTENSION: &str = "so";

pub const MANIFEST_FILE_NAME: &str = "cpm.toml";
