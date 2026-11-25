use serde::{Deserialize, Serialize};

use crate::models::appointment::AppointmentDetail;

/// Selectable summary period for Dashboard KPIs.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DashboardPeriod {
    Today,
    #[default]
    ThisWeek,
    ThisMonth,
}

/// Reporting cadence for operational summaries.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportPeriodType {
    #[default]
    Weekly,
    Monthly,
}

/// Explicit appointment counts by status.
///
/// Semantics:
/// - `scheduled`: active booked appointments (not completed or cancelled).
/// - `completed`: finished appointments explicitly marked completed.
/// - `cancelled`: cancelled appointments; never counted as completed or upcoming.
/// - `total`: `scheduled + completed + cancelled`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusCounts {
    pub total: i64,
    pub scheduled: i64,
    pub completed: i64,
    pub cancelled: i64,
}

/// Non-financial service usage aggregation derived from historical appointment-service snapshots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceUsageItem {
    pub service_name: String,
    pub appointment_count: i64,
    pub total_minutes: i64,
}

/// Per-day status breakdown for accessible charts and tabular reports.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DailyBreakdownPoint {
    pub date_local: String,
    pub scheduled: i64,
    pub completed: i64,
    pub cancelled: i64,
    pub total: i64,
}

/// Persisted operational dashboard snapshot calculated in the business timezone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardSummary {
    pub business_name: String,
    pub timezone: String,
    pub today_local: String,
    pub period: DashboardPeriod,
    pub period_start_local: String,
    pub period_end_local: String,
    pub today_appointments: Vec<AppointmentDetail>,
    pub upcoming_appointments: Vec<AppointmentDetail>,
    pub today_counts: StatusCounts,
    pub period_counts: StatusCounts,
    /// Scheduled appointments whose end time has not yet passed (`status = 'scheduled' AND end_utc > now`).
    pub upcoming_scheduled_count: i64,
    /// Scheduled appointments whose end time has passed and still need a `completed` or `cancelled` outcome.
    pub needs_outcome_count: i64,
    pub active_clients_count: i64,
    pub active_session_plans_count: i64,
    pub unread_notifications_count: i64,
}

/// Input parameters for querying a weekly or monthly operational report.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OperationalReportInput {
    #[serde(default)]
    pub period_type: ReportPeriodType,
    /// Optional business-local anchor date (`YYYY-MM-DD`). Defaults to business-local today.
    pub anchor_date: Option<String>,
}

/// Non-financial weekly or monthly operational summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperationalReport {
    pub period_type: ReportPeriodType,
    pub timezone: String,
    pub anchor_date: String,
    pub period_start_local: String,
    pub period_end_local: String,
    pub period_start_utc: String,
    pub period_end_utc_exclusive: String,
    pub counts: StatusCounts,
    /// `completed / total * 100` (rounded to 1 decimal place), or `None` when `total == 0`.
    pub completion_rate_percent: Option<f64>,
    /// Total scheduled/completed appointment duration in minutes (excludes cancelled).
    pub total_active_minutes: i64,
    /// Distinct clients with at least one non-cancelled appointment in the period.
    pub unique_clients_served: i64,
    pub daily_breakdown: Vec<DailyBreakdownPoint>,
    pub top_services: Vec<ServiceUsageItem>,
}
