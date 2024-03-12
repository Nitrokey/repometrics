use anstream::{print, println};
use anstyle::{AnsiColor, Color, Style};

use crate::data::Comparison;

const STYLE_METRIC: Style = Style::new().bold();
const STYLE_CHANGE_NONE: Style = Style::new().dimmed();
const STYLE_CHANGE_BETTER: Style = Color::Ansi(AnsiColor::Green).on_default();
const STYLE_CHANGE_WORSE: Style = Color::Ansi(AnsiColor::Red).on_default();

pub fn print_comparisons(comparisons: &[Comparison]) {
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
