use anyhow::{Context, Result, bail};
use colored::Colorize;
use git2::Repository;
use std::io::Write;
use std::process::Command;

use crate::config::Config;

pub fn run(url: String, no_commit: bool) -> Result<()> {
    let repo = Repository::open_from_env().context("Not in a git repository")?;
    let config = Config::load(&repo)?;

    // Parse the URL to determine the submodule path
    let (git_url, sub_path) = parse_url_and_path(&url, &config.root)?;

    let workdir = repo
        .workdir()
        .context("Bare repositories are not supported")?;

    let full_path = workdir.join(&sub_path);

    if full_path.exists() {
        bail!("Path already exists: {}", full_path.display());
    }

    println!(
        "{} {} → {}",
        "Adding submodule".bold(),
        git_url.cyan(),
        sub_path.dimmed()
    );

    // Use git CLI for submodule add (git2 doesn't fully support this)
    let output = Command::new("git")
        .args(["submodule", "add", &git_url, &sub_path])
        .current_dir(workdir)
        .output()
        .context("Failed to run git submodule add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git submodule add failed: {}", stderr);
    }

    println!("{} Submodule added successfully.", "✓".green().bold());

    // Prompt to commit if we're on a TTY (unless --no-commit)
    if !no_commit && atty::is(atty::Stream::Stdin) {
        print!("Commit to parent repo? [Y/n] ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().is_empty() || input.trim().eq_ignore_ascii_case("y") {
            let msg = format!("Add submodule: {}", sub_path);
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
        } else {
            println!("{}", "Changes left staged.".dimmed());
        }
    }

    Ok(())
}

/// Parse clone input into a full git URL and a local path.
///
/// Supported inputs:
/// - owner/repo (expands to github.com)
/// - host/owner/repo
/// - full URLs (https://, ssh://, git@...)
fn parse_url_and_path(url: &str, root: &str) -> Result<(String, String)> {
    // Already a full URL
    if url.starts_with("https://") || url.starts_with("git@") || url.starts_with("ssh://") {
        let path = url_to_path(url, root)?;
        return Ok((url.to_string(), path));
    }

    // Shorthand: owner/repo (defaults to github.com)
    let parts: Vec<&str> = url.splitn(2, '/').collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() && !parts[1].contains('/') {
        let git_url = format!("https://github.com/{}", url);
        let path = format!("{}/github.com/{}", root, url);
        return Ok((git_url, path));
    }

    // Shorthand: github.com/user/repo
    let parts: Vec<&str> = url.splitn(3, '/').collect();
    if parts.len() == 3 {
        let git_url = format!("https://{}", url);
        let path = format!("{}/{}", root, url);
        return Ok((git_url, path));
    }

    bail!(
        "Could not parse URL: {}. Use owner/repo, github.com/user/repo, or a full git URL.",
        url
    );
}

fn url_to_path(url: &str, root: &str) -> Result<String> {
    let stripped = url
        .trim_start_matches("https://")
        .trim_start_matches("ssh://")
        .trim_start_matches("git@")
        .replace(':', "/")
        .trim_end_matches(".git")
        .to_string();

    Ok(format!("{}/{}", root, stripped))
}

#[cfg(test)]
mod tests {
    use super::parse_url_and_path;

    #[test]
    fn parses_owner_repo_slug() {
        let (git_url, path) = parse_url_and_path("tokio-rs/tokio", "src").unwrap();
        assert_eq!(git_url, "https://github.com/tokio-rs/tokio");
        assert_eq!(path, "src/github.com/tokio-rs/tokio");
    }

    #[test]
    fn parses_host_owner_repo_shorthand() {
        let (git_url, path) = parse_url_and_path("github.com/tokio-rs/tokio", "src").unwrap();
        assert_eq!(git_url, "https://github.com/tokio-rs/tokio");
        assert_eq!(path, "src/github.com/tokio-rs/tokio");
    }

    #[test]
    fn parses_full_https_url() {
        let (git_url, path) =
            parse_url_and_path("https://github.com/tokio-rs/tokio.git", "src").unwrap();
        assert_eq!(git_url, "https://github.com/tokio-rs/tokio.git");
        assert_eq!(path, "src/github.com/tokio-rs/tokio");
    }

    #[test]
    fn parses_full_ssh_url() {
        let (git_url, path) =
            parse_url_and_path("git@github.com:tokio-rs/tokio.git", "src").unwrap();
        assert_eq!(git_url, "git@github.com:tokio-rs/tokio.git");
        assert_eq!(path, "src/github.com/tokio-rs/tokio");
    }
}
