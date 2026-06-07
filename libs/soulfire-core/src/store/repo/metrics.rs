//! Usage-metric persistence (`DATA-20a`). Aggregation lives in the stats layer
//! and reads these rows (`STAT`).

use lib_soulfire::ids::{AdventureId, ChatId};
use lib_soulfire::metric::UsageMetric;
use rusqlite::params;

use crate::error::CoreResult;
use crate::store::Store;

use super::{count, select_many, to_data};

impl Store {
    /// Persist a usage metric. Zero-token records are not written (`DATA-20a`);
    /// the caller is expected to skip them, and this guards it as well.
    pub fn save_metric(&self, metric: &UsageMetric) -> CoreResult<()> {
        if metric.is_zero() {
            return Ok(());
        }
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO usage_metrics
                   (metric_id, created_at, label, ai_model, chat_id, adventure_id,
                    blueprint_id, character_id, input_tokens, cached_input_tokens,
                    output_tokens, data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    metric.metric_id.to_string(),
                    metric.created_at.to_string(),
                    metric.label.to_string(),
                    metric.ai_model.to_string(),
                    metric.chat_id.as_ref().map(|i| i.to_string()),
                    metric.adventure_id.as_ref().map(|i| i.to_string()),
                    metric.blueprint_id.as_ref().map(|i| i.to_string()),
                    metric.character_id.as_ref().map(|i| i.to_string()),
                    metric.input_tokens as i64,
                    metric.cached_input_tokens.map(|t| t as i64),
                    metric.output_tokens as i64,
                    to_data(metric)?,
                ],
            )?;
            Ok(())
        })
    }

    /// All usage metrics, newest first.
    pub fn all_metrics(&self) -> CoreResult<Vec<UsageMetric>> {
        self.with_conn(|conn| {
            select_many(
                conn,
                "SELECT data FROM usage_metrics ORDER BY created_at DESC",
                [],
            )
        })
    }

    pub fn metrics_for_chat(&self, chat_id: &ChatId) -> CoreResult<Vec<UsageMetric>> {
        self.with_conn(|conn| {
            select_many(
                conn,
                "SELECT data FROM usage_metrics WHERE chat_id = ?1 ORDER BY created_at DESC",
                params![chat_id.to_string()],
            )
        })
    }

    pub fn metrics_for_adventure(&self, adventure_id: &AdventureId) -> CoreResult<Vec<UsageMetric>> {
        self.with_conn(|conn| {
            select_many(
                conn,
                "SELECT data FROM usage_metrics WHERE adventure_id = ?1 ORDER BY created_at DESC",
                params![adventure_id.to_string()],
            )
        })
    }

    pub fn count_metrics(&self) -> CoreResult<i64> {
        self.with_conn(|conn| count(conn, "SELECT count(*) FROM usage_metrics", []))
    }

    /// Clear all usage history (`STAT-2`; confirmed by the UI).
    pub fn clear_metrics(&self) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute("DELETE FROM usage_metrics", [])?;
            Ok(())
        })
    }
}
