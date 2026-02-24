use anyhow::Result;
use clap::CommandFactory;
use clap_complete::generate;

use crate::cli::Cli;

pub fn run(shell: clap_complete::Shell) -> Result<()> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "git-dom", &mut std::io::stdout());
    Ok(())
}
