use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
