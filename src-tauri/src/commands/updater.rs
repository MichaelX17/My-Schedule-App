use tauri::State;

use crate::errors::AppResult;
use crate::updater::{self, StagedUpdateResult, UpdateCheckResult, UpdaterInfo};
use crate::AppState;

#[tauri::command]
pub fn get_updater_info() -> UpdaterInfo {
    updater::get_updater_info()
}

#[tauri::command]
pub async fn check_for_updates() -> UpdateCheckResult {
    let platform = updater::current_platform_key();
    updater::check_for_updates(env!("CARGO_PKG_VERSION"), &platform, None).await
}

#[tauri::command]
pub async fn download_and_stage_update(
    state: State<'_, AppState>,
) -> AppResult<StagedUpdateResult> {
    let pool = state.db.require_pool()?;
    let platform = updater::current_platform_key();
    updater::download_and_stage_update(
        &pool,
        &state.storage.paths,
        env!("CARGO_PKG_VERSION"),
        &platform,
        None,
    )
    .await
}
