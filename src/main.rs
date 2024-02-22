mod commands;
mod compiler;
mod filenames;
mod manifest;
mod package;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug, Clone)]
pub enum CPMOperation {
    #[command(about = "Create a new executable package")]
    Init { path: PathBuf },
    #[command(about = "Build a package")]
    Build,
    #[command(about = "Build and run a executable package")]
    Run {
        #[arg(short, long)]
        package: Option<String>,
    },
}
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CPMArguments {
    #[command(subcommand)]
    op: CPMOperation,
}

fn main() {
    let args = CPMArguments::parse();
    match args.op {
        CPMOperation::Init { path } => commands::init(path),
        CPMOperation::Build => {
            let _ = commands::build_project();
        }
        CPMOperation::Run { package } => commands::run_project(package),
    }
}
