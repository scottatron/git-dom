use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;

mod cli;
mod commands;
mod config;
mod submodule;

fn main() -> Result<()> {
    // Runtime shell completion — handles COMPLETE=zsh/bash/fish requests
    CompleteEnv::with_factory(cli::Cli::command).complete();

    let args = cli::Cli::parse();

    // Respect NO_COLOR (https://no-color.org/)
    if std::env::var("NO_COLOR").is_ok() || args.no_colour {
        colored::control::set_override(false);
    }

    match args.command {
        cli::Command::Ls { name } => commands::ls::run(name),
        cli::Command::Status { name } => commands::status::run(name),
        cli::Command::Clone { url, no_commit } => commands::clone::run(url, no_commit),
        cli::Command::Pull { name, commit } => commands::pull::run(name, commit),
        cli::Command::Rm { name } => commands::rm::run(name),
        cli::Command::Diff { name, full } => commands::diff::run(name, full),
        cli::Command::Foreach { command, parallel } => {
            commands::foreach::run(command, parallel)
        }
        cli::Command::Completions { shell } => commands::completions::run(shell),
    }
}
