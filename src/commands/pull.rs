use anyhow::{Context, Result};
use colored::Colorize;
use git2::Repository;
use std::process::Command;

use crate::config::{CommitMode, Config};
use crate::submodule;

pub fn run(name: Option<String>, commit_override: Option<CommitMode>) -> Result<()> {
    let repo = Repository::open_from_env().context("Not in a git repository")?;
    let config = Config::load(&repo)?;
    let commit_mode = commit_override.unwrap_or(config.commit_mode);

    let workdir = repo
        .workdir()
        .context("Bare repositories are not supported")?;

    println!("{}", "Fetching and updating submodules...".bold());

    // Fetch all submodules (or specific one)
    let mut fetch_args = vec![
        "submodule".to_string(),
        "update".to_string(),
        "--remote".to_string(),
        "--merge".to_string(),
    ];

    if let Some(ref n) = name {
        fetch_args.push("--".to_string());
        fetch_args.push(n.clone());
    }

    let output = Command::new("git")
        .args(&fetch_args)
        .current_dir(workdir)
        .output()
        .context("Failed to run git submodule update")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{} {}", "✗".red().bold(), stderr);
        return Ok(());
    }

    // Show what changed
    let submodules = submodule::discover(&repo, name.as_deref())?;
    let mut updated = Vec::new();

    for sm in &submodules {
        // Check if the submodule ref changed in the parent
        let output = Command::new("git")
            .args(["diff", "--name-only", "--", sm.path.to_str().unwrap_or("")])
            .current_dir(workdir)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            updated.push(&sm.name);
            println!("  {} {} updated", "↑".green(), sm.name.bold());
        }
    }

    if updated.is_empty() {
        println!("{}", "All submodules already up to date.".green());
        return Ok(());
    }

    // Handle commit
    match commit_mode {
        CommitMode::Auto => {
            let msg = format!(
                "Update submodule(s): {}",
                updated
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            Command::new("git")
                .args(["add", "-A"])
                .current_dir(workdir)
                .output()?;

            Command::new("git")
                .args(["commit", "-m", &msg])
                .current_dir(workdir)
                .output()?;

            println!(
                "{} {}",
                "✓".green().bold(),
                format!("Committed: {}", msg).dimmed()
            );
        }
        CommitMode::Stage => {
            Command::new("git")
                .args(["add", "-A"])
                .current_dir(workdir)
                .output()?;

            println!(
                "{} {}",
                "✓".green().bold(),
                "Changes staged. Run git commit when ready.".dimmed()
            );
        }
        CommitMode::Prompt => {
            // Show what changed
            let diff = Command::new("git")
                .args(["diff", "--cached", "--stat"])
                .current_dir(workdir)
                .output()?;
            println!("{}", String::from_utf8_lossy(&diff.stdout));

            print!("Commit these changes? [Y/n] ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().is_empty() || input.trim().to_lowercase() == "y" {
                let msg = format!(
                    "Update submodule(s): {}",
                    updated
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                Command::new("git")
                    .args(["add", "-A"])
                    .current_dir(workdir)
                    .output()?;

                Command::new("git")
                    .args(["commit", "-m", &msg])
                    .current_dir(workdir)
                    .output()?;

                println!("{} {}", "✓".green().bold(), "Committed.".dimmed());
            } else {
                println!("{}", "Skipped commit.".dimmed());
            }
        }
    }

    Ok(())
}
