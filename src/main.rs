mod args;
mod cache;
mod config;
mod data;
mod gitlab;
mod output;

use std::path::Path;

use anyhow::{Context as _, Result};
use log::{error, info};

fn main() -> Result<()> {
    env_logger::init();

    let args = args::parse();
    let config = config::load(args.config, args.command.root())?;

    match args.command {
        args::Command::Compare {
            baseline,
            test,
            compare_args,
        } => {
            let metrics = config.metrics()?;
            let baseline_values = data::Values::load(baseline)?;
            let test_values = data::Values::load(test)?;
            let comparisons = metrics.compare(&baseline_values, &test_values);
            output::print_comparisons(compare_args.output_format, &comparisons);
        }
        args::Command::Generate { cache, root } => {
            let metrics = config.metrics()?;
            let root = root.as_deref().unwrap_or_else(|| ".".as_ref());
            let (_, formatted) = generate(metrics, root, cache)?;
            print!("{}", formatted);
        }
        args::Command::Load { root, rev, gitlab } => {
            let root = root.as_deref().unwrap_or_else(|| ".".as_ref());
            let gitlab = gitlab.into_config()?;
            let rev = cache::get_rev(root, rev.rev.as_deref(), rev.base.as_deref())?;
            let s = load(&config, gitlab, root, &rev)?;
            print!("{}", s)
        }
        args::Command::Run {
            root,
            rev,
            gitlab,
            compare_args,
            cache,
        } => {
            let metrics = config.metrics()?;
            let root = root.as_deref().unwrap_or_else(|| ".".as_ref());
            let gitlab = gitlab.into_config()?;
            let baseline_rev = cache::get_rev(root, rev.rev.as_deref(), rev.base.as_deref())?;
            info!("Resolved baseline to commit {baseline_rev}");
            let baseline_values = load(&config, gitlab, root, &baseline_rev)?;
            let baseline_values = toml::from_str(&baseline_values)
                .context("failed to parse cached baseline values")?;
            let (values, _) = generate(metrics, root, cache)?;
            let comparisons = metrics.compare(&baseline_values, &values);
            output::print_comparisons(compare_args.output_format, &comparisons);
        }
    }

    Ok(())
}

fn generate(metrics: &data::Metrics, root: &Path, cache: bool) -> Result<(data::Values, String)> {
    let values = metrics.generate(root);
    let formatted = values.format()?;
    if cache {
        if let Err(err) = cache::store(root, &formatted) {
            error!("Failed to cache generated metrics: {}", err);
        }
    }
    Ok((values, formatted))
}

fn load(
    config: &config::Config,
    gitlab: Option<config::GitlabConfig>,
    root: &Path,
    rev: &str,
) -> Result<String> {
    if let Some(values) = cache::load(root, rev)? {
        return Ok(values);
    }
    if let Some(gitlab) = gitlab.as_ref().or(config.gitlab.as_ref()) {
        let s = gitlab
            .api()?
            .get_artifact(rev)
            .context("failed to retrieve metrics from Gitlab")?;
        if let Err(err) = cache::store_for_rev(root, rev, &s) {
            error!("Failed to cache downloaded metrics: {}", err);
        }
        return Ok(s);
    }
    anyhow::bail!("Missing cache entry for {rev} and no Gitlab configuration")
}
