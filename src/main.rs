mod args;
mod cache;
mod data;
mod gitlab;

use std::path::Path;

use anstream::{print, println};
use anstyle::{AnsiColor, Color, Style};
use anyhow::{Context as _, Result};
use clap::Parser;
use log::{error, info};

const STYLE_METRIC: Style = Style::new().bold();
const STYLE_CHANGE_NONE: Style = Style::new().dimmed();
const STYLE_CHANGE_BETTER: Style = Color::Ansi(AnsiColor::Green).on_default();
const STYLE_CHANGE_WORSE: Style = Color::Ansi(AnsiColor::Red).on_default();

fn main() -> Result<()> {
    env_logger::init();

    let args = args::Args::parse();

    match args.command {
        args::Command::Compare {
            metrics,
            baseline,
            test,
        } => {
            let metrics = data::Metrics::load(metrics)?;
            let baseline_values = data::Values::load(baseline)?;
            let test_values = data::Values::load(test)?;
            compare(&metrics, &baseline_values, &test_values);
        }
        args::Command::Generate {
            metrics,
            cache,
            root,
        } => {
            let metrics = data::Metrics::load(metrics)?;
            let (_, formatted) = generate(&metrics, &root, cache)?;
            print!("{}", formatted);
        }
        args::Command::Load { root, rev, gitlab } => {
            let rev = cache::get_rev(&root, rev.rev.as_deref(), rev.base.as_deref())?;
            let s = load(&gitlab, &root, &rev)?;
            print!("{}", s)
        }
        args::Command::Run {
            metrics,
            root,
            rev,
            gitlab,
            cache,
        } => {
            let metrics = data::Metrics::load(metrics)?;
            let root = root.as_deref().unwrap_or_else(|| ".".as_ref());
            let baseline_rev = cache::get_rev(root, rev.rev.as_deref(), rev.base.as_deref())?;
            info!("Resolved baseline to commit {baseline_rev}");
            let baseline_values = load(&gitlab, root, &baseline_rev)?;
            let baseline_values = toml::from_str(&baseline_values)
                .context("failed to parse cached baseline values")?;
            let (values, _) = generate(&metrics, root, cache)?;
            compare(&metrics, &baseline_values, &values);
        }
    }

    Ok(())
}

fn compare(metrics: &data::Metrics, baseline: &data::Values, test: &data::Values) {
    let comparisons = metrics.compare(baseline, test);
    for comparison in comparisons {
        let mut style_change = Style::new();
        if let Some(absolute_change) = comparison.absolute_change {
            if absolute_change.is_positive() {
                style_change = STYLE_CHANGE_WORSE;
            } else if absolute_change.is_negative() {
                style_change = STYLE_CHANGE_BETTER;
            } else {
                style_change = STYLE_CHANGE_NONE;
            }
        }
        print!("{STYLE_METRIC}{}{STYLE_METRIC:#}\t", comparison.metric);
        if let Some(old_value) = comparison.old_value {
            print!("{old_value}");
        } else {
            print!("-");
        }
        print!("\t");
        if let Some(new_value) = comparison.new_value {
            print!("{new_value}");
        } else {
            print!("-");
        }
        print!("\t");
        if let Some(absolute_change) = comparison.absolute_change {
            print!("{style_change}{absolute_change:+}{style_change:#}");
        } else {
            print!("-");
        }
        print!("\t");
        if let Some(relative_change) = comparison.relative_change {
            print!(
                "{style_change}{:+.2}%{style_change:#}",
                relative_change * 100.0
            );
        } else {
            print!("-");
        }
        println!();
    }
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

fn load(gitlab: &args::Gitlab, root: &Path, rev: &str) -> Result<String> {
    if let Some(values) = cache::load(root, rev)? {
        return Ok(values);
    }
    if gitlab.any() {
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
