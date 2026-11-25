pub mod appointments;
pub mod backups;
pub mod business;
pub mod calendar;
pub mod clients;
pub mod reminders;
pub mod reports;
pub mod services;
pub mod session_plans;
pub mod settings;
pub mod updater;

use serde::Serialize;
use tauri::State;

use crate::database::health::{self, DatabaseHealth};
use crate::errors::AppResult;
use crate::platform::StorageMode;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}

/// Safe storage diagnostics DTO exposed to Settings/About screens.
#[derive(Debug, Serialize)]
pub struct StorageInfo {
    pub mode: StorageMode,
    pub data_dir: String,
    pub fallback_reason: Option<String>,
}

/// Minimal IPC command verifying Tauri command registration.
#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "My Schedule App",
        version: env!("CARGO_PKG_VERSION"),
    }
}

/// Returns the active storage mode (portable or fallback) and resolved data directory.
#[tauri::command]
pub fn get_storage_info(state: State<'_, AppState>) -> StorageInfo {
    StorageInfo {
        mode: state.storage.mode,
        data_dir: state.storage.paths.data_dir.to_string_lossy().into_owned(),
        fallback_reason: state.storage.fallback_reason.clone(),
    }
}

/// Runs a safe, read-only database integrity and diagnostics check.
#[tauri::command]
pub async fn get_database_health(state: State<'_, AppState>) -> AppResult<DatabaseHealth> {
    let pool = state.db.require_pool()?;
    health::check_health(&pool).await
}
