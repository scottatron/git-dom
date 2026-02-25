use anyhow::{Context, Result, bail};
use colored::Colorize;
use std::process::Command;

use git2::Repository;

pub fn run(name: String) -> Result<()> {
    let repo = Repository::open_from_env().context("Not in a git repository")?;
    let workdir = repo
        .workdir()
        .context("Bare repositories are not supported")?;

    // Verify the submodule exists
    let submodules = repo.submodules()?;
    let sm = submodules
        .iter()
        .find(|s| s.name().unwrap_or("") == name || s.path().to_str().unwrap_or("") == name);

    let sm = match sm {
        Some(s) => s,
        None => bail!("Submodule '{}' not found", name),
    };

    let sm_path = sm.path().to_str().unwrap_or("").to_string();
    let sm_name = sm.name().unwrap_or("").to_string();

    println!("{} {}", "Removing submodule".bold(), sm_name.red());

    // 1. Deinit the submodule
    let output = Command::new("git")
        .args(["submodule", "deinit", "-f", "--", &sm_path])
        .current_dir(workdir)
        .output()
        .context("Failed to deinit submodule")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git submodule deinit failed: {}", stderr);
    }

    // 2. Remove from .git/modules
    let modules_path = workdir.join(".git").join("modules").join(&sm_path);
    if modules_path.exists() {
        std::fs::remove_dir_all(&modules_path)
            .with_context(|| format!("Failed to remove {}", modules_path.display()))?;
    }

    // 3. Remove the submodule entry and worktree
    let output = Command::new("git")
        .args(["rm", "-f", &sm_path])
        .current_dir(workdir)
        .output()
        .context("Failed to git rm submodule")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git rm failed: {}", stderr);
    }

    println!("{} {} removed cleanly.", "✓".green().bold(), sm_name.bold());

    Ok(())
}
