use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::errors::{AppError, AppResult};
use crate::models::service::{
    ListServicesParams, Service, ServiceListResult, ServiceSortBy, ServiceStatusFilter,
};

const SELECT_COLUMNS: &str = "id, name, description, default_duration_minutes, is_active, is_archived, archived_at, created_at, updated_at";

fn escape_like_pattern(raw: &str) -> String {
    let mut escaped = String::with_capacity(raw.len() + 2);
    escaped.push('%');
    for ch in raw.chars() {
        match ch {
            '%' | '_' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped.push('%');
    escaped
}

fn push_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    filter: ServiceStatusFilter,
    search_pattern: Option<String>,
) {
    let mut has_where = false;
    match filter {
        ServiceStatusFilter::Active => {
            builder.push(" WHERE is_archived = 0 AND is_active = 1");
            has_where = true;
        }
        ServiceStatusFilter::Inactive => {
            builder.push(" WHERE is_archived = 0 AND is_active = 0");
            has_where = true;
        }
        ServiceStatusFilter::Archived => {
            builder.push(" WHERE is_archived = 1");
            has_where = true;
        }
        ServiceStatusFilter::All => {}
    }

    if let Some(pattern) = search_pattern {
        if has_where {
            builder.push(" AND ");
        } else {
            builder.push(" WHERE ");
        }
        builder.push("(name LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" ESCAPE '\\' OR description LIKE ");
        builder.push_bind(pattern);
        builder.push(" ESCAPE '\\')");
    }
}

/// Lists services with search, status filtering, and sorting.
pub async fn list(pool: &SqlitePool, params: &ListServicesParams) -> AppResult<ServiceListResult> {
    let filter = params.filter.unwrap_or_default();
    let sort_by = params.sort_by.unwrap_or_default();
    let search_pattern = params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(escape_like_pattern);

    let active_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM services WHERE is_archived = 0 AND is_active = 1")
            .fetch_one(pool)
            .await?;
    let inactive_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM services WHERE is_archived = 0 AND is_active = 0")
            .fetch_one(pool)
            .await?;
    let archived_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM services WHERE is_archived = 1")
            .fetch_one(pool)
            .await?;

    let mut qb: QueryBuilder<'_, Sqlite> =
        QueryBuilder::new(format!("SELECT {SELECT_COLUMNS} FROM services"));
    push_filters(&mut qb, filter, search_pattern);

    let order_clause = match sort_by {
        ServiceSortBy::NameAsc => " ORDER BY name COLLATE NOCASE ASC, created_at DESC",
        ServiceSortBy::NameDesc => " ORDER BY name COLLATE NOCASE DESC, created_at DESC",
        ServiceSortBy::DurationAsc => {
            " ORDER BY default_duration_minutes ASC, name COLLATE NOCASE ASC"
        }
        ServiceSortBy::DurationDesc => {
            " ORDER BY default_duration_minutes DESC, name COLLATE NOCASE ASC"
        }
        ServiceSortBy::RecentlyUpdated => " ORDER BY updated_at DESC, name COLLATE NOCASE ASC",
    };
    qb.push(order_clause);

    let items = qb.build_query_as::<Service>().fetch_all(pool).await?;
    let total = items.len() as i64;

    Ok(ServiceListResult {
        items,
        total,
        active_count,
        inactive_count,
        archived_count,
    })
}

/// Fetches a single service by ID (including inactive or archived services for history).
pub async fn get_by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<Service>> {
    let query = format!("SELECT {SELECT_COLUMNS} FROM services WHERE id = ?1");
    let service = sqlx::query_as::<_, Service>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(service)
}

/// Inserts a new service record.
pub async fn insert(pool: &SqlitePool, service: &Service) -> AppResult<Service> {
    sqlx::query(
        r#"
        INSERT INTO services (
            id, name, description, default_duration_minutes,
            is_active, is_archived, archived_at, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
    )
    .bind(&service.id)
    .bind(&service.name)
    .bind(&service.description)
    .bind(service.default_duration_minutes)
    .bind(if service.is_active { 1 } else { 0 })
    .bind(if service.is_archived { 1 } else { 0 })
    .bind(&service.archived_at)
    .bind(&service.created_at)
    .bind(&service.updated_at)
    .execute(pool)
    .await?;

    get_by_id(pool, &service.id)
        .await?
        .ok_or_else(|| AppError::Internal("Inserted service could not be reloaded.".into()))
}

/// Updates an existing service's editable fields.
pub async fn update(pool: &SqlitePool, service: &Service) -> AppResult<Service> {
    let result = sqlx::query(
        r#"
        UPDATE services
        SET name = ?1,
            description = ?2,
            default_duration_minutes = ?3,
            updated_at = ?4
        WHERE id = ?5
        "#,
    )
    .bind(&service.name)
    .bind(&service.description)
    .bind(service.default_duration_minutes)
    .bind(&service.updated_at)
    .bind(&service.id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Service not found.".into()));
    }

    get_by_id(pool, &service.id)
        .await?
        .ok_or_else(|| AppError::Internal("Updated service could not be reloaded.".into()))
}

/// Updates active/inactive flag for a non-archived service.
pub async fn set_active(
    pool: &SqlitePool,
    id: &str,
    is_active: bool,
    updated_at: &str,
) -> AppResult<Service> {
    let result = sqlx::query(
        r#"
        UPDATE services
        SET is_active = ?1,
            updated_at = ?2
        WHERE id = ?3
        "#,
    )
    .bind(if is_active { 1 } else { 0 })
    .bind(updated_at)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Service not found.".into()));
    }

    get_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("Updated service could not be reloaded.".into()))
}

/// Updates archive state (archiving also deactivates; restoring reactivates).
pub async fn set_archived(
    pool: &SqlitePool,
    id: &str,
    is_archived: bool,
    is_active: bool,
    archived_at: Option<&str>,
    updated_at: &str,
) -> AppResult<Service> {
    let result = sqlx::query(
        r#"
        UPDATE services
        SET is_archived = ?1,
            is_active = ?2,
            archived_at = ?3,
            updated_at = ?4
        WHERE id = ?5
        "#,
    )
    .bind(if is_archived { 1 } else { 0 })
    .bind(if is_active { 1 } else { 0 })
    .bind(archived_at)
    .bind(updated_at)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Service not found.".into()));
    }

    get_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("Updated service could not be reloaded.".into()))
}

/// Counts appointment-service snapshot rows referencing a service.
pub async fn count_dependent_appointments(pool: &SqlitePool, id: &str) -> AppResult<i64> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM appointment_services WHERE service_id = ?1")
            .bind(id)
            .fetch_one(pool)
            .await?;

    Ok(count)
}

/// Permanently deletes a service record by ID.
pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM services WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Service not found.".into()));
    }

    Ok(())
}
