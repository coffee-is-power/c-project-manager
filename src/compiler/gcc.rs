use crate::manifest::PackageKind;

use super::Compiler;
use std::{path::PathBuf, process::Command};
pub struct GCC;

impl Compiler for GCC {
    fn compile_command(
        &self,
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
        command.arg(format!("-I{}", package_info.include_folder.display()));
        command
    }

    fn link_command(
        &self,
        object_files: Vec<PathBuf>,
        output_path: PathBuf,
        package_info: &crate::manifest::Package,
    ) -> Command {
        let mut command = Command::new("gcc");
        command
            .args(&object_files)
            .args(package_info.additional_linker_flags.as_slice());
        if !package_info.disable_std_library {
            command.arg("-lc");
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
