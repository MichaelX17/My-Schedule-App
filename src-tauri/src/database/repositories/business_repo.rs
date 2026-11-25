use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::business::BusinessProfile;

const SELECT_COLUMNS: &str = "id, name, logo_filename, address, city, region, postal_code, country, phone, email, website, timezone, onboarding_completed, created_at, updated_at";

/// Retrieves the single business profile if setup has created one.
pub async fn get(pool: &SqlitePool) -> AppResult<Option<BusinessProfile>> {
    let query = format!("SELECT {SELECT_COLUMNS} FROM business_profile WHERE singleton_key = 1");
    let profile = sqlx::query_as::<_, BusinessProfile>(&query)
        .fetch_optional(pool)
        .await?;
    Ok(profile)
}

/// Inserts the single business profile. Enforces the database-level `singleton_key = 1`
/// unique constraint so a second business profile cannot be created.
pub async fn insert(pool: &SqlitePool, profile: &BusinessProfile) -> AppResult<BusinessProfile> {
    let result = sqlx::query(
        r#"
        INSERT INTO business_profile (
            id, singleton_key, name, logo_filename, address, city, region,
            postal_code, country, phone, email, website, timezone,
            onboarding_completed, created_at, updated_at
        )
        VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        "#,
    )
    .bind(&profile.id)
    .bind(&profile.name)
    .bind(&profile.logo_filename)
    .bind(&profile.address)
    .bind(&profile.city)
    .bind(&profile.region)
    .bind(&profile.postal_code)
    .bind(&profile.country)
    .bind(&profile.phone)
    .bind(&profile.email)
    .bind(&profile.website)
    .bind(&profile.timezone)
    .bind(if profile.onboarding_completed { 1 } else { 0 })
    .bind(&profile.created_at)
    .bind(&profile.updated_at)
    .execute(pool)
    .await;

    match result {
        Ok(_) => get(pool).await?.ok_or_else(|| {
            AppError::Internal("Inserted business profile could not be reloaded.".into())
        }),
        Err(sqlx::Error::Database(ref db_err))
            if db_err.is_unique_violation()
                || db_err.message().to_ascii_lowercase().contains("unique") =>
        {
            Err(AppError::AlreadyExists(
                "A business profile already exists for this installation.".into(),
            ))
        }
        Err(err) => Err(AppError::Database(err)),
    }
}

/// Updates the existing single business profile fields (excluding logo filename).
pub async fn update(pool: &SqlitePool, profile: &BusinessProfile) -> AppResult<BusinessProfile> {
    let result = sqlx::query(
        r#"
        UPDATE business_profile
        SET name = ?1,
            address = ?2,
            city = ?3,
            region = ?4,
            postal_code = ?5,
            country = ?6,
            phone = ?7,
            email = ?8,
            website = ?9,
            timezone = ?10,
            updated_at = ?11
        WHERE singleton_key = 1 AND id = ?12
        "#,
    )
    .bind(&profile.name)
    .bind(&profile.address)
    .bind(&profile.city)
    .bind(&profile.region)
    .bind(&profile.postal_code)
    .bind(&profile.country)
    .bind(&profile.phone)
    .bind(&profile.email)
    .bind(&profile.website)
    .bind(&profile.timezone)
    .bind(&profile.updated_at)
    .bind(&profile.id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Business profile does not exist.".into(),
        ));
    }

    get(pool)
        .await?
        .ok_or_else(|| AppError::Internal("Updated business profile could not be reloaded.".into()))
}

/// Updates only the managed `logo_filename` reference on the business profile.
pub async fn update_logo_filename(
    pool: &SqlitePool,
    id: &str,
    logo_filename: Option<&str>,
    updated_at: &str,
) -> AppResult<BusinessProfile> {
    let result = sqlx::query(
        r#"
        UPDATE business_profile
        SET logo_filename = ?1,
            updated_at = ?2
        WHERE singleton_key = 1 AND id = ?3
        "#,
    )
    .bind(logo_filename)
    .bind(updated_at)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Business profile does not exist.".into(),
        ));
    }

    get(pool)
        .await?
        .ok_or_else(|| AppError::Internal("Updated business profile could not be reloaded.".into()))
}
