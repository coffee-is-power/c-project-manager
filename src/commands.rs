use std::path::PathBuf;

use clap::CommandFactory;

use crate::CPMArguments;

pub fn init(path: PathBuf) -> color_eyre::Result<()> {
    let mut cmd = CPMArguments::command();
    let path = path.canonicalize()?;
    let package_name = path
        .file_name()
        .ok_or_else(|| color_eyre::eyre::eyre!("Failed to figure out project name"))?
        .to_string_lossy()
        .to_string();
    if package_name
        .chars()
        .any(|c| !c.is_ascii() || c.is_whitespace() || c.is_uppercase())
    {
        cmd.error(clap::error::ErrorKind::InvalidValue, "The project name must in snake_case (ascii lower case characters with underlines instead of spaces)").exit();
    }

    Ok(())
}
