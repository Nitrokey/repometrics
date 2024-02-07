mod args;
mod cache;
mod data;

use anstream::{print, println};
use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::Parser;
use log::error;

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
            let metrics = data::Metrics::load(&metrics)?;
            let baseline_values = data::Values::load(&baseline)?;
            let test_values = data::Values::load(&test)?;
            let comparisons = metrics.compare(&baseline_values, &test_values);
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
        args::Command::Generate {
            metrics,
            cache,
            root,
        } => {
            let metrics = data::Metrics::load(metrics)?;
            let values = metrics.generate(&root);
            let formatted = values.format()?;
            if cache {
                if let Err(err) = cache::store(&root, &formatted) {
                    error!("Failed to cache generated metrics: {}", err);
                }
            }
            print!("{}", formatted);
        }
        args::Command::Load { root, rev } => {
            let s = cache::load(&root, rev.rev.as_deref(), rev.base.as_deref())?;
            print!("{}", s)
        }
    }

    Ok(())
}
