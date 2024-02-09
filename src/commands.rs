use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    manifest::{Manifest, Package},
    CPMArguments,
};
use clap::CommandFactory;
use walkdir::WalkDir;
/// Helper macro to report errors more easily
/// # Examples
/// ```no_run
/// assert_eq!(handle_error!(result = "9".parse::<i32>()), 9);
/// ```
macro_rules! handle_error {
    (result = $result:expr) => {
        match $result {
            Ok(it) => it,
            Err(err) => {
                eprintln!("\x1b[1;31merror:\x1b[0m {err}");
                std::process::exit(1)
            }
        }
    };

    (result = $result:expr, message = $message:expr) => {
        match $result {
            Ok(it) => it,
            Err(err) => {
                eprintln!("\x1b[1;31merror:\x1b[0m {}: {err}", $message);
                std::process::exit(1)
            }
        }
    };
    (result = ?$result:expr, message = $message:expr) => {
        match $result {
            Ok(it) => it,
            Err(err) => {
                eprintln!("\x1b[1;31merror:\x1b[0m {}: {err:?}", $message);
                std::process::exit(1)
            }
        }
    };
    (result = #?$result:expr, message = $message:expr) => {
        match $result {
            Ok(it) => it,
            Err(err) => {
                eprintln!("\x1b[1;31merror:\x1b[0m {}: {err:#?}", $message);
                std::process::exit(1)
            }
        }
    };
    (option = $opt:expr, message = $message:expr) => {
        match $opt {
            Some(it) => it,
            None => {
                eprintln!("\x1b[1;31merror:\x1b[0m {}", $message);
                std::process::exit(1)
            }
        }
    };
}
pub fn init(path: PathBuf) {
    handle_error!(
        result = std::fs::create_dir_all(&path),
        message = "failed to create project folder"
    );
    let path = handle_error!(
        result = path.canonicalize(),
        message = "Failed to resolve path into a absolute path"
    ); // the specified path doesn't contain a file name to use as the name of the project
    let package_name = handle_error!(
        option = path.file_name().map(|s| s.to_string_lossy().to_string()),
        message = "the (absolute) path doesn't have a file name to use as the project name"
    );
    if !package_name
        .chars()
        .all(|c| matches!(c, 'a'..='z' | '_' | '0'..='9'))
    {
        CPMArguments::command()
            .error(
                clap::error::ErrorKind::ValueValidation,
                "the project name must in snake_case (allowed chars: 'a'..'z' | '_' | '0'..'9')",
            )
            .exit();
    }
    let mut manifest_path = path.clone();
    manifest_path.push("cpm.toml");
    if manifest_path.exists() {
        CPMArguments::command()
            .error(
                clap::error::ErrorKind::ValueValidation,
                "the specified folder already contains a `cpm.toml` file",
            )
            .exit();
    }
    handle_error!(
        result = std::fs::write(manifest_path, Manifest::init_manifest(package_name)),
        message = "failed to create project folder"
    );
    let mut src_folder_path = path.clone();
    src_folder_path.push("src");
    if !src_folder_path.exists() {
        handle_error!(
            result = std::fs::create_dir_all(&src_folder_path),
            message = "failed to create source folder"
        );
        let mut main_c_path = src_folder_path.clone();
        main_c_path.push("main.c");
        handle_error!(
            result = std::fs::write(main_c_path, include_str!("hello_world.c")),
            message = "failed to create hello world program"
        );
    }
    let mut include_folder_path = path.clone();
    include_folder_path.push("include");
    handle_error!(
        result = std::fs::create_dir_all(include_folder_path),
        message = "failed to create include folder"
    );
}
pub fn object_file_path_of_source(
    package: &Package,
    source_path: PathBuf,
) -> std::io::Result<PathBuf> {
    let mut src_folder_path = std::env::current_dir()?;
    src_folder_path.push("src");
    let mut object_file_path = PathBuf::new();
    object_file_path.push("target");
    object_file_path.push(&package.name);
    object_file_path.push(package.version.to_string());
    object_file_path.push("objs");
    object_file_path.push(source_path.with_extension("o"));
    std::fs::create_dir_all(object_file_path.parent().unwrap())?;
    Ok(object_file_path)
}
pub fn needs_rebuild(source_path: &Path, object_path: &Path) -> bool {
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
pub fn build_c_file(source_path: PathBuf, package: &Package) -> Result<PathBuf, String> {
    let object_path = object_file_path_of_source(package, source_path.clone())
        .expect("failed to get object path of source file");
    if needs_rebuild(source_path.as_path(), object_path.as_path()) {
        let mut command = Command::new("gcc");
        if package.disable_std_library {
            command.arg("-no-std");
        }
        command
            .args(package.additional_compiler_flags.as_slice())
            .arg(format!("-I{}", package.include_folder.display()))
            .arg("-c")
            .arg("-o")
            .arg(&object_path)
            .arg(&source_path);
        let status = command
            .spawn()
            .map_err(|e| e.to_string())?
            .wait()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!(
                "failed to compile file {} of package {:?} (version {})",
                source_path.display(),
                package.name,
                package.version
            ));
        }
    }
    Ok(object_path)
}

pub fn link_package(package: &Package, object_files: HashSet<PathBuf>) -> Result<PathBuf, String> {
    let mut command = Command::new("gcc");
    command
        .args(&object_files)
        .args(package.additional_linker_flags.as_slice());
    if !package.disable_std_library {
        command.arg("-lc");
    }
    if package.enable_math_library {
        command.arg("-lm");
    }
    if package.enable_pthread_library {
        command.arg("-lpthread");
    }
    let binary_output_path = package.binary_path()?;
    command.arg("-o").arg(&binary_output_path);
    let status = command
        .spawn()
        .map_err(|e| e.to_string())?
        .wait()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!(
            "failed to link package {:?} (version {})",
            package.name, package.version
        ));
    }
    Ok(binary_output_path)
}
pub fn build_project() -> PathBuf {
    let mut cmd = CPMArguments::command();
    if !PathBuf::from("cpm.toml").exists() {
        cmd.error(
            clap::error::ErrorKind::Io,
            "you're not currently on a CPM project",
        )
        .exit();
    }
    let manifest_string = handle_error!(
        result = std::fs::read_to_string("cpm.toml"),
        message = "failed to read `cpm.toml`"
    );
    let manifest: Manifest = handle_error!(
        result = toml::from_str(&manifest_string),
        message = "`cpm.toml` contains errors"
    );
    let package: &Package = handle_error!(
        option = manifest.package.as_ref(),
        message = "no package to compile"
    );
    let src_walkdir = WalkDir::new(&package.src_folder)
        .contents_first(true)
        .into_iter();
    let mut compilation_error_count = 0u32;
    let obj_file_paths: HashSet<PathBuf> = src_walkdir
        .filter_entry(|e| e.path().extension().is_some_and(|ext| ext == "c"))
        .filter_map(|e| {
            let e = match e {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("\x1b[1;31merror:\x1b[0m failed to read file: {err}");
                    compilation_error_count += 1;
                    return None;
                }
            };
            match build_c_file(e.into_path(), &package) {
                Ok(obj) => Some(obj),
                Err(err) => {
                    eprintln!("\x1b[1;31merror:\x1b[0m {err}");
                    compilation_error_count += 1;
                    return None;
                }
            }
        })
        .collect();
    if compilation_error_count > 0 {
        eprintln!("\x1b[1;31merror:\x1b[0m stopping compilation due to {compilation_error_count} previous errors");
        std::process::exit(1);
    }
    if !obj_file_paths.is_empty() {
        let binary_path = handle_error!(result = link_package(&package, obj_file_paths));
        println!(
            "\x1b[1;32mFinished building package \x1b[0m ({})",
            binary_path.display()
        );
        binary_path
    } else {
        eprintln!("No files to compile");
        std::process::exit(1);
    }
}
pub fn run_project() {
    let binary_path = build_project();
    std::process::exit(handle_error!(
        option = handle_error!(
            result = handle_error!(result = Command::new(binary_path).spawn()).wait()
        )
        .code(),
        message = "No exit code? *megamind meme*"
    ));
}
