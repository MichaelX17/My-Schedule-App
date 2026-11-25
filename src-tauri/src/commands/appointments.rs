use tauri::State;

use crate::errors::AppResult;
use crate::models::appointment::{
    AppointmentDetail, AppointmentMutationResult, CheckOverlapsInput, CreateAppointmentInput,
    ListAppointmentsParams, OverlapWarning, RescheduleAppointmentInput, UpdateAppointmentInput,
};
use crate::services::appointment_service;
use crate::AppState;

#[tauri::command]
pub async fn list_appointments(
    state: State<'_, AppState>,
    params: Option<ListAppointmentsParams>,
) -> AppResult<Vec<AppointmentDetail>> {
    let pool = state.db.require_pool()?;
    appointment_service::list_appointments(&pool, params.unwrap_or_default()).await
}

#[tauri::command]
pub async fn get_appointment(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<AppointmentDetail> {
    let pool = state.db.require_pool()?;
    appointment_service::get_appointment(&pool, &id).await
}

#[tauri::command]
pub async fn check_appointment_overlaps(
    state: State<'_, AppState>,
    input: CheckOverlapsInput,
) -> AppResult<Vec<OverlapWarning>> {
    let pool = state.db.require_pool()?;
    appointment_service::check_overlaps(&pool, input).await
}

#[tauri::command]
pub async fn create_appointment(
    state: State<'_, AppState>,
    input: CreateAppointmentInput,
) -> AppResult<AppointmentMutationResult> {
    let pool = state.db.require_pool()?;
    appointment_service::create_appointment(&pool, input).await
}

#[tauri::command]
pub async fn update_appointment(
    state: State<'_, AppState>,
    input: UpdateAppointmentInput,
) -> AppResult<AppointmentMutationResult> {
    let pool = state.db.require_pool()?;
    appointment_service::update_appointment(&pool, input).await
}

#[tauri::command]
pub async fn reschedule_appointment(
    state: State<'_, AppState>,
    input: RescheduleAppointmentInput,
) -> AppResult<AppointmentMutationResult> {
    let pool = state.db.require_pool()?;
    appointment_service::reschedule_appointment(&pool, input).await
}

#[tauri::command]
pub async fn complete_appointment(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<AppointmentDetail> {
    let pool = state.db.require_pool()?;
    appointment_service::complete_appointment(&pool, &id).await
}

#[tauri::command]
pub async fn cancel_appointment(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<AppointmentDetail> {
    let pool = state.db.require_pool()?;
    appointment_service::cancel_appointment(&pool, &id).await
}
