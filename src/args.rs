use std::path::{Path, PathBuf};

use anyhow::{Context as _, Error, Result};
use clap::{Parser, Subcommand};

use crate::config::GitlabConfig;

pub fn parse() -> Args {
    Args::parse()
}

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Compare {
        baseline: PathBuf,
        test: PathBuf,
    },
    Generate {
        #[arg(long)]
        cache: bool,
        root: Option<PathBuf>,
    },
    Load {
        root: Option<PathBuf>,
        #[command(flatten)]
        rev: Rev,
        #[command(flatten)]
        gitlab: Gitlab,
    },
    Run {
        #[arg(long)]
        root: Option<PathBuf>,
        #[command(flatten)]
        rev: Rev,
        #[command(flatten)]
        gitlab: Gitlab,
        #[arg(long)]
        cache: bool,
    },
}

impl Command {
    pub fn root(&self) -> Option<&Path> {
        match self {
            Self::Compare { .. } => None,
            Self::Generate { root, .. } => root.as_deref(),
            Self::Load { root, .. } => root.as_deref(),
            Self::Run { root, .. } => root.as_deref(),
        }
    }
}

#[derive(Debug, clap::Args)]
#[group(multiple = false)]
pub struct Rev {
    #[arg(long)]
    pub rev: Option<String>,
    #[arg(long)]
    pub base: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct Gitlab {
    #[arg(long = "gitlab-host")]
    pub host: Option<String>,
    #[arg(long = "gitlab-project")]
    pub project: Option<String>,
    #[arg(long = "gitlab-job")]
    pub job: Option<String>,
    #[arg(long = "gitlab-artifact")]
    pub artifact: Option<String>,
}

impl Gitlab {
    fn any(&self) -> bool {
        self.host.is_some()
            || self.project.is_some()
            || self.job.is_some()
            || self.artifact.is_some()
    }

    pub fn into_config(self) -> Result<Option<GitlabConfig>> {
        if self.any() {
            self.try_into().map(Some)
        } else {
            Ok(None)
        }
    }
}

impl TryFrom<Gitlab> for GitlabConfig {
    type Error = Error;

    fn try_from(gitlab: Gitlab) -> Result<Self, Self::Error> {
        let host = gitlab.host.context("--gitlab-host not set")?;
        let project = gitlab.project.context("--gitlab-project not set")?;
        let job = gitlab.job.context("--gitlab-job not set")?;
        let artifact = gitlab.artifact.context("--gitlab-artifact not set")?;
        Ok(GitlabConfig {
            host,
            project,
            job,
            artifact,
        })
    }
}
