use anyhow::{Context, Result};
use colored::Colorize;
use git2::Repository;
use std::process::Command;

use crate::submodule;

pub fn run(name: Option<String>, full: bool) -> Result<()> {
    let repo = Repository::open_from_env().context("Not in a git repository")?;
    let workdir = repo
        .workdir()
        .context("Bare repositories are not supported")?;
    let submodules = submodule::discover(&repo, name.as_deref())?;

    if submodules.is_empty() {
        println!("No submodules found.");
        return Ok(());
    }

    let mut any_changes = false;

    for sm in &submodules {
        let sm_path = sm.path.to_str().unwrap_or("");

        if full {
            // Full diff within the submodule
            let output = Command::new("git")
                .args(["diff", "HEAD"])
                .current_dir(workdir.join(&sm.path))
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.trim().is_empty() {
                    any_changes = true;
                    println!("{}", sm.name.bold());
                    println!("{}", stdout);
                }
            }
        } else {
            // Summary: show submodule ref changes and dirty state
            let output = Command::new("git")
                .args(["diff", "--submodule=short", "--", sm_path])
                .current_dir(workdir)
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);

            if !stdout.trim().is_empty() || sm.is_dirty {
                any_changes = true;
                print!("{}", sm.name.bold());

                if sm.is_dirty {
                    print!(" {}", "(dirty)".red());
                }
                println!();

                if !stdout.trim().is_empty() {
                    println!("  {}", stdout.trim().dimmed());
                }

                if sm.staged > 0 {
                    println!("  {} staged", sm.staged.to_string().green());
                }
                if sm.modified > 0 {
                    println!("  {} modified", sm.modified.to_string().red());
                }
                if sm.untracked > 0 {
                    println!("  {} untracked", sm.untracked.to_string().dimmed());
                }
            }
        }
    }

    if !any_changes {
        println!("{}", "No changes across submodules.".green());
    }

    Ok(())
}
