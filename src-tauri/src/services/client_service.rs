use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::repositories::client_repo;
use crate::errors::{AppError, AppResult};
use crate::models::client::{
    Client, ClientListResult, CreateClientInput, ListClientsParams, UpdateClientInput,
};
use crate::time;

const MAX_FULL_NAME_LEN: usize = 150;
const MAX_PHONE_LEN: usize = 50;
const MAX_EMAIL_LEN: usize = 255;
const MAX_ADDRESS_LEN: usize = 255;
const MAX_NOTES_LEN: usize = 4000;

fn validate_full_name(raw: &str) -> AppResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Client name is required.".into()));
    }
    if trimmed.chars().count() > MAX_FULL_NAME_LEN {
        return Err(AppError::Validation(format!(
            "Client name must be {MAX_FULL_NAME_LEN} characters or fewer."
        )));
    }
    Ok(trimmed.to_string())
}

fn normalize_optional(
    raw: Option<String>,
    label: &str,
    max_len: usize,
) -> AppResult<Option<String>> {
    let Some(value) = raw else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > max_len {
        return Err(AppError::Validation(format!(
            "{label} must be {max_len} characters or fewer."
        )));
    }
    Ok(Some(trimmed.to_string()))
}

fn normalize_optional_email(raw: Option<String>) -> AppResult<Option<String>> {
    let Some(email) = normalize_optional(raw, "Email", MAX_EMAIL_LEN)? else {
        return Ok(None);
    };
    let mut parts = email.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    if local.is_empty()
        || domain.is_empty()
        || !domain.contains('.')
        || parts.next().is_some()
        || email.chars().any(char::is_whitespace)
    {
        return Err(AppError::Validation(
            "Please enter a valid email address or leave it blank.".into(),
        ));
    }
    Ok(Some(email))
}

/// Lists clients with search, archive filter, sorting, and pagination.
pub async fn list_clients(
    pool: &SqlitePool,
    params: ListClientsParams,
) -> AppResult<ClientListResult> {
    client_repo::list(pool, &params).await
}

/// Retrieves a single client by ID, returning `NotFound` if it does not exist.
pub async fn get_client(pool: &SqlitePool, id: &str) -> AppResult<Client> {
    let trimmed_id = id.trim();
    if trimmed_id.is_empty() {
        return Err(AppError::Validation("Client ID is required.".into()));
    }
    client_repo::get_by_id(pool, trimmed_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Client not found.".into()))
}

/// Validates input and constructs a new active `Client` ready for insertion.
pub(crate) fn build_validated_client(input: CreateClientInput) -> AppResult<Client> {
    let full_name = validate_full_name(&input.full_name)?;
    let phone = normalize_optional(input.phone, "Phone", MAX_PHONE_LEN)?;
    let email = normalize_optional_email(input.email)?;
    let address = normalize_optional(input.address, "Address", MAX_ADDRESS_LEN)?;
    let notes = normalize_optional(input.notes, "Notes", MAX_NOTES_LEN)?;

    let now = time::now_utc_iso();
    Ok(Client {
        id: Uuid::new_v4().to_string(),
        full_name,
        phone,
        email,
        address,
        notes,
        is_archived: false,
        archived_at: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// Validates input and creates a new active client.
pub async fn create_client(pool: &SqlitePool, input: CreateClientInput) -> AppResult<Client> {
    let client = build_validated_client(input)?;
    let created = client_repo::insert(pool, &client).await?;
    tracing::info!(client_id = %created.id, "Client created");
    Ok(created)
}

/// Validates input and updates an existing client's profile fields.
pub async fn update_client(pool: &SqlitePool, input: UpdateClientInput) -> AppResult<Client> {
    let existing = get_client(pool, &input.id).await?;

    let updated = Client {
        id: existing.id,
        full_name: validate_full_name(&input.full_name)?,
        phone: normalize_optional(input.phone, "Phone", MAX_PHONE_LEN)?,
        email: normalize_optional_email(input.email)?,
        address: normalize_optional(input.address, "Address", MAX_ADDRESS_LEN)?,
        notes: normalize_optional(input.notes, "Notes", MAX_NOTES_LEN)?,
        is_archived: existing.is_archived,
        archived_at: existing.archived_at,
        created_at: existing.created_at,
        updated_at: time::now_utc_iso(),
    };

    let saved = client_repo::update(pool, &updated).await?;
    tracing::info!(client_id = %saved.id, "Client updated");
    Ok(saved)
}

/// Archives a client while preserving all dependent history.
pub async fn archive_client(pool: &SqlitePool, id: &str) -> AppResult<Client> {
    let existing = get_client(pool, id).await?;
    if existing.is_archived {
        return Ok(existing);
    }

    let now = time::now_utc_iso();
    let archived = client_repo::set_archived(pool, &existing.id, true, Some(&now), &now).await?;
    tracing::info!(client_id = %archived.id, "Client archived");
    Ok(archived)
}

/// Restores an archived client back to active status.
pub async fn restore_client(pool: &SqlitePool, id: &str) -> AppResult<Client> {
    let existing = get_client(pool, id).await?;
    if !existing.is_archived {
        return Ok(existing);
    }

    let now = time::now_utc_iso();
    let restored = client_repo::set_archived(pool, &existing.id, false, None, &now).await?;
    tracing::info!(client_id = %restored.id, "Client restored");
    Ok(restored)
}
