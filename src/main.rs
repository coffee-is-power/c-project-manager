mod commands;
mod manifest;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug, Clone)]
pub enum CPMOperation {
    Init { path: PathBuf },
    Build,
    Run,
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
        CPMOperation::Run => commands::run_project(),
    }
}
