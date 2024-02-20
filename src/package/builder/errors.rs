use std::{path::PathBuf, process::ExitStatus};

use crate::manifest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreatePackageCompilerError {
    #[error("invalid package manifest: {0}")]
    InvalidPackageManifest(manifest::errors::ManifestLoadError),
    #[error("invalid workspace manifest: {0}")]
    InvalidWorkspaceManifest(manifest::errors::ManifestLoadError),
    #[error("`package` section not found")]
    PackageNotFound,
    #[error("`workspace` section not found")]
    WorkspaceNotFound,
}

#[derive(Error, Debug)]
#[error("failed to compile file {src_file_path:?} (exit code {exit_code:?})")]
pub struct CompileFileError {
    pub stderr: String,
    pub exit_code: ExitStatus,
    pub src_file_path: PathBuf,
    pub object_file_path: PathBuf,
}

#[derive(Error, Debug)]
pub enum BuildPackageError {
    #[error("no files to compile (no source files found)")]
    NoFilesToCompile,
    #[error("compilation errors occurred: {0:#?}")]
    CompilationError(Vec<CompileFileError>),
    #[error("a linking error occurred:")]
    LinkingError {
        stderr: String,
        output_file_path: PathBuf,
        exit_code: ExitStatus,
    },
    #[error("Io error: {0}")]
    IOError(#[from] std::io::Error),
}
