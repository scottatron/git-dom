use anyhow::{Context, Result};
use git2::{Repository, StatusOptions};
use std::path::{Path, PathBuf};

pub struct SubmoduleInfo {
    pub name: String,
    pub path: PathBuf,
    pub url: String,
    pub branch: Option<String>,
    pub head_commit: Option<String>,
    pub head_message: Option<String>,
    pub is_dirty: bool,
    pub ahead: usize,
    pub behind: usize,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
}

/// Discover all submodules in the given repo, optionally filtering by name.
pub fn discover(repo: &Repository, filter: Option<&str>) -> Result<Vec<SubmoduleInfo>> {
    let submodules = repo.submodules().context("Failed to read submodules")?;
    let workdir = repo
        .workdir()
        .context("Bare repositories are not supported")?;

    let mut results = Vec::new();

    for sm in &submodules {
        let name = sm.name().unwrap_or("").to_string();

        if let Some(f) = filter {
            if name != f {
                continue;
            }
        }

        let path = PathBuf::from(sm.path());
        let url = sm.url().unwrap_or("").to_string();

        let abs_path = workdir.join(&path);
        let info = if abs_path.exists() {
            gather_info(&name, &path, &url, &abs_path)?
        } else {
            SubmoduleInfo {
                name,
                path,
                url,
                branch: None,
                head_commit: None,
                head_message: None,
                is_dirty: false,
                ahead: 0,
                behind: 0,
                staged: 0,
                modified: 0,
                untracked: 0,
            }
        };

        results.push(info);
    }

    Ok(results)
}

fn gather_info(
    name: &str,
    path: &Path,
    url: &str,
    abs_path: &Path,
) -> Result<SubmoduleInfo> {
    let sub_repo = Repository::open(abs_path)
        .with_context(|| format!("Failed to open submodule repo at {}", abs_path.display()))?;

    let branch = sub_repo
        .head()
        .ok()
        .and_then(|h| {
            if h.is_branch() {
                h.shorthand().map(|s| s.to_string())
            } else {
                // Detached HEAD — show short commit hash
                h.target().map(|oid| format!("{:.7}", oid))
            }
        });

    let (head_commit, head_message) = sub_repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| {
            let id = format!("{:.7}", c.id());
            let msg = c.summary().unwrap_or("").to_string();
            (Some(id), Some(msg))
        })
        .unwrap_or((None, None));

    // Count staged, modified, untracked
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    let statuses = sub_repo.statuses(Some(&mut opts)).unwrap_or_else(|_| {
        // Return empty statuses on error
        sub_repo.statuses(None).unwrap()
    });

    let mut staged = 0;
    let mut modified = 0;
    let mut untracked = 0;

    for entry in statuses.iter() {
        let s = entry.status();
        if s.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        ) {
            staged += 1;
        }
        if s.intersects(
            git2::Status::WT_MODIFIED | git2::Status::WT_DELETED | git2::Status::WT_RENAMED,
        ) {
            modified += 1;
        }
        if s.contains(git2::Status::WT_NEW) {
            untracked += 1;
        }
    }

    let is_dirty = staged > 0 || modified > 0;

    // Ahead/behind upstream
    let (ahead, behind) = sub_repo
        .head()
        .ok()
        .and_then(|head| {
            let local_oid = head.target()?;
            let branch_name = head.shorthand()?;
            let upstream_name = format!("origin/{}", branch_name);
            let upstream_ref = sub_repo.find_reference(&format!("refs/remotes/{}", upstream_name)).ok()?;
            let upstream_oid = upstream_ref.target()?;
            sub_repo.graph_ahead_behind(local_oid, upstream_oid).ok()
        })
        .unwrap_or((0, 0));

    Ok(SubmoduleInfo {
        name: name.to_string(),
        path: path.to_path_buf(),
        url: url.to_string(),
        branch,
        head_commit,
        head_message,
        is_dirty,
        ahead,
        behind,
        staged,
        modified,
        untracked,
    })
}
