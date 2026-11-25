use tauri::State;

use crate::errors::AppResult;
use crate::models::reports::{
    DashboardPeriod, DashboardSummary, OperationalReport, OperationalReportInput,
};
use crate::services::report_service;
use crate::AppState;

#[tauri::command]
pub async fn get_dashboard_summary(
    state: State<'_, AppState>,
    period: Option<DashboardPeriod>,
) -> AppResult<DashboardSummary> {
    let pool = state.db.require_pool()?;
    report_service::get_dashboard_summary(&pool, period.unwrap_or_default()).await
}

#[tauri::command]
pub async fn get_operational_report(
    state: State<'_, AppState>,
    input: Option<OperationalReportInput>,
) -> AppResult<OperationalReport> {
    let pool = state.db.require_pool()?;
    report_service::get_operational_report(&pool, input.unwrap_or_default()).await
}
