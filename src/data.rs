use std::{collections::BTreeMap, fs, path::Path};

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
    pub old_value: Option<usize>,
    pub new_value: Option<usize>,
    pub absolute_change: Option<isize>,
    pub relative_change: Option<f32>,
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
            old_value,
            new_value,
            absolute_change,
            relative_change,
        }
    }
}
