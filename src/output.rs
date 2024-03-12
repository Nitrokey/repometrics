mod markdown;
mod text;

use crate::{args::OutputFormat, data::Comparison};

pub fn print_comparisons(format: OutputFormat, comparisons: &[Comparison]) {
    match format {
        OutputFormat::Text => text::print_comparisons(comparisons),
        OutputFormat::Markdown => markdown::print_comparisons(comparisons),
    }
}
