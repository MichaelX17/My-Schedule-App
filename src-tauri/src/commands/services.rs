use tauri::State;

use crate::errors::AppResult;
use crate::models::service::{
    CreateServiceInput, ListServicesParams, Service, ServiceListResult, UpdateServiceInput,
};
use crate::services::service_catalog_service;
use crate::AppState;

/// Lists offered services with search, status filter (defaults to active), and sorting.
#[tauri::command]
pub async fn list_services(
    state: State<'_, AppState>,
    params: Option<ListServicesParams>,
) -> AppResult<ServiceListResult> {
    let pool = state.db.require_pool()?;
    service_catalog_service::list_services(&pool, params.unwrap_or_default()).await
}

/// Retrieves a single service by ID (including inactive or archived services).
#[tauri::command]
pub async fn get_service(state: State<'_, AppState>, id: String) -> AppResult<Service> {
    let pool = state.db.require_pool()?;
    service_catalog_service::get_service(&pool, &id).await
}

/// Creates a new active service.
#[tauri::command]
pub async fn create_service(
    state: State<'_, AppState>,
    input: CreateServiceInput,
) -> AppResult<Service> {
    let pool = state.db.require_pool()?;
    service_catalog_service::create_service(&pool, input).await
}

/// Updates an existing service's name, description, and default duration.
#[tauri::command]
pub async fn update_service(
    state: State<'_, AppState>,
    input: UpdateServiceInput,
) -> AppResult<Service> {
    let pool = state.db.require_pool()?;
    service_catalog_service::update_service(&pool, input).await
}

/// Activates a non-archived service.
#[tauri::command]
pub async fn activate_service(state: State<'_, AppState>, id: String) -> AppResult<Service> {
    let pool = state.db.require_pool()?;
    service_catalog_service::activate_service(&pool, &id).await
}

/// Deactivates a service without archiving it.
#[tauri::command]
pub async fn deactivate_service(state: State<'_, AppState>, id: String) -> AppResult<Service> {
    let pool = state.db.require_pool()?;
    service_catalog_service::deactivate_service(&pool, &id).await
}

/// Archives a service while preserving historical records.
#[tauri::command]
pub async fn archive_service(state: State<'_, AppState>, id: String) -> AppResult<Service> {
    let pool = state.db.require_pool()?;
    service_catalog_service::archive_service(&pool, &id).await
}

/// Restores an archived service back to active status.
#[tauri::command]
pub async fn restore_service(state: State<'_, AppState>, id: String) -> AppResult<Service> {
    let pool = state.db.require_pool()?;
    service_catalog_service::restore_service(&pool, &id).await
}
