use tauri::{AppHandle, Emitter, State};

use crate::errors::AppResult;
use crate::models::reminder::{NotificationItem, ReminderRunSummary, ReminderSummary};
use crate::notifications::TauriNotificationSender;
use crate::services::reminder_service;
use crate::AppState;

#[tauri::command]
pub async fn list_notifications(
    state: State<'_, AppState>,
    include_dismissed: Option<bool>,
) -> AppResult<Vec<NotificationItem>> {
    let pool = state.db.require_pool()?;
    reminder_service::list_notifications(&pool, include_dismissed.unwrap_or(false)).await
}

#[tauri::command]
pub async fn get_unread_notification_count(state: State<'_, AppState>) -> AppResult<i64> {
    let pool = state.db.require_pool()?;
    reminder_service::get_unread_count(&pool).await
}

#[tauri::command]
pub async fn mark_notification_read(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<NotificationItem> {
    let pool = state.db.require_pool()?;
    reminder_service::mark_notification_read(&pool, &id).await
}

#[tauri::command]
pub async fn mark_all_notifications_read(state: State<'_, AppState>) -> AppResult<u64> {
    let pool = state.db.require_pool()?;
    reminder_service::mark_all_notifications_read(&pool).await
}

#[tauri::command]
pub async fn dismiss_notification(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<NotificationItem> {
    let pool = state.db.require_pool()?;
    reminder_service::dismiss_notification(&pool, &id).await
}

#[tauri::command]
pub async fn list_upcoming_reminders(
    state: State<'_, AppState>,
) -> AppResult<Vec<ReminderSummary>> {
    let pool = state.db.require_pool()?;
    reminder_service::list_upcoming_reminders(&pool).await
}

#[tauri::command]
pub async fn run_reminder_check_now(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<ReminderRunSummary> {
    let pool = state.db.require_pool()?;
    let sender = TauriNotificationSender::new(app.clone());
    let summary = reminder_service::run_reminder_check_now(&pool, &sender).await?;
    if summary.total_new_notifications() > 0 {
        let _ = app.emit("reminders-updated", &summary);
    }
    Ok(summary)
}
