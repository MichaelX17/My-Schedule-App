use tauri::State;

use crate::backup::{self, BackupMetadata, BackupType, BackupValidationResult, RestoreResult};
use crate::errors::AppResult;
use crate::export::{self, CsvEntity, ExportResult};
use crate::AppState;

#[tauri::command]
pub async fn export_business_json(
    state: State<'_, AppState>,
    custom_filename: Option<String>,
) -> AppResult<ExportResult> {
    let pool = state.db.require_pool()?;
    export::export_json(
        &pool,
        &state.storage.paths.exports_dir,
        custom_filename.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn export_entity_csv(
    state: State<'_, AppState>,
    entity: CsvEntity,
    custom_filename: Option<String>,
) -> AppResult<ExportResult> {
    let pool = state.db.require_pool()?;
    export::export_csv(
        &pool,
        &state.storage.paths.exports_dir,
        entity,
        custom_filename.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn list_backups(state: State<'_, AppState>) -> AppResult<Vec<BackupMetadata>> {
    backup::list_backups(&state.storage.paths)
}

#[tauri::command]
pub async fn create_manual_backup(
    state: State<'_, AppState>,
    notes: Option<String>,
) -> AppResult<BackupMetadata> {
    let pool = state.db.require_pool()?;
    backup::create_backup(&pool, &state.storage.paths, BackupType::Manual, notes, None).await
}

#[tauri::command]
pub async fn validate_backup(
    state: State<'_, AppState>,
    backup_type: BackupType,
    filename: String,
) -> AppResult<BackupValidationResult> {
    let path = backup::resolve_managed_backup_path(&state.storage.paths, backup_type, &filename)?;
    backup::validate_backup_file(&path).await
}

#[tauri::command]
pub async fn restore_backup(
    state: State<'_, AppState>,
    backup_type: BackupType,
    filename: String,
) -> AppResult<RestoreResult> {
    let _guard = state.restore_lock.lock().await;
    backup::restore_backup(&state.db, &state.storage.paths, backup_type, &filename).await
}
