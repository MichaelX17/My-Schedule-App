use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Persisted offered service record (non-financial).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub default_duration_minutes: i64,
    pub is_active: bool,
    pub is_archived: bool,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Immutable service snapshot captured at appointment booking time (used in Step 05)
/// so renaming or archiving a service never rewrites historical appointments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceSnapshot {
    pub service_id: String,
    pub service_name: String,
    pub service_description: Option<String>,
    pub duration_minutes: i64,
}

impl Service {
    /// Captures a booking-time snapshot of this service's descriptive attributes.
    pub fn to_snapshot(&self) -> ServiceSnapshot {
        ServiceSnapshot {
            service_id: self.id.clone(),
            service_name: self.name.clone(),
            service_description: self.description.clone(),
            duration_minutes: self.default_duration_minutes,
        }
    }
}

/// Input payload for creating a new offered service.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateServiceInput {
    pub name: String,
    pub description: Option<String>,
    pub default_duration_minutes: i64,
}

/// Input payload for updating an existing offered service.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateServiceInput {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub default_duration_minutes: i64,
}

/// Visibility filter for listing services. Defaults to `Active` so inactive
/// and archived services are excluded from new appointment selection by default.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatusFilter {
    #[default]
    Active,
    Inactive,
    Archived,
    All,
}

/// Sort order options for listing services.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceSortBy {
    #[default]
    NameAsc,
    NameDesc,
    DurationAsc,
    DurationDesc,
    RecentlyUpdated,
}

/// Query parameters for listing/searching services.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListServicesParams {
    pub search: Option<String>,
    pub filter: Option<ServiceStatusFilter>,
    pub sort_by: Option<ServiceSortBy>,
}

/// Service catalog list response including per-state totals.
#[derive(Debug, Clone, Serialize)]
pub struct ServiceListResult {
    pub items: Vec<Service>,
    pub total: i64,
    pub active_count: i64,
    pub inactive_count: i64,
    pub archived_count: i64,
}
