use serde::{Deserialize, Serialize};

use crate::models::appointment::{AppointmentDetail, AppointmentStatus};

/// Supported calendar display modes.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CalendarView {
    Day,
    #[default]
    Week,
    Month,
    Agenda,
}

/// Input parameters for querying appointments in a business-local calendar range.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CalendarRangeInput {
    #[serde(default)]
    pub view: CalendarView,
    /// Optional anchor date (`YYYY-MM-DD`) in the business timezone. Defaults to today.
    pub anchor_date: Option<String>,
    /// Optional appointment status filter. `None` returns all statuses.
    pub status: Option<AppointmentStatus>,
}

/// Calendar query response with authoritative business-timezone boundaries and hydrated appointments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarRangeResponse {
    pub view: CalendarView,
    pub timezone: String,
    pub today_local: String,
    pub anchor_date: String,
    pub range_start_local: String,
    pub range_end_local: String,
    pub range_start_utc: String,
    pub range_end_utc_exclusive: String,
    pub appointments: Vec<AppointmentDetail>,
}
