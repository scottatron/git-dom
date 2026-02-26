use clap::{Parser, Subcommand};
use clap_complete::Shell;
use clap_complete::engine::ArgValueCompleter;
use std::path::PathBuf;

use crate::config::CommitMode;

/// Return submodule names for shell completion by reading .gitmodules.
fn complete_submodule_name(current: &std::ffi::OsStr) -> Vec<clap_complete::CompletionCandidate> {
    let current = current.to_string_lossy();
    let Ok(repo) = git2::Repository::open_from_env() else {
        return Vec::new();
    };
    let Ok(submodules) = repo.submodules() else {
        return Vec::new();
    };
    submodules
        .iter()
        .filter_map(|sm| {
            let name = sm.name()?.to_string();
            if name.starts_with(current.as_ref()) {
                Some(clap_complete::CompletionCandidate::new(name))
            } else {
                None
            }
        })
        .collect()
}

#[derive(Parser)]
#[command(
    name = "git-dom",
    version,
    about = "A an opinionated & friendlier UX for git submodules"
)]
pub struct Cli {
    /// Disable colour output
    #[arg(long = "no-colour", global = true)]
    pub no_colour: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// List all submodules
    Ls {
        /// Target a specific submodule by name
        #[arg(add = ArgValueCompleter::new(complete_submodule_name))]
        name: Option<String>,
    },

    /// Show rich status for submodules
    Status {
        /// Target a specific submodule by name
        #[arg(add = ArgValueCompleter::new(complete_submodule_name))]
        name: Option<String>,
    },

    /// Add a submodule with Go-style path convention
    Clone {
        /// URL, host/user/repo, or owner/repo (defaults to github.com)
        url: String,

        /// Skip commit prompt even when running interactively
        #[arg(long)]
        no_commit: bool,
    },

    /// Fetch and update submodules from upstream
    Pull {
        /// Target a specific submodule by name
        #[arg(add = ArgValueCompleter::new(complete_submodule_name))]
        name: Option<String>,

        /// Commit mode: auto, stage, or prompt
        #[arg(long, value_enum)]
        commit: Option<CommitMode>,
    },

    /// Remove a submodule cleanly
    Rm {
        /// Submodule name or path
        #[arg(add = ArgValueCompleter::new(complete_submodule_name))]
        name: String,
    },

    /// Show changes across submodules
    Diff {
        /// Target a specific submodule by name
        #[arg(add = ArgValueCompleter::new(complete_submodule_name))]
        name: Option<String>,

        /// Show full per-submodule diffs
        #[arg(long)]
        full: bool,
    },

    /// Run a command in each submodule
    Foreach {
        /// The command to run
        command: Vec<String>,

        /// Run in parallel
        #[arg(long)]
        parallel: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Generate or install a man page for git-dom
    Man {
        /// Write man page to this path (defaults to stdout)
        #[arg(long, short = 'o', value_name = "PATH", conflicts_with = "install")]
        output: Option<PathBuf>,

        /// Install to $XDG_DATA_HOME/man/man1/git-dom.1 (or ~/.local/share/man/man1)
        #[arg(long)]
        install: bool,
    },
}
