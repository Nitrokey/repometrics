use std::path::PathBuf;

use anyhow::{Context as _, Result};
use clap::{Parser, Subcommand};

use crate::gitlab::Api;

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Compare {
        #[arg(long)]
        metrics: String,
        baseline: PathBuf,
        test: PathBuf,
    },
    Generate {
        #[arg(long)]
        metrics: String,
        #[arg(long)]
        cache: bool,
        root: PathBuf,
    },
    Load {
        root: PathBuf,
        #[command(flatten)]
        rev: Rev,
        #[command(flatten)]
        gitlab: Gitlab,
    },
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
    pub fn any(&self) -> bool {
        self.host.is_some()
            || self.project.is_some()
            || self.job.is_some()
            || self.artifact.is_some()
    }

    pub fn api(&self) -> Result<Api<'_>> {
        let host = self.host.as_deref().context("--gitlab-host not set")?;
        let project = self
            .project
            .as_deref()
            .context("--gitlab-project not set")?;
        let job = self.job.as_deref().context("--gitlab-job not set")?;
        let artifact = self
            .artifact
            .as_deref()
            .context("--gitlab-artifact not set")?;
        Api::new(host, project, job, artifact)
    }
}
