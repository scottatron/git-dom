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

    for (i, sm) in submodules.iter().enumerate() {
        if i > 0 {
            println!();
        }

        let branch_display = sm.branch.as_deref().unwrap_or("(detached)");

        println!("{}", sm.name.bold());
        println!("  {} {}", "path:".dimmed(), sm.path.display());
        println!("  {} {}", "url: ".dimmed(), sm.url);
        println!(
            "  {} {}",
            "on:  ".dimmed(),
            branch_display.cyan()
        );

        if let Some(ref commit) = sm.head_commit {
            let msg = sm.head_message.as_deref().unwrap_or("");
            println!("  {} {} {}", "head:".dimmed(), commit.yellow(), msg);
        }

        // Ahead/behind
        match (sm.ahead, sm.behind) {
            (0, 0) => println!(
                "  {}",
                "Your branch is up to date with upstream.".green()
            ),
            (a, 0) => println!(
                "  {} ahead of upstream by {} commit(s)",
                "↑".green(),
                a.to_string().bold()
            ),
            (0, b) => println!(
                "  {} behind upstream by {} commit(s)",
                "↓".red(),
                b.to_string().bold()
            ),
            (a, b) => println!(
                "  {} ahead by {}, behind by {}",
                "↕".yellow(),
                a.to_string().bold(),
                b.to_string().bold()
            ),
        }

        // Working tree status
        if sm.staged == 0 && sm.modified == 0 && sm.untracked == 0 {
            println!("  {}", "nothing to commit, working tree clean".green());
        } else {
            if sm.staged > 0 {
                println!(
                    "  {} {} staged change(s)",
                    "●".green(),
                    sm.staged
                );
            }
            if sm.modified > 0 {
                println!(
                    "  {} {} modified file(s)",
                    "✱".red(),
                    sm.modified
                );
            }
            if sm.untracked > 0 {
                println!(
                    "  {} {} untracked file(s)",
                    "?".dimmed(),
                    sm.untracked
                );
            }
        }

        // Pending changes in parent repo
        if sm.parent_changed {
            println!(
                "  {} {}",
                "⬆".yellow(),
                "submodule ref changed in parent (uncommitted)".yellow()
            );
        }
    }

    Ok(())
}
