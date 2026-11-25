use tauri::State;

use crate::errors::AppResult;
use crate::models::session_plan::{
    CreateSessionPlanInput, CreateSessionPlanWithAppointmentsInput,
    CreateSessionPlanWithAppointmentsResult, ListSessionPlansParams, SessionPlanSummary,
    UpdateSessionPlanInput,
};
use crate::services::session_plan_service;
use crate::AppState;

#[tauri::command]
pub async fn list_session_plans(
    state: State<'_, AppState>,
    params: Option<ListSessionPlansParams>,
) -> AppResult<Vec<SessionPlanSummary>> {
    let pool = state.db.require_pool()?;
    session_plan_service::list_session_plans(&pool, params.unwrap_or_default()).await
}

#[tauri::command]
pub async fn get_session_plan(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<SessionPlanSummary> {
    let pool = state.db.require_pool()?;
    session_plan_service::get_session_plan(&pool, &id).await
}

#[tauri::command]
pub async fn create_session_plan(
    state: State<'_, AppState>,
    input: CreateSessionPlanInput,
) -> AppResult<SessionPlanSummary> {
    let pool = state.db.require_pool()?;
    session_plan_service::create_session_plan(&pool, input).await
}

#[tauri::command]
pub async fn create_session_plan_with_appointments(
    state: State<'_, AppState>,
    input: CreateSessionPlanWithAppointmentsInput,
) -> AppResult<CreateSessionPlanWithAppointmentsResult> {
    let pool = state.db.require_pool()?;
    session_plan_service::create_session_plan_with_appointments(&pool, input).await
}

#[tauri::command]
pub async fn update_session_plan(
    state: State<'_, AppState>,
    input: UpdateSessionPlanInput,
) -> AppResult<SessionPlanSummary> {
    let pool = state.db.require_pool()?;
    session_plan_service::update_session_plan(&pool, input).await
}

#[tauri::command]
pub async fn archive_session_plan(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<SessionPlanSummary> {
    let pool = state.db.require_pool()?;
    session_plan_service::archive_session_plan(&pool, &id).await
}

#[tauri::command]
pub async fn delete_session_plan(
    state: State<'_, AppState>,
    id: String,
    delete_linked_appointments: Option<bool>,
) -> AppResult<()> {
    let pool = state.db.require_pool()?;
    session_plan_service::delete_session_plan(
        &pool,
        &id,
        delete_linked_appointments.unwrap_or(false),
    )
    .await
}
