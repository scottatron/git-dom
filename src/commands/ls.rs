use anyhow::{Context, Result};
use colored::Colorize;
use git2::Repository;

use crate::submodule;

pub fn run(name: Option<String>) -> Result<()> {
    let repo = Repository::open_from_env().context("Not in a git repository")?;
    let submodules = submodule::discover(&repo, name.as_deref())?;

    if submodules.is_empty() {
        println!("No submodules found.");
        return Ok(());
    }

    // Calculate column widths
    let max_name = submodules.iter().map(|s| s.name.len()).max().unwrap_or(0);

    for sm in &submodules {
        let branch = sm.branch.as_deref().unwrap_or("(none)");
        let commit = sm.head_commit.as_deref().unwrap_or("-------");

        let status = if sm.is_dirty {
            "dirty".red().to_string()
        } else {
            "clean".green().to_string()
        };

        println!(
            "{:<width$}  {}  {}  {}",
            sm.name.bold(),
            commit.dimmed(),
            branch.cyan(),
            status,
            width = max_name
        );
    }

    Ok(())
}
