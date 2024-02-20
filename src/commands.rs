use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{filenames::MANIFEST_FILE_NAME, package::compiler::PackageCompiler};
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
                print_error(format!("\x1b[1;31merror:\x1b[0m {err}"));
                std::process::exit(1)
            }
        }
    };

    (result = $result:expr, message = $message:expr) => {
        match $result {
            Ok(it) => it,
            Err(err) => {
                print_error(format!("\x1b[1;31merror:\x1b[0m {}: {err}", $message));
                std::process::exit(1)
            }
        }
    };
    (result = ?$result:expr, message = $message:expr) => {
        match $result {
            Ok(it) => it,
            Err(err) => {
                print_error(format!("{}: {err:?}", $message));
                std::process::exit(1)
            }
        }
    };
    (result = #?$result:expr, message = $message:expr) => {
        match $result {
            Ok(it) => it,
            Err(err) => {
                print_error(format!("{}: {err:#?}", $message));
                std::process::exit(1)
            }
        }
    };
    (option = $opt:expr, message = $message:expr) => {
        match $opt {
            Some(it) => it,
            None => {
                print_error($message);
                std::process::exit(1)
            }
        }
    };
}

fn print_error(message: impl Into<String>) {
    eprintln!("\x1b[1;31merror:\x1b[0m {}", message.into());
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
                &format!("the specified folder already contains a `{MANIFEST_FILE_NAME}` file"),
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
pub fn build_project() {
    let mut cmd = CPMArguments::command();
    if !PathBuf::from(MANIFEST_FILE_NAME).exists() {
        cmd.error(
            clap::error::ErrorKind::Io,
            &format!(
                "you're not currently on a CPM project (`{MANIFEST_FILE_NAME}` does not exist)"
            ),
        )
        .exit();
    }
    let manifest_string = handle_error!(
        result = std::fs::read_to_string("cpm.toml"),
        message = &format!("failed to read`{MANIFEST_FILE_NAME}`")
    );
    let manifest: Manifest = handle_error!(
        result = toml::from_str(&manifest_string),
        message = &format!("`{MANIFEST_FILE_NAME}` contains errors")
    );
    let cwd = handle_error!(
        result = std::env::current_dir(),
        message = "failed to access current working dir"
    );
    let mut packages_to_compile: Vec<PathBuf> = vec![];
    if manifest.package.is_some() {
        packages_to_compile.push(cwd.clone());
    }
    if let Some(workspace) = manifest.workspace.as_ref() {
        packages_to_compile.extend(workspace.members.iter().map(|member_path| {
            let mut absolute_path = cwd.clone();
            absolute_path.push(member_path);
            absolute_path
        }));
    }
    let workspace_path = cwd.clone();
    for package_path in packages_to_compile {
        let package_compiler =
            handle_error!(result = PackageCompiler::new(package_path, workspace_path.clone()));
        package_compiler.compile(todo!()).unwrap();
    }
    // if todo!() as i32 > 0 {
    //     print_error("stopping compilation due to {compilation_error_count} previous errors");
    //     std::process::exit(1);
    // }
    // println!("\x1b[1;32mFinished building package \x1b[0m ({})", todo!());
}
pub fn run_project() {
    todo!("Run project");
}
