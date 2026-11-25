use tauri::State;

use crate::errors::AppResult;
use crate::models::settings::{AppSettings, UpdateAppSettingsInput};
use crate::services::settings_service;
use crate::AppState;

#[tauri::command]
pub async fn get_app_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
    let pool = state.db.require_pool()?;
    settings_service::get_settings(&pool).await
}

#[tauri::command]
pub async fn update_app_settings(
    state: State<'_, AppState>,
    input: UpdateAppSettingsInput,
) -> AppResult<AppSettings> {
    let pool = state.db.require_pool()?;
    settings_service::update_settings(
        &pool,
        Some(&state.storage.paths.automatic_backups_dir),
        input,
    )
    .await
}
