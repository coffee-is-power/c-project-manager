use crate::manifest::PackageKind;

use super::Compiler;
use std::{path::PathBuf, process::Command};
pub struct GCC;

impl Compiler for GCC {
    fn compile_command(
        &self,
        package_path: PathBuf,
        source_path: PathBuf,
        output_path: PathBuf,
        package_info: &crate::manifest::Package,
    ) -> Command {
        let mut command = Command::new("gcc");
        if package_info.disable_std_library {
            command.arg("-no-std");
        }
        command.arg(source_path).args(["-c", "-o"]).arg(output_path);
        command.args(&package_info.additional_compiler_flags);
        let mut include_folder_absolute_path = package_path.clone();
        include_folder_absolute_path.push(&package_info.include_folder);
        command.arg(format!("-I{}", include_folder_absolute_path.display()));
        command
    }

    fn link_command(
        &self,
        _package_path: PathBuf,
        object_files: Vec<PathBuf>,
        output_path: PathBuf,
        package_info: &crate::manifest::Package,
    ) -> Command {
        let mut command = Command::new("gcc");
        command
            .args(&object_files)
            .args(package_info.additional_linker_flags.as_slice());
        if package_info.disable_std_library {
            command.arg("-no-std");
        }
        if package_info.enable_math_library {
            command.arg("-lm");
        }
        if package_info.enable_pthread_library {
            command.arg("-lpthread");
        }
        match package_info.kind {
            PackageKind::DynamicLibrary => {
                command.arg("-shared");
            }
            PackageKind::StaticLibrary => {
                command.arg("-static");
            }
            PackageKind::Executable => {}
        }
        command.arg("-o").arg(&output_path);
        command
    }
}
