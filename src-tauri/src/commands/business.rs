use tauri::State;

use crate::errors::AppResult;
use crate::models::business::{
    BusinessLogoData, BusinessProfile, CreateBusinessProfileInput, OnboardingStatus,
    UpdateBusinessProfileInput,
};
use crate::services::business_service;
use crate::AppState;

/// Returns whether onboarding is complete, the business profile (if any),
/// and the OS-detected IANA timezone identifier.
#[tauri::command]
pub async fn get_onboarding_status(state: State<'_, AppState>) -> AppResult<OnboardingStatus> {
    let pool = state.db.require_pool()?;
    business_service::get_onboarding_status(&pool).await
}

/// Retrieves the single business profile for this installation.
#[tauri::command]
pub async fn get_business_profile(
    state: State<'_, AppState>,
) -> AppResult<Option<BusinessProfile>> {
    let pool = state.db.require_pool()?;
    business_service::get_business_profile(&pool).await
}

/// Creates the initial business profile during first-run onboarding.
#[tauri::command]
pub async fn create_business_profile(
    state: State<'_, AppState>,
    input: CreateBusinessProfileInput,
) -> AppResult<BusinessProfile> {
    let pool = state.db.require_pool()?;
    business_service::create_business_profile(&pool, &state.storage.paths.logos_dir, input).await
}

/// Updates the existing business profile details.
#[tauri::command]
pub async fn update_business_profile(
    state: State<'_, AppState>,
    input: UpdateBusinessProfileInput,
) -> AppResult<BusinessProfile> {
    let pool = state.db.require_pool()?;
    business_service::update_business_profile(&pool, input).await
}

/// Uploads and replaces the business logo in managed application storage.
#[tauri::command]
pub async fn upload_business_logo(
    state: State<'_, AppState>,
    bytes: Vec<u8>,
) -> AppResult<BusinessLogoData> {
    let pool = state.db.require_pool()?;
    business_service::upload_business_logo(&pool, &state.storage.paths.logos_dir, bytes).await
}

/// Reads the managed business logo as a data URL for frontend display.
#[tauri::command]
pub async fn get_business_logo(state: State<'_, AppState>) -> AppResult<Option<BusinessLogoData>> {
    let pool = state.db.require_pool()?;
    business_service::get_business_logo(&pool, &state.storage.paths.logos_dir).await
}

/// Removes the business logo reference and managed file.
#[tauri::command]
pub async fn remove_business_logo(state: State<'_, AppState>) -> AppResult<BusinessProfile> {
    let pool = state.db.require_pool()?;
    business_service::remove_business_logo(&pool, &state.storage.paths.logos_dir).await
}
