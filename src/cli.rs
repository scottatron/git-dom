use clap::{Parser, Subcommand};
use clap_complete::Shell;

use crate::config::CommitMode;

#[derive(Parser)]
#[command(name = "git-dom", version, about = "A friendlier UX for git submodules")]
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
        name: Option<String>,
    },

    /// Show rich status for submodules
    Status {
        /// Target a specific submodule by name
        name: Option<String>,
    },

    /// Add a submodule with Go-style path convention
    Clone {
        /// URL or host/user/repo path (e.g. github.com/user/repo)
        url: String,
    },

    /// Fetch and update submodules from upstream
    Pull {
        /// Target a specific submodule by name
        name: Option<String>,

        /// Commit mode: auto, stage, or prompt
        #[arg(long, value_enum)]
        commit: Option<CommitMode>,
    },

    /// Remove a submodule cleanly
    Rm {
        /// Submodule name or path
        name: String,
    },

    /// Show changes across submodules
    Diff {
        /// Target a specific submodule by name
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
}
