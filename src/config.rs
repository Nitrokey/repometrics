use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result};
use log::debug;
use serde::Deserialize;

use crate::{data::Metrics, gitlab::Api};

const CONFIG_FILE_NAME: &str = "repometrics.toml";

pub fn load(path: Option<PathBuf>, root: Option<&Path>) -> Result<Config> {
    // Priority 1: explicit argument
    if let Some(path) = path {
        return Config::load(path);
    }

    // Priority 2: repometrics.toml in the root directory
    if let Some(root) = root {
        let path = root.join(CONFIG_FILE_NAME);
        if path.exists() {
            return Config::load(path);
        }
    }
    // Priority 3: repometrics.toml in the working directory
    let path = Path::new(".").join(CONFIG_FILE_NAME);
    if path.exists() {
        return Config::load(path);
    }

    // Fallback: default configuration
    Ok(Default::default())
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub defaults: Defaults,
    pub gitlab: Option<GitlabConfig>,
    pub metrics: Option<Metrics>,
}

impl Config {
    fn load(path: PathBuf) -> Result<Self> {
        debug!("Loading config file {}", path.display());
        let s = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config file '{}'", path.display()))?;
        toml::from_str(&s)
            .with_context(|| format!("failed to parse config file '{}'", path.display()))
            .map(|mut config: Self| {
                config.path = Some(path);
                config
            })
    }

    pub fn metrics(&self) -> Result<&Metrics> {
        self.metrics
            .as_ref()
            .with_context(|| {
                if let Some(path) = &self.path {
                    format!("no metrics section in config file '{}'", path.display())
                } else {
                    "no config file found".to_string()
                }
            })
            .context("missing metrics definition")
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Defaults {
    pub significance_threshold: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct GitlabConfig {
    pub host: String,
    pub project: String,
    pub job: String,
    pub artifact: String,
}

impl GitlabConfig {
    pub fn api(&self) -> Result<Api<'_>> {
        Api::new(&self.host, &self.project, &self.job, &self.artifact)
    }
}
