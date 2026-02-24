use anyhow::Result;
use clap::ValueEnum;
use git2::Repository;

#[derive(Clone, Debug, ValueEnum)]
pub enum CommitMode {
    Auto,
    Stage,
    Prompt,
}

pub struct Config {
    pub root: String,
    pub commit_mode: CommitMode,
}

impl Config {
    pub fn load(repo: &Repository) -> Result<Self> {
        let git_config = repo.config()?;

        let root = git_config
            .get_string("dom.root")
            .unwrap_or_else(|_| "src".to_string());

        let commit_mode = git_config
            .get_string("dom.commit")
            .ok()
            .and_then(|s| match s.as_str() {
                "stage" => Some(CommitMode::Stage),
                "prompt" => Some(CommitMode::Prompt),
                _ => None,
            })
            .unwrap_or(CommitMode::Auto);

        Ok(Config { root, commit_mode })
    }
}
