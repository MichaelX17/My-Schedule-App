use tauri::State;

use crate::errors::AppResult;
use crate::models::calendar::{CalendarRangeInput, CalendarRangeResponse};
use crate::services::calendar_service;
use crate::AppState;

#[tauri::command]
pub async fn get_calendar_range(
    state: State<'_, AppState>,
    input: Option<CalendarRangeInput>,
) -> AppResult<CalendarRangeResponse> {
    let pool = state.db.require_pool()?;
    calendar_service::get_calendar_range(&pool, input.unwrap_or_default()).await
}
