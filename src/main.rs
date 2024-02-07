mod args;
mod data;

use anstream::{print, println};
use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::Parser;

const STYLE_METRIC: Style = Style::new().bold();
const STYLE_CHANGE_NONE: Style = Style::new().dimmed();
const STYLE_CHANGE_BETTER: Style = Color::Ansi(AnsiColor::Green).on_default();
const STYLE_CHANGE_WORSE: Style = Color::Ansi(AnsiColor::Red).on_default();

fn main() -> Result<()> {
    let args = args::Args::parse();

    let metrics = data::Metrics::load(&args.metrics)?;
    match args.command {
        args::Command::Compare { baseline, test } => {
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
        args::Command::Generate { root } => {
            let values = metrics.generate(&root);
            print!("{}", values.format()?);
        }
    }

    Ok(())
}
