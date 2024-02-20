use crate::compiler::Compiler;
use walkdir::WalkDir;

use crate::{
    filenames::{DYN_LIB_EXTENSION, EXECUTABLE_EXTENSION, STATIC_LIB_EXTENSION},
    manifest::{self, Manifest, PackageKind},
};
use std::{collections::HashSet, path::PathBuf, process::Stdio};
pub mod errors;
use errors::*;
/// Reads the manifest of the package and workspace and compiles a package
pub struct PackageBuilder {
    package_path: PathBuf,
    package_manifest: manifest::Manifest,
    workspace_path: PathBuf,
    workspace_info: Option<manifest::Workspace>,
}

impl PackageBuilder {
    /// Creates a new package compiler
    pub fn new(
        package_path: impl Into<PathBuf>,
        workspace_path: impl Into<PathBuf>,
    ) -> Result<Self, errors::CreatePackageCompilerError> {
        let package_path = package_path.into();
        let workspace_path = workspace_path.into();
        let package_manifest = Manifest::load_manifest_from_project_path(&package_path)
            .map_err(errors::CreatePackageCompilerError::InvalidPackageManifest)?;
        if package_manifest.package.is_none() {
            return Err(errors::CreatePackageCompilerError::PackageNotFound);
        }
        let workspace_manifest = Manifest::load_manifest_from_project_path(&workspace_path)
            .map_err(errors::CreatePackageCompilerError::InvalidWorkspaceManifest)?;
        Ok(Self {
            package_path,
            workspace_path,
            package_manifest,
            workspace_info: workspace_manifest.workspace,
        })
    }

    /// The path of the workspace. Will be the same as package_path if `workspace_info` is `None`
    ///
    /// Compilation outputs will be placed at `$workspace_path/target`
    pub fn workspace_path(&self) -> PathBuf {
        self.workspace_path.clone()
    }

    /// The workspace section of the manifest file at `$workspace_path/cpm.toml`
    pub fn workspace(&self) -> Option<&manifest::Workspace> {
        self.workspace_info.as_ref()
    }

    /// Path of the package to compile
    pub fn package_path(&self) -> PathBuf {
        self.package_path.clone()
    }

    /// Package info of the manifest file at `$package_path/cpm.toml`
    pub fn package(&self) -> &manifest::Package {
        self.package_manifest.package.as_ref().expect(
            "package manifest must have package info because it was checked in the constructor",
        )
    }

    /// The manifest file at `$package_path/cpm.toml`
    pub fn package_manifest(&self) -> &Manifest {
        &self.package_manifest
    }

    /// Generates the output folder path for this package.
    ///
    /// It does not create the folder,
    /// use `create_output_folder` to create the necessary folders
    pub fn output_folder_path(&self) -> PathBuf {
        let mut output_path = self.workspace_path.clone();
        output_path.push("target");
        match self.package().kind {
            PackageKind::Executable => output_path.push("executables"),
            PackageKind::StaticLibrary => output_path.push("staticlibs"),
            PackageKind::DynamicLibrary => output_path.push("dynlibs"),
        }
        output_path.push(format!(
            "{}-{:?}",
            self.package().name,
            self.package().version
        ));
        output_path
    }

    /// Creates the output folder for this package and returns the path to it
    pub fn create_output_folder(&self) -> std::io::Result<PathBuf> {
        let output_folder_path = self.output_folder_path();
        std::fs::create_dir_all(&output_folder_path)?;
        Ok(output_folder_path)
    }

    /// Generates the output path for this package.
    ///
    /// It does not create the folder,
    /// use `create_output_folder` to create the necessary folders
    pub fn output_path(&self) -> PathBuf {
        let mut output_path = self.output_folder_path();
        output_path.push(format!(
            "{}-{:?}",
            self.package().name,
            self.package().version
        ));
        match self.package().kind {
            PackageKind::Executable => output_path.set_extension(EXECUTABLE_EXTENSION),
            PackageKind::StaticLibrary => output_path.set_extension(STATIC_LIB_EXTENSION),
            PackageKind::DynamicLibrary => output_path.set_extension(DYN_LIB_EXTENSION),
        };
        output_path
    }
    /// Gets all the paths to the source files in the source folder recursively
    /// using walkdir library.
    pub fn src_files(&self) -> impl std::iter::Iterator<Item = PathBuf> {
        let mut package_src_folder_path = self.package_path();
        package_src_folder_path.push(&self.package().src_folder);

        let src_walkdir = WalkDir::new(&self.package().src_folder)
            .contents_first(true)
            .into_iter();

        src_walkdir
            .filter_entry(|e| e.path().extension().is_some_and(|ext| ext == "c"))
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
    }

    /// Returns all the files the output depends on
    /// Includes manifest files, source files and other libraries
    pub fn inputs(&self) -> HashSet<PathBuf> {
        let mut inputs = HashSet::<PathBuf>::new();
        let mut package_manifest_path = self.package_path();
        package_manifest_path.push("cpm.toml");
        inputs.insert(package_manifest_path);

        inputs.extend(self.src_files());
        inputs
    }

    /// Checks if the package needs recompilation
    ///
    /// This only checks the inputs with the output path
    pub fn needs_recompilation(&self) -> bool {
        let inputs = self.inputs();
        let output = self.output_path();
        return inputs
            .iter()
            .any(|input| file_needs_rebuild(input, &output));
    }

    fn absolute_source_path_to_relative_path(&self, source_file: impl Into<PathBuf>) -> PathBuf {
        let source_file = source_file.into();
        if source_file.is_absolute() {
            source_file
                .strip_prefix(&self.package().src_folder)
                .expect("all source files must be inside the source folder")
                .into()
        } else {
            source_file.into()
        }
    }

    pub fn object_file_folder_for_source_file(&self, source_file: impl Into<PathBuf>) -> PathBuf {
        let source_file: PathBuf = source_file.into();
        let path_from_source_folder = self.absolute_source_path_to_relative_path(source_file);
        let mut objects_folder_path = PathBuf::from("target/objects");
        objects_folder_path.push(format!(
            "{}-{:?}",
            self.package().name,
            self.package().version
        ));
        objects_folder_path.push(path_from_source_folder);
        objects_folder_path
    }
    pub fn object_file_for_source_file(&self, source_file: impl Into<PathBuf>) -> PathBuf {
        let source_file = source_file.into();
        let mut object_file_path: PathBuf = self.object_file_folder_for_source_file(&source_file);
        object_file_path.push(
            source_file
                .file_name()
                .expect("source file must have a file name"),
        );
        object_file_path.set_extension("o");
        object_file_path
    }
    /// Compiles the package
    pub fn compile(&self, compiler: &dyn Compiler) -> Result<(), errors::BuildPackageError> {
        if !self.needs_recompilation() {
            return Ok(());
        }
        let src_files = self.src_files();
        let mut compilation_errors = vec![];
        let mut object_files = vec![];
        for src in src_files {
            let object_folder_path = self.object_file_folder_for_source_file(&src);
            create_parent_folder(object_folder_path)?;
            let object_file_path = self.object_file_folder_for_source_file(&src);
            if file_needs_rebuild(&src, &object_file_path) {
                let output = compiler
                    .compile_command(src.clone(), object_file_path.clone(), self.package())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()?;
                if !output.status.success() {
                    compilation_errors.push(CompileFileError {
                        stderr: String::from_utf8(output.stderr)
                            .expect("compiler command must output valid UTF-8 in stderr"),
                        exit_code: output.status,
                        src_file_path: src,
                        object_file_path,
                    });
                } else {
                    object_files.push(object_file_path);
                }
            }
        }
        if !compilation_errors.is_empty() {
            return Err(errors::BuildPackageError::CompilationError(
                compilation_errors,
            ));
        }
        if object_files.is_empty() {
            return Err(errors::BuildPackageError::NoFilesToCompile);
        }
        self.create_output_folder()?;
        let package_output_path = self.output_path();
        let link_command_output = compiler
            .link_command(object_files, package_output_path.clone(), self.package())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
        if !link_command_output.status.success() {
            return Err(errors::BuildPackageError::LinkingError {
                stderr: String::from_utf8(link_command_output.stderr)
                    .expect("compiler command must output valid UTF-8 in stderr"),
                output_file_path: package_output_path,
                exit_code: link_command_output.status,
            });
        }
        Ok(())
    }
}
/// Checks if a file needs to be rebuilt based on the modified at timestamps.
///
/// If the input file was modified after the output path was modified, `true` is returned, `false` otherwise
///
/// # Errors
/// In case of an error to open the input or the output file, this function always returns true
pub fn file_needs_rebuild(input_path: impl Into<PathBuf>, output_path: impl Into<PathBuf>) -> bool {
    let source_path = input_path.into();
    let object_path = output_path.into();
    let Ok(object_path_metadata) = std::fs::metadata(object_path) else {
        return true;
    };
    let Ok(source_path_metadata) = std::fs::metadata(source_path) else {
        return true;
    };
    let Ok(source_modified) = source_path_metadata.modified() else {
        return true;
    };
    let Ok(object_modified) = object_path_metadata.modified() else {
        return true;
    };
    object_modified < source_modified
}

pub fn create_parent_folder(path: impl Into<PathBuf>) -> std::io::Result<()> {
    let path = path.into();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
