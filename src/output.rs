mod markdown;
mod text;

use crate::{args::OutputFormat, data::Comparisons};

pub fn print_comparisons(format: OutputFormat, comparisons: &Comparisons) {
    match format {
        OutputFormat::Text => text::print_comparisons(comparisons),
        OutputFormat::Markdown => markdown::print_comparisons(comparisons),
    }
}
fn prettify_integer(i: impl ToString) -> String {
    let printed = i.to_string();

    let tmp: Vec<char> = printed.chars().collect();
    tmp.into_iter()
        .rev()
        .enumerate()
        .rev()
        .flat_map(|(idx, c)| {
            if idx % 3 == 0 && c != '-' && idx != 0 {
                [c].into_iter().chain(Some(','))
            } else {
                [c].into_iter().chain(None)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prettyfy() {
        assert_eq!(prettify_integer(0), "0");
        assert_eq!(prettify_integer(123), "123");
        assert_eq!(prettify_integer(1230), "1,230");
        assert_eq!(prettify_integer(1230000), "1,230,000");
        assert_eq!(prettify_integer(-1), "-1");
        assert_eq!(prettify_integer(-123), "-123");
        assert_eq!(prettify_integer(-1230), "-1,230");
        assert_eq!(prettify_integer(-1230000), "-1,230,000");
    }
}
