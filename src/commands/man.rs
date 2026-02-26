use anyhow::{Context, Result, bail};
use clap::CommandFactory;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::cli::Cli;

pub fn run(output: Option<PathBuf>, install: bool) -> Result<()> {
    if install && output.is_some() {
        bail!("--install cannot be used with --output");
    }

    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);

    let mut buffer = Vec::new();
    man.render(&mut buffer)
        .context("Failed to render man page")?;

    if install {
        let path = install_path()?;
        write_file(&path, &buffer)?;
        println!("{}", path.display());
        return Ok(());
    }

    if let Some(path) = output {
        write_file(&path, &buffer)?;
        return Ok(());
    }

    io::stdout()
        .write_all(&buffer)
        .context("Failed to write man page to stdout")?;

    Ok(())
}

fn install_path() -> Result<PathBuf> {
    if let Some(xdg_data_home) = std::env::var_os("XDG_DATA_HOME") {
        return Ok(PathBuf::from(xdg_data_home).join("man/man1/git-dom.1"));
    }

    let home = std::env::var_os("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".local/share/man/man1/git-dom.1"))
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    fs::write(path, bytes).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}
