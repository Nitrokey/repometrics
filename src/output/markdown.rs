use crate::data::Comparison;

pub fn print_comparisons(comparisons: &[Comparison]) {
    println!("| metric | value |     | change |");
    println!("| ------ | ----: | :-: | :----: |");
    for comparison in comparisons {
        let trend = if let Some(absolute_change) = comparison.absolute_change {
            if absolute_change.is_positive() {
                ":red_circle:"
            } else if absolute_change.is_negative() {
                ":white_check_mark:"
            } else {
                ":white_circle:"
            }
        } else {
            ""
        };
        print!("| {} | ", comparison.metric);
        if let Some(new_value) = comparison.new_value {
            print!("{new_value}");
        } else {
            print!("-");
        }
        print!(" | {trend} | ");
        let change = comparison.absolute_change.zip(comparison.relative_change);
        if let Some((absolute_change, relative_change)) = change {
            print!("{absolute_change:+} ({relative_change:+.2}%)");
        }
        println!(" |");
    }
}
