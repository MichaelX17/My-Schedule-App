use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::models::client::CreateClientInput;
use crate::time::DstDisambiguation;

/// Explicit appointment lifecycle status.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppointmentStatus {
    #[default]
    Scheduled,
    Completed,
    Cancelled,
}

impl AppointmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Persisted appointment-service snapshot row.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct AppointmentServiceItem {
    pub id: String,
    pub appointment_id: String,
    pub service_id: String,
    pub service_name_snapshot: String,
    pub service_description_snapshot: Option<String>,
    pub duration_minutes_snapshot: i64,
    pub sort_order: i64,
}

/// Raw appointment row joined with client and optional session plan metadata.
#[derive(Debug, Clone, FromRow)]
pub struct AppointmentRow {
    pub id: String,
    pub client_id: String,
    pub client_name: String,
    pub client_is_archived: bool,
    pub session_plan_id: Option<String>,
    pub session_plan_title: Option<String>,
    pub start_utc: String,
    pub end_utc: String,
    pub timezone: String,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Full appointment DTO returned to the frontend with business-local formatted times
/// and immutable service snapshots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppointmentDetail {
    pub id: String,
    pub client_id: String,
    pub client_name: String,
    pub client_is_archived: bool,
    pub session_plan_id: Option<String>,
    pub session_plan_title: Option<String>,
    pub start_utc: String,
    pub end_utc: String,
    pub start_local: String,
    pub end_local: String,
    pub timezone: String,
    pub status: String,
    pub notes: Option<String>,
    pub services: Vec<AppointmentServiceItem>,
    pub created_at: String,
    pub updated_at: String,
}

/// Overlap warning returned when a scheduled appointment overlaps another scheduled appointment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverlapWarning {
    pub appointment_id: String,
    pub client_name: String,
    pub start_utc: String,
    pub end_utc: String,
    pub start_local: String,
    pub end_local: String,
}

/// Result returned when creating, updating, or rescheduling an appointment,
/// including any non-blocking overlap warnings.
#[derive(Debug, Clone, Serialize)]
pub struct AppointmentMutationResult {
    pub appointment: AppointmentDetail,
    pub overlap_warnings: Vec<OverlapWarning>,
}

/// Input for creating an appointment (with either an existing `client_id` or `inline_client`).
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAppointmentInput {
    pub client_id: Option<String>,
    pub inline_client: Option<CreateClientInput>,
    pub session_plan_id: Option<String>,
    /// Business-local start date-time (`YYYY-MM-DDTHH:MM`).
    pub start_local: String,
    /// Business-local end date-time (`YYYY-MM-DDTHH:MM`).
    pub end_local: String,
    pub dst_disambiguation: Option<DstDisambiguation>,
    pub service_ids: Vec<String>,
    pub notes: Option<String>,
}

/// Input for updating an existing scheduled appointment's details, services, or time.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAppointmentInput {
    pub id: String,
    pub session_plan_id: Option<String>,
    pub start_local: String,
    pub end_local: String,
    pub dst_disambiguation: Option<DstDisambiguation>,
    pub service_ids: Vec<String>,
    pub notes: Option<String>,
}

/// Input for rescheduling a scheduled appointment to a new time window.
#[derive(Debug, Clone, Deserialize)]
pub struct RescheduleAppointmentInput {
    pub id: String,
    pub start_local: String,
    pub end_local: String,
    pub dst_disambiguation: Option<DstDisambiguation>,
}

/// Input for checking overlaps before saving.
#[derive(Debug, Clone, Deserialize)]
pub struct CheckOverlapsInput {
    pub start_local: String,
    pub end_local: String,
    pub dst_disambiguation: Option<DstDisambiguation>,
    pub exclude_appointment_id: Option<String>,
}

/// Filter/search parameters for listing appointments.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListAppointmentsParams {
    pub search: Option<String>,
    pub status: Option<AppointmentStatus>,
    pub client_id: Option<String>,
    pub session_plan_id: Option<String>,
    /// Inclusive start UTC bound (RFC 3339).
    pub from_utc: Option<String>,
    /// Inclusive end UTC bound (RFC 3339).
    pub to_utc: Option<String>,
}
