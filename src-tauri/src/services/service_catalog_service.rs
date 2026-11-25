use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::repositories::service_repo;
use crate::errors::{AppError, AppResult};
use crate::models::service::{
    CreateServiceInput, ListServicesParams, Service, ServiceListResult, UpdateServiceInput,
};
use crate::time;

pub const MIN_DURATION_MINUTES: i64 = 5;
pub const MAX_DURATION_MINUTES: i64 = 1440;
const MAX_NAME_LEN: usize = 120;
const MAX_DESCRIPTION_LEN: usize = 1000;

fn validate_name(raw: &str) -> AppResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Service name is required.".into()));
    }
    if trimmed.chars().count() > MAX_NAME_LEN {
        return Err(AppError::Validation(format!(
            "Service name must be {MAX_NAME_LEN} characters or fewer."
        )));
    }
    Ok(trimmed.to_string())
}

fn normalize_description(raw: Option<String>) -> AppResult<Option<String>> {
    let Some(value) = raw else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MAX_DESCRIPTION_LEN {
        return Err(AppError::Validation(format!(
            "Service description must be {MAX_DESCRIPTION_LEN} characters or fewer."
        )));
    }
    Ok(Some(trimmed.to_string()))
}

fn validate_duration(minutes: i64) -> AppResult<i64> {
    if !(MIN_DURATION_MINUTES..=MAX_DURATION_MINUTES).contains(&minutes) {
        return Err(AppError::Validation(format!(
            "Default duration must be between {MIN_DURATION_MINUTES} and {MAX_DURATION_MINUTES} minutes."
        )));
    }
    Ok(minutes)
}

/// Lists services with search, status filter (defaults to active), and sorting.
pub async fn list_services(
    pool: &SqlitePool,
    params: ListServicesParams,
) -> AppResult<ServiceListResult> {
    service_repo::list(pool, &params).await
}

/// Retrieves a single service by ID (including inactive or archived services).
pub async fn get_service(pool: &SqlitePool, id: &str) -> AppResult<Service> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Service ID is required.".into()));
    }
    service_repo::get_by_id(pool, trimmed)
        .await?
        .ok_or_else(|| AppError::NotFound("Service not found.".into()))
}

/// Validates and creates a new active service.
pub async fn create_service(pool: &SqlitePool, input: CreateServiceInput) -> AppResult<Service> {
    let name = validate_name(&input.name)?;
    let description = normalize_description(input.description)?;
    let default_duration_minutes = validate_duration(input.default_duration_minutes)?;

    let now = time::now_utc_iso();
    let service = Service {
        id: Uuid::new_v4().to_string(),
        name,
        description,
        default_duration_minutes,
        is_active: true,
        is_archived: false,
        archived_at: None,
        created_at: now.clone(),
        updated_at: now,
    };

    let created = service_repo::insert(pool, &service).await?;
    tracing::info!(service_id = %created.id, "Service created");
    Ok(created)
}

/// Validates and updates an existing service's name, description, and default duration.
pub async fn update_service(pool: &SqlitePool, input: UpdateServiceInput) -> AppResult<Service> {
    let existing = get_service(pool, &input.id).await?;

    let updated = Service {
        id: existing.id,
        name: validate_name(&input.name)?,
        description: normalize_description(input.description)?,
        default_duration_minutes: validate_duration(input.default_duration_minutes)?,
        is_active: existing.is_active,
        is_archived: existing.is_archived,
        archived_at: existing.archived_at,
        created_at: existing.created_at,
        updated_at: time::now_utc_iso(),
    };

    let saved = service_repo::update(pool, &updated).await?;
    tracing::info!(service_id = %saved.id, "Service updated");
    Ok(saved)
}

/// Activates a non-archived service so it is offered for new appointments.
pub async fn activate_service(pool: &SqlitePool, id: &str) -> AppResult<Service> {
    let existing = get_service(pool, id).await?;
    if existing.is_archived {
        return Err(AppError::Validation(
            "Archived services must be restored before they can be activated.".into(),
        ));
    }
    if existing.is_active {
        return Ok(existing);
    }

    let saved = service_repo::set_active(pool, &existing.id, true, &time::now_utc_iso()).await?;
    tracing::info!(service_id = %saved.id, "Service activated");
    Ok(saved)
}

/// Deactivates a service so it is excluded from new appointments while remaining in history.
pub async fn deactivate_service(pool: &SqlitePool, id: &str) -> AppResult<Service> {
    let existing = get_service(pool, id).await?;
    if existing.is_archived || !existing.is_active {
        return Ok(existing);
    }

    let saved = service_repo::set_active(pool, &existing.id, false, &time::now_utc_iso()).await?;
    tracing::info!(service_id = %saved.id, "Service deactivated");
    Ok(saved)
}

/// Archives a service (and deactivates it) while preserving historical references.
pub async fn archive_service(pool: &SqlitePool, id: &str) -> AppResult<Service> {
    let existing = get_service(pool, id).await?;
    if existing.is_archived {
        return Ok(existing);
    }

    let now = time::now_utc_iso();
    let saved =
        service_repo::set_archived(pool, &existing.id, true, false, Some(&now), &now).await?;
    tracing::info!(service_id = %saved.id, "Service archived");
    Ok(saved)
}

/// Restores an archived service back to active status.
pub async fn restore_service(pool: &SqlitePool, id: &str) -> AppResult<Service> {
    let existing = get_service(pool, id).await?;
    if !existing.is_archived {
        return Ok(existing);
    }

    let now = time::now_utc_iso();
    let saved = service_repo::set_archived(pool, &existing.id, false, true, None, &now).await?;
    tracing::info!(service_id = %saved.id, "Service restored");
    Ok(saved)
}
