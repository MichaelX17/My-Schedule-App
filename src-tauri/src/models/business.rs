use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Persisted single-business profile for this installation.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct BusinessProfile {
    pub id: String,
    pub name: String,
    pub logo_filename: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub timezone: String,
    pub onboarding_completed: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Input payload for initial first-run business profile creation.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBusinessProfileInput {
    pub name: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub timezone: String,
    /// Optional raw image bytes uploaded during onboarding.
    pub logo_bytes: Option<Vec<u8>>,
}

/// Input payload for updating the existing business profile.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBusinessProfileInput {
    pub name: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub timezone: String,
}

/// First-run onboarding state returned at startup.
#[derive(Debug, Clone, Serialize)]
pub struct OnboardingStatus {
    pub completed: bool,
    pub business: Option<BusinessProfile>,
    pub detected_timezone: String,
}

/// Managed logo data serialized safely as a data URL for frontend rendering.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BusinessLogoData {
    pub filename: String,
    pub mime_type: String,
    pub data_url: String,
}
