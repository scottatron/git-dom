use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    emit_git_rerun_rules();

    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let git_describe = git(&["describe", "--tags", "--always", "--dirty"]);

    let version = match git_describe {
        Some(describe) if describe == format!("v{pkg_version}") => pkg_version,
        Some(describe) => format!("{pkg_version} ({describe})"),
        None => pkg_version,
    };

    println!("cargo:rustc-env=GIT_DOM_VERSION={version}");
}

fn emit_git_rerun_rules() {
    if let Some(head_path) = git(&["rev-parse", "--git-path", "HEAD"]) {
        println!("cargo:rerun-if-changed={head_path}");
    }

    if let Some(symbolic_ref) = git(&["symbolic-ref", "-q", "HEAD"])
        && let Some(ref_path) = git(&["rev-parse", "--git-path", symbolic_ref.trim()])
    {
        println!("cargo:rerun-if-changed={ref_path}");
    }

    if let Some(packed_refs_path) = git(&["rev-parse", "--git-path", "packed-refs"]) {
        println!("cargo:rerun-if-changed={packed_refs_path}");
    }

    if let Some(git_dir) = git(&["rev-parse", "--git-dir"]) {
        let tags_dir = PathBuf::from(git_dir).join("refs/tags");
        if let Some(tags_dir) = tags_dir.to_str() {
            println!("cargo:rerun-if-changed={tags_dir}");
        }
    }
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    Some(value.to_string())
}
