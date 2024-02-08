use std::{fs, path::Path, process::Command};

use anyhow::{Context as _, Result};
use log::{debug, error, info};

pub fn store(path: &Path, data: &str) -> Result<()> {
    if !is_git_repo(path) {
        debug!("Root is not a Git repository, not caching metrics");
        return Ok(());
    }
    if !is_clean(path) {
        debug!("Git repository is not in a clean state, not caching metrics");
        return Ok(());
    }
    let commit = get_commit(path)?;
    store_for_rev(path, &commit, data)
}

pub fn store_for_rev(path: &Path, rev: &str, data: &str) -> Result<()> {
    let cache_dir = path.join(".repometrics");
    if !cache_dir.exists() {
        fs::create_dir(&cache_dir).with_context(|| {
            format!("failed to create cache directory '{}'", cache_dir.display())
        })?;
    }
    let gitignore = cache_dir.join(".gitignore");
    if !gitignore.exists() {
        fs::write(&gitignore, "*").with_context(|| {
            format!("failed to create gitignore file '{}'", gitignore.display())
        })?;
    }
    let cache_file = cache_dir.join(format!("{}.toml", rev));
    info!("Writing data to cache file '{}'", cache_file.display());
    fs::write(&cache_file, data)
        .with_context(|| format!("failed to write cache file '{}'", cache_file.display()))
}

pub fn load(path: &Path, rev: &str) -> Result<String> {
    let cache_file = path.join(format!(".repometrics/{}.toml", rev));
    anyhow::ensure!(cache_file.exists(), "no cache entry for commit {rev}");
    info!("Reading cache file '{}'", cache_file.display());
    fs::read_to_string(&cache_file)
        .with_context(|| format!("failed to read cache file '{}'", cache_file.display()))
}

pub fn get_rev(path: &Path, rev: Option<&str>, base: Option<&str>) -> Result<String> {
    anyhow::ensure!(
        is_git_repo(path),
        "Directory '{}' is not a Git repository",
        path.display(),
    );
    match (rev, base) {
        (Some(rev), _) => resolve_commit(path, rev),
        (_, Some(base)) => merge_base(path, base),
        (None, None) => {
            anyhow::ensure!(
                is_clean(path),
                "Git repository '{}' is not in a clean state",
                path.display(),
            );
            get_commit(path)
        }
    }
}

fn is_git_repo(path: &Path) -> bool {
    path.join(".git").is_dir()
}

fn is_clean(path: &Path) -> bool {
    let result = Command::new("git")
        .current_dir(path)
        .arg("status")
        .arg("--porcelain")
        .output();
    let output = match result {
        Ok(output) => output,
        Err(err) => {
            error!("Could not run git status: {err}");
            return false;
        }
    };
    if !output.status.success() {
        error!(
            "Running git status failed with status code {} in '{}'",
            output.status,
            path.display()
        );
        return false;
    }
    output.stdout.is_empty()
}

fn get_commit(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(path)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .context("failed to run git rev-parse")?;
    anyhow::ensure!(
        output.status.success(),
        "running git rev-parse failed with status code {} in '{}'",
        output.status,
        path.display()
    );
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_owned())
        .context("failed to decode git revision as UTF-8")
}

fn resolve_commit(path: &Path, rev: &str) -> Result<String> {
    let output = Command::new("git")
        .current_dir(path)
        .arg("rev-list")
        .arg("-1")
        .arg(rev)
        .output()
        .context("failed to run git rev-list")?;
    anyhow::ensure!(
        output.status.success(),
        "running git rev-list failed with status code {} in '{}'",
        output.status,
        path.display()
    );
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_owned())
        .context("failed to decode git revision as UTF-8")
}

fn merge_base(path: &Path, base: &str) -> Result<String> {
    let output = Command::new("git")
        .current_dir(path)
        .arg("merge-base")
        .arg(base)
        .arg("HEAD")
        .output()
        .context("failed to run git merge-base")?;
    anyhow::ensure!(
        output.status.success(),
        "running git merge-base list failed with status code {} in '{}'",
        output.status,
        path.display()
    );
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_owned())
        .context("failed to decode git revision as UTF-8")
}
