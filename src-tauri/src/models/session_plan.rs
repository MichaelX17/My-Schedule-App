use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::models::appointment::AppointmentDetail;
use crate::time::DstDisambiguation;

/// Status of an optional multi-session plan.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionPlanStatus {
    #[default]
    Active,
    Completed,
    Archived,
}

impl SessionPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Archived => "archived",
        }
    }
}

/// Persisted session plan row joined with client name and linked appointment counts.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct SessionPlanSummary {
    pub id: String,
    pub client_id: String,
    pub client_name: String,
    pub title: String,
    pub description: Option<String>,
    pub planned_sessions: i64,
    pub status: String,
    pub linked_total: i64,
    pub linked_scheduled: i64,
    pub linked_completed: i64,
    pub linked_cancelled: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Core persisted session plan record.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct SessionPlan {
    pub id: String,
    pub client_id: String,
    pub title: String,
    pub description: Option<String>,
    pub planned_sessions: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSessionPlanInput {
    pub client_id: String,
    pub title: String,
    pub description: Option<String>,
    pub planned_sessions: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSessionPlanInput {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub planned_sessions: i64,
    pub status: Option<SessionPlanStatus>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListSessionPlansParams {
    pub client_id: Option<String>,
    pub status: Option<SessionPlanStatus>,
}

/// One scheduled session slot when creating a session plan together with its appointments.
#[derive(Debug, Clone, Deserialize)]
pub struct PlanSessionSlotInput {
    pub start_local: String,
    pub end_local: String,
    #[serde(default)]
    pub dst_disambiguation: Option<DstDisambiguation>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Input to atomically create a session plan along with one individual appointment per session.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSessionPlanWithAppointmentsInput {
    pub client_id: String,
    pub title: String,
    pub description: Option<String>,
    pub planned_sessions: i64,
    pub service_ids: Vec<String>,
    pub sessions: Vec<PlanSessionSlotInput>,
}

/// Output of creating a session plan together with all scheduled session appointments.
#[derive(Debug, Clone, Serialize)]
pub struct CreateSessionPlanWithAppointmentsResult {
    pub plan: SessionPlanSummary,
    pub appointments: Vec<AppointmentDetail>,
}
