use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod config;
mod submodule;
mod version;

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    // Respect NO_COLOR (https://no-color.org/)
    if std::env::var("NO_COLOR").is_ok() || args.no_colour {
        colored::control::set_override(false);
    }

    match args.command {
        cli::Command::Ls { name } => commands::ls::run(name),
        cli::Command::Status { name } => commands::status::run(name),
        cli::Command::Clone {
            url,
            branch,
            no_commit,
        } => commands::clone::run(url, branch, no_commit),
        cli::Command::Pull { name, commit } => commands::pull::run(name, commit),
        cli::Command::Rm { name, no_commit } => commands::rm::run(name, no_commit),
        cli::Command::Diff { name, full } => commands::diff::run(name, full),
        cli::Command::Foreach { command, parallel } => commands::foreach::run(command, parallel),
        cli::Command::Completions { shell } => commands::completions::run(shell),
        cli::Command::Man { output, install } => commands::man::run(output, install),
    }
}
