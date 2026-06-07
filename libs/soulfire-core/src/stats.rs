//! Token-statistics aggregation (`STAT`).
//!
//! Pure aggregation over the `UsageMetric` entries the engines record (`AI-15`,
//! `DATA-20a`). Token counts only — no cost figure is ever produced (`STAT` scope
//! note). Cached-input tokens are a subset of input tokens and are summed
//! separately so totals are not double-counted (`STAT-3`).

use indexmap::IndexMap;

use lib_soulfire::ai_model::AiModel;
use lib_soulfire::metric::{MetricLabel, UsageMetric};

/// Aggregate token totals over a set of usage entries (`STAT-4`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TokenTotals {
    pub requests: u64,
    pub input_tokens: u64,
    /// A subset of `input_tokens` (`STAT-3`).
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
}

impl TokenTotals {
    fn add(&mut self, m: &UsageMetric) {
        self.requests += 1;
        self.input_tokens += m.input_tokens;
        self.cached_input_tokens += m.cached_input_tokens.unwrap_or(0);
        self.output_tokens += m.output_tokens;
    }
}

/// Compute overall totals (`STAT-4`).
pub fn totals(metrics: &[UsageMetric]) -> TokenTotals {
    let mut t = TokenTotals::default();
    for m in metrics {
        t.add(m);
    }
    t
}

/// Totals broken down by model, in first-seen order (`STAT-4`).
pub fn by_model(metrics: &[UsageMetric]) -> IndexMap<AiModel, TokenTotals> {
    let mut map: IndexMap<AiModel, TokenTotals> = IndexMap::new();
    for m in metrics {
        map.entry(m.ai_model).or_default().add(m);
    }
    map
}

/// Totals broken down by operation label, in first-seen order (`STAT-4`).
pub fn by_label(metrics: &[UsageMetric]) -> IndexMap<MetricLabel, TokenTotals> {
    let mut map: IndexMap<MetricLabel, TokenTotals> = IndexMap::new();
    for m in metrics {
        map.entry(m.label).or_default().add(m);
    }
    map
}

/// Totals broken down by calendar day (`YYYY-MM-DD`, UTC), sorted ascending
/// (`STAT-4` time trend).
pub fn by_day(metrics: &[UsageMetric]) -> Vec<(String, TokenTotals)> {
    grouped_by_period(metrics, "%Y-%m-%d")
}

/// Totals broken down by month (`YYYY-MM`, UTC), sorted ascending (`STAT-4`).
pub fn by_month(metrics: &[UsageMetric]) -> Vec<(String, TokenTotals)> {
    grouped_by_period(metrics, "%Y-%m")
}

fn grouped_by_period(metrics: &[UsageMetric], format: &str) -> Vec<(String, TokenTotals)> {
    let mut map: IndexMap<String, TokenTotals> = IndexMap::new();
    for m in metrics {
        let key = m.created_at.format(format);
        map.entry(key).or_default().add(m);
    }
    let mut out: Vec<(String, TokenTotals)> = map.into_iter().collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// A full statistics report assembled from a set of entries (`STAT-5`).
#[derive(Debug, Clone, Default)]
pub struct StatsReport {
    pub totals: TokenTotals,
    pub by_model: IndexMap<AiModel, TokenTotals>,
    pub by_label: IndexMap<MetricLabel, TokenTotals>,
    pub by_day: Vec<(String, TokenTotals)>,
}

impl StatsReport {
    /// Build the report from usage entries (`STAT-4`, `STAT-5`).
    pub fn from_metrics(metrics: &[UsageMetric]) -> Self {
        StatsReport {
            totals: totals(metrics),
            by_model: by_model(metrics),
            by_label: by_label(metrics),
            by_day: by_day(metrics),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_soulfire::ids::ChatId;

    fn metric(
        label: MetricLabel,
        model: AiModel,
        input: u64,
        cached: u64,
        output: u64,
    ) -> UsageMetric {
        UsageMetric::builder()
            .label(label)
            .chat_id(ChatId::new())
            .input_tokens(input)
            .output_tokens(output)
            .cached_input_tokens(cached)
            .ai_model(model)
            .build()
    }

    #[test]
    fn totals_sum_and_partition_by_model_and_label() {
        // AC-STAT-b: totals equal the sum; breakdowns partition the totals.
        let metrics = vec![
            metric(MetricLabel::ChatMessage, AiModel::Gpt5_1, 100, 20, 50),
            metric(MetricLabel::ChatMessage, AiModel::Gpt5_1, 200, 0, 80),
            metric(
                MetricLabel::AdventureAction,
                AiModel::Gpt5_4Nano,
                300,
                30,
                40,
            ),
        ];
        let t = totals(&metrics);
        assert_eq!(t.requests, 3);
        assert_eq!(t.input_tokens, 600);
        assert_eq!(t.cached_input_tokens, 50);
        assert_eq!(t.output_tokens, 170);

        let models = by_model(&metrics);
        // by-model partitions the input total.
        let model_input_sum: u64 = models.values().map(|v| v.input_tokens).sum();
        assert_eq!(model_input_sum, t.input_tokens);
        assert_eq!(models[&AiModel::Gpt5_1].requests, 2);

        let labels = by_label(&metrics);
        let label_output_sum: u64 = labels.values().map(|v| v.output_tokens).sum();
        assert_eq!(label_output_sum, t.output_tokens);
        assert_eq!(labels[&MetricLabel::AdventureAction].input_tokens, 300);
    }

    #[test]
    fn empty_metrics_yield_zero_totals() {
        // AC-STAT-c: clearing history empties the aggregates.
        let t = totals(&[]);
        assert_eq!(t, TokenTotals::default());
    }
}
