use serde::Serialize;
use soulfire_core::model::ai_model::AiModel;
use soulfire_core::model::ids::{AdventureId, ChatId};
use soulfire_core::model::metric::{MetricLabel, UsageMetric};
use soulfire_core::stats::{self, TokenTotals};
use tauri::State;

use crate::error::CommandError;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTotalsDto {
    pub requests: u64,
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTokenTotals {
    pub model: AiModel,
    pub totals: TokenTotalsDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationTokenTotals {
    pub label: MetricLabel,
    pub totals: TokenTotalsDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodTokenTotals {
    pub period: String,
    pub totals: TokenTotalsDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenStatsReportDto {
    pub metric_count: u64,
    pub totals: TokenTotalsDto,
    pub by_model: Vec<ModelTokenTotals>,
    pub by_operation: Vec<OperationTokenTotals>,
    pub by_day: Vec<PeriodTokenTotals>,
    pub by_month: Vec<PeriodTokenTotals>,
}

fn totals_dto(totals: TokenTotals) -> TokenTotalsDto {
    TokenTotalsDto {
        requests: totals.requests,
        input_tokens: totals.input_tokens,
        cached_input_tokens: totals.cached_input_tokens,
        output_tokens: totals.output_tokens,
    }
}

fn period_dto((period, totals): (String, TokenTotals)) -> PeriodTokenTotals {
    PeriodTokenTotals {
        period,
        totals: totals_dto(totals),
    }
}

fn report_from_metrics(metrics: Vec<UsageMetric>) -> TokenStatsReportDto {
    let report = stats::StatsReport::from_metrics(&metrics);
    TokenStatsReportDto {
        metric_count: metrics.len() as u64,
        totals: totals_dto(report.totals),
        by_model: report
            .by_model
            .into_iter()
            .map(|(model, totals)| ModelTokenTotals {
                model,
                totals: totals_dto(totals),
            })
            .collect(),
        by_operation: report
            .by_label
            .into_iter()
            .map(|(label, totals)| OperationTokenTotals {
                label,
                totals: totals_dto(totals),
            })
            .collect(),
        by_day: report.by_day.into_iter().map(period_dto).collect(),
        by_month: stats::by_month(&metrics)
            .into_iter()
            .map(period_dto)
            .collect(),
    }
}

#[tauri::command]
pub async fn get_token_stats(
    state: State<'_, AppState>,
) -> Result<TokenStatsReportDto, CommandError> {
    state
        .with_store(|store| Ok(report_from_metrics(store.all_metrics()?)))
        .await
}

#[tauri::command]
pub async fn get_chat_token_stats(
    chat_id: ChatId,
    state: State<'_, AppState>,
) -> Result<TokenStatsReportDto, CommandError> {
    state
        .with_store(move |store| Ok(report_from_metrics(store.metrics_for_chat(&chat_id)?)))
        .await
}

#[tauri::command]
pub async fn get_adventure_token_stats(
    adventure_id: AdventureId,
    state: State<'_, AppState>,
) -> Result<TokenStatsReportDto, CommandError> {
    state
        .with_store(move |store| {
            Ok(report_from_metrics(
                store.metrics_for_adventure(&adventure_id)?,
            ))
        })
        .await
}

#[tauri::command]
pub async fn clear_token_stats(
    state: State<'_, AppState>,
) -> Result<TokenStatsReportDto, CommandError> {
    state
        .with_store(|store| {
            store.clear_metrics()?;
            Ok(report_from_metrics(Vec::new()))
        })
        .await
}
