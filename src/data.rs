use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter, Write as _},
    fs,
    ops::Deref,
    path::Path,
};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};

use crate::config::Defaults;

#[derive(Debug, Deserialize)]
pub struct Metrics(BTreeMap<String, Metric>);

impl Metrics {
    pub fn generate(&self, root: &Path) -> Values {
        let mut values = ValuesV1::default();
        for (id, metric) in &self.0 {
            if let Some(value) = metric.generate(root) {
                values.values.insert(id.to_owned(), value);
            }
        }
        Values::V1(values)
    }

    pub fn compare(&self, defaults: &Defaults, baseline: &Values, test: &Values) -> Comparisons {
        let mut comparisons = Comparisons::default();
        for (id, metric) in &self.0 {
            let significance_threshold = metric
                .significance_threshold
                .or(defaults.significance_threshold);
            let old_value = baseline.get(id);
            let new_value = test.get(id);
            let comparison = Comparison::new(id.to_owned(), old_value, new_value);
            let is_significant = significance_threshold
                .zip(comparison.relative_change)
                .map(|(threshold, change)| change.abs() >= threshold)
                .unwrap_or(true);
            if is_significant {
                comparisons.significant.push(comparison);
            } else {
                comparisons.insignificant.push(comparison);
            }
        }
        comparisons
    }
}

#[derive(Debug, Deserialize)]
pub struct Metric {
    #[serde(flatten)]
    def: MetricDef,
    significance_threshold: Option<f32>,
}

impl Metric {
    fn generate(&self, root: &Path) -> Option<usize> {
        self.def.generate(root)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum MetricDef {
    FileSize(FileSize),
}

impl MetricDef {
    fn generate(&self, root: &Path) -> Option<usize> {
        match self {
            Self::FileSize(metric) => metric.generate(root),
        }
    }
}

#[derive(Debug, Deserialize)]
struct FileSize {
    input: String,
}

impl FileSize {
    fn generate(&self, root: &Path) -> Option<usize> {
        let path = root.join(&self.input);
        let metadata = fs::metadata(path).ok()?;
        metadata.len().try_into().ok()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "version")]
pub enum Values {
    #[serde(rename = "1")]
    V1(ValuesV1),
}

impl Values {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let s = fs::read_to_string(path)
            .with_context(|| format!("failed to read values file '{}'", path.display()))?;
        toml::from_str(&s)
            .with_context(|| format!("failed to parse values file '{}'", path.display()))
    }

    pub fn get(&self, metric: &String) -> Option<usize> {
        match self {
            Self::V1(values) => values.values.get(metric).copied(),
        }
    }

    pub fn format(&self) -> Result<String> {
        toml::to_string_pretty(self).context("failed to format metric values")
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ValuesV1 {
    values: BTreeMap<String, usize>,
}

#[derive(Debug, Default)]
pub struct Comparisons {
    pub significant: Vec<Comparison>,
    pub insignificant: Vec<Comparison>,
}

#[derive(Debug)]
pub struct Comparison {
    pub metric: String,
    pub old_value: Option<AbsoluteValue>,
    pub new_value: Option<AbsoluteValue>,
    pub absolute_change: Option<AbsoluteChange>,
    pub relative_change: Option<RelativeChange>,
}

impl Comparison {
    fn new(metric: String, old_value: Option<usize>, new_value: Option<usize>) -> Self {
        let mut absolute_change = None;
        let mut relative_change = None;
        if let Some((old_value, new_value)) = old_value.zip(new_value) {
            if let Ok(old_value) = isize::try_from(old_value) {
                if let Ok(new_value) = isize::try_from(new_value) {
                    if let Some(delta) = new_value.checked_sub(old_value) {
                        absolute_change = Some(delta);
                        relative_change = Some((delta as f32) / (old_value as f32));
                    }
                }
            }
        }
        Self {
            metric,
            old_value: old_value.map(AbsoluteValue),
            new_value: new_value.map(AbsoluteValue),
            absolute_change: absolute_change.map(AbsoluteChange),
            relative_change: relative_change.map(RelativeChange),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AbsoluteValue(usize);

impl Deref for AbsoluteValue {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for AbsoluteValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        display_int(f, &self.0.to_string())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AbsoluteChange(isize);

impl Deref for AbsoluteChange {
    type Target = isize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for AbsoluteChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        display_int(f, &format!("{:+}", self.0))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RelativeChange(f32);

impl Deref for RelativeChange {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for RelativeChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:+.2}%", self.0 * 100.0)
    }
}

fn display_int(f: &mut Formatter<'_>, s: &str) -> fmt::Result {
    let tmp: Vec<char> = s.chars().collect();
    tmp.into_iter()
        .rev()
        .enumerate()
        .rev()
        .flat_map(|(idx, c)| {
            if idx % 3 == 0 && c != '+' && c != '-' && idx != 0 {
                [c].into_iter().chain(Some(','))
            } else {
                [c].into_iter().chain(None)
            }
        })
        .try_for_each(|c| f.write_char(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_absolute_value() {
        assert_eq!(AbsoluteValue(0).to_string(), "0");
        assert_eq!(AbsoluteValue(123).to_string(), "123");
        assert_eq!(AbsoluteValue(1230).to_string(), "1,230");
        assert_eq!(AbsoluteValue(1230000).to_string(), "1,230,000");
    }

    #[test]
    fn display_absolute_change() {
        assert_eq!(AbsoluteChange(0).to_string(), "+0");
        assert_eq!(AbsoluteChange(123).to_string(), "+123");
        assert_eq!(AbsoluteChange(1230).to_string(), "+1,230");
        assert_eq!(AbsoluteChange(1230000).to_string(), "+1,230,000");
        assert_eq!(AbsoluteChange(-1).to_string(), "-1");
        assert_eq!(AbsoluteChange(-123).to_string(), "-123");
        assert_eq!(AbsoluteChange(-1230).to_string(), "-1,230");
        assert_eq!(AbsoluteChange(-1230000).to_string(), "-1,230,000");
    }

    #[test]
    fn display_relative_change() {
        assert_eq!(RelativeChange(0.0).to_string(), "+0.00%");
        assert_eq!(RelativeChange(0.1).to_string(), "+10.00%");
        assert_eq!(RelativeChange(0.9999).to_string(), "+99.99%");
        assert_eq!(RelativeChange(-0.1).to_string(), "-10.00%");
        assert_eq!(RelativeChange(-0.9999).to_string(), "-99.99%");
    }
}
