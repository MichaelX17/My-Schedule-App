use tauri::State;

use crate::errors::AppResult;
use crate::models::client::{
    Client, ClientListResult, CreateClientInput, ListClientsParams, UpdateClientInput,
};
use crate::services::client_service;
use crate::AppState;

/// Lists clients with search, archive filtering, sorting, and pagination.
#[tauri::command]
pub async fn list_clients(
    state: State<'_, AppState>,
    params: Option<ListClientsParams>,
) -> AppResult<ClientListResult> {
    let pool = state.db.require_pool()?;
    client_service::list_clients(&pool, params.unwrap_or_default()).await
}

/// Retrieves a single client by stable ID.
#[tauri::command]
pub async fn get_client(state: State<'_, AppState>, id: String) -> AppResult<Client> {
    let pool = state.db.require_pool()?;
    client_service::get_client(&pool, &id).await
}

/// Creates a new active client.
#[tauri::command]
pub async fn create_client(
    state: State<'_, AppState>,
    input: CreateClientInput,
) -> AppResult<Client> {
    let pool = state.db.require_pool()?;
    client_service::create_client(&pool, input).await
}

/// Updates an existing client's contact/profile details and notes.
#[tauri::command]
pub async fn update_client(
    state: State<'_, AppState>,
    input: UpdateClientInput,
) -> AppResult<Client> {
    let pool = state.db.require_pool()?;
    client_service::update_client(&pool, input).await
}

/// Archives a client without deleting historical records.
#[tauri::command]
pub async fn archive_client(state: State<'_, AppState>, id: String) -> AppResult<Client> {
    let pool = state.db.require_pool()?;
    client_service::archive_client(&pool, &id).await
}

/// Restores an archived client back to active status.
#[tauri::command]
pub async fn restore_client(state: State<'_, AppState>, id: String) -> AppResult<Client> {
    let pool = state.db.require_pool()?;
    client_service::restore_client(&pool, &id).await
}
