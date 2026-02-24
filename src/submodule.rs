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
    pub parent_changed: bool,
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
        let mut info = if abs_path.exists() {
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
                parent_changed: false,
            }
        };

        // Check if the submodule ref has uncommitted changes in the parent
        info.parent_changed = check_parent_changed(repo, &info.path);

        results.push(info);
    }

    Ok(results)
}

/// Check if the submodule has pending (staged or unstaged) changes in the parent repo's index.
fn check_parent_changed(repo: &Repository, sm_path: &Path) -> bool {
    let sm_path_str = match sm_path.to_str() {
        Some(s) => s,
        None => return false,
    };

    // Check for staged changes (index vs HEAD)
    let head_tree = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_tree().ok());

    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.pathspec(sm_path_str);

    // Index vs HEAD (staged)
    let staged = repo
        .diff_tree_to_index(head_tree.as_ref(), None, Some(&mut diff_opts))
        .ok()
        .is_some_and(|d| d.deltas().count() > 0);

    // Workdir vs index (unstaged submodule ref change)
    let mut diff_opts2 = git2::DiffOptions::new();
    diff_opts2.pathspec(sm_path_str);
    let unstaged = repo
        .diff_index_to_workdir(None, Some(&mut diff_opts2))
        .ok()
        .is_some_and(|d| d.deltas().count() > 0);

    staged || unstaged
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
        parent_changed: false, // filled in by discover()
    })
}
