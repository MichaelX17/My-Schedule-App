use std::fs;
use std::path::Path;

use base64::Engine;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::repositories::business_repo;
use crate::errors::{AppError, AppResult};
use crate::models::business::{
    BusinessLogoData, BusinessProfile, CreateBusinessProfileInput, OnboardingStatus,
    UpdateBusinessProfileInput,
};
use crate::platform;
use crate::time;

/// Maximum allowed logo file size (2 MiB).
pub const MAX_LOGO_SIZE_BYTES: usize = 2 * 1024 * 1024;
const MAX_NAME_LEN: usize = 120;
const MAX_FIELD_LEN: usize = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DetectedImageFormat {
    extension: &'static str,
    mime_type: &'static str,
}

/// Validates and detects supported raster image formats (PNG, JPEG, WebP) from magic bytes.
/// Explicitly rejects SVG and unknown formats.
fn detect_supported_image(bytes: &[u8]) -> AppResult<DetectedImageFormat> {
    if bytes.is_empty() {
        return Err(AppError::Validation("Logo image file is empty.".into()));
    }
    if bytes.len() > MAX_LOGO_SIZE_BYTES {
        return Err(AppError::Validation(
            "Logo image exceeds the maximum allowed size of 2 MB.".into(),
        ));
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if bytes.len() >= 8 && bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok(DetectedImageFormat {
            extension: "png",
            mime_type: "image/png",
        });
    }

    // JPEG: FF D8 FF
    if bytes.len() >= 3 && bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(DetectedImageFormat {
            extension: "jpg",
            mime_type: "image/jpeg",
        });
    }

    // WebP: "RIFF" + 4 bytes + "WEBP"
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok(DetectedImageFormat {
            extension: "webp",
            mime_type: "image/webp",
        });
    }

    Err(AppError::Validation(
        "Unsupported logo format. Please choose a PNG, JPEG, or WebP image.".into(),
    ))
}

fn validate_business_name(name: &str) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Business name is required.".into()));
    }
    if trimmed.chars().count() > MAX_NAME_LEN {
        return Err(AppError::Validation(format!(
            "Business name must be {MAX_NAME_LEN} characters or fewer."
        )));
    }
    Ok(trimmed.to_string())
}

fn normalize_optional_field(value: Option<String>, field_label: &str) -> AppResult<Option<String>> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MAX_FIELD_LEN {
        return Err(AppError::Validation(format!(
            "{field_label} must be {MAX_FIELD_LEN} characters or fewer."
        )));
    }
    Ok(Some(trimmed.to_string()))
}

fn normalize_optional_email(value: Option<String>) -> AppResult<Option<String>> {
    let Some(email) = normalize_optional_field(value, "Email")? else {
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

/// Persists validated logo bytes atomically into `logos_dir` and returns the managed filename.
fn write_managed_logo(logos_dir: &Path, bytes: &[u8]) -> AppResult<String> {
    let fmt = detect_supported_image(bytes)?;
    fs::create_dir_all(logos_dir)?;

    let filename = format!("logo_{}.{}", Uuid::new_v4(), fmt.extension);
    let target_path = platform::safe_join_child(logos_dir, &filename)?;
    let tmp_name = format!(".tmp_{filename}");
    let tmp_path = platform::safe_join_child(logos_dir, &tmp_name)?;

    fs::write(&tmp_path, bytes)?;
    if let Err(err) = fs::rename(&tmp_path, &target_path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(AppError::Io(err));
    }

    Ok(filename)
}

fn remove_managed_logo_file(logos_dir: &Path, filename: &str) {
    if let Ok(path) = platform::safe_join_child(logos_dir, filename) {
        let _ = fs::remove_file(path);
    }
}

/// Returns whether onboarding is complete, the existing business profile (if any),
/// and the OS-detected IANA timezone identifier.
pub async fn get_onboarding_status(pool: &SqlitePool) -> AppResult<OnboardingStatus> {
    let business = business_repo::get(pool).await?;
    let completed = business
        .as_ref()
        .is_some_and(|profile| profile.onboarding_completed);

    Ok(OnboardingStatus {
        completed,
        business,
        detected_timezone: time::detect_system_timezone(),
    })
}

/// Retrieves the single business profile if it exists.
pub async fn get_business_profile(pool: &SqlitePool) -> AppResult<Option<BusinessProfile>> {
    business_repo::get(pool).await
}

/// Creates the initial single business profile and marks onboarding complete.
/// Enforces the one-business-per-installation invariant.
pub async fn create_business_profile(
    pool: &SqlitePool,
    logos_dir: &Path,
    input: CreateBusinessProfileInput,
) -> AppResult<BusinessProfile> {
    if business_repo::get(pool).await?.is_some() {
        return Err(AppError::AlreadyExists(
            "A business profile already exists for this installation.".into(),
        ));
    }

    let name = validate_business_name(&input.name)?;
    let timezone = time::validate_iana_timezone(&input.timezone)?;
    let address = normalize_optional_field(input.address, "Address")?;
    let city = normalize_optional_field(input.city, "City")?;
    let region = normalize_optional_field(input.region, "Region")?;
    let postal_code = normalize_optional_field(input.postal_code, "Postal code")?;
    let country = normalize_optional_field(input.country, "Country")?;
    let phone = normalize_optional_field(input.phone, "Phone")?;
    let email = normalize_optional_email(input.email)?;
    let website = normalize_optional_field(input.website, "Website")?;

    let logo_filename = match input.logo_bytes {
        Some(bytes) => Some(write_managed_logo(logos_dir, &bytes)?),
        None => None,
    };

    let now = time::now_utc_iso();
    let profile = BusinessProfile {
        id: Uuid::new_v4().to_string(),
        name,
        logo_filename: logo_filename.clone(),
        address,
        city,
        region,
        postal_code,
        country,
        phone,
        email,
        website,
        timezone,
        onboarding_completed: true,
        created_at: now.clone(),
        updated_at: now,
    };

    match business_repo::insert(pool, &profile).await {
        Ok(created) => {
            tracing::info!(business_id = %created.id, timezone = %created.timezone, "Business profile created");
            Ok(created)
        }
        Err(err) => {
            if let Some(ref filename) = logo_filename {
                remove_managed_logo_file(logos_dir, filename);
            }
            Err(err)
        }
    }
}

/// Updates the existing single business profile.
pub async fn update_business_profile(
    pool: &SqlitePool,
    input: UpdateBusinessProfileInput,
) -> AppResult<BusinessProfile> {
    let existing = business_repo::get(pool).await?.ok_or_else(|| {
        AppError::NotFound("Business profile has not been initialized yet.".into())
    })?;

    let updated = BusinessProfile {
        id: existing.id,
        name: validate_business_name(&input.name)?,
        logo_filename: existing.logo_filename,
        address: normalize_optional_field(input.address, "Address")?,
        city: normalize_optional_field(input.city, "City")?,
        region: normalize_optional_field(input.region, "Region")?,
        postal_code: normalize_optional_field(input.postal_code, "Postal code")?,
        country: normalize_optional_field(input.country, "Country")?,
        phone: normalize_optional_field(input.phone, "Phone")?,
        email: normalize_optional_email(input.email)?,
        website: normalize_optional_field(input.website, "Website")?,
        timezone: time::validate_iana_timezone(&input.timezone)?,
        onboarding_completed: existing.onboarding_completed,
        created_at: existing.created_at,
        updated_at: time::now_utc_iso(),
    };

    let previous_timezone = existing.timezone.clone();
    let saved = business_repo::update(pool, &updated).await?;
    if saved.timezone != previous_timezone {
        crate::scheduler::recalculate_all_scheduled_reminders(pool).await?;
    }
    tracing::info!(business_id = %saved.id, timezone = %saved.timezone, "Business profile updated");
    Ok(saved)
}

/// Uploads and replaces the business logo inside managed storage, updating `business_profile`.
pub async fn upload_business_logo(
    pool: &SqlitePool,
    logos_dir: &Path,
    bytes: Vec<u8>,
) -> AppResult<BusinessLogoData> {
    let existing = business_repo::get(pool).await?.ok_or_else(|| {
        AppError::NotFound("Business profile has not been initialized yet.".into())
    })?;

    let fmt = detect_supported_image(&bytes)?;
    let new_filename = write_managed_logo(logos_dir, &bytes)?;
    let updated_at = time::now_utc_iso();

    match business_repo::update_logo_filename(pool, &existing.id, Some(&new_filename), &updated_at)
        .await
    {
        Ok(_) => {
            if let Some(old_filename) = existing.logo_filename {
                if old_filename != new_filename {
                    remove_managed_logo_file(logos_dir, &old_filename);
                }
            }
            let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
            Ok(BusinessLogoData {
                filename: new_filename,
                mime_type: fmt.mime_type.to_string(),
                data_url: format!("data:{};base64,{encoded}", fmt.mime_type),
            })
        }
        Err(err) => {
            remove_managed_logo_file(logos_dir, &new_filename);
            Err(err)
        }
    }
}

/// Reads the managed business logo as a data URL for safe frontend display.
pub async fn get_business_logo(
    pool: &SqlitePool,
    logos_dir: &Path,
) -> AppResult<Option<BusinessLogoData>> {
    let Some(profile) = business_repo::get(pool).await? else {
        return Ok(None);
    };
    let Some(filename) = profile.logo_filename else {
        return Ok(None);
    };

    let path = platform::safe_join_child(logos_dir, &filename)?;
    if !path.exists() {
        tracing::warn!(filename = %filename, "Managed logo file is missing from storage");
        return Ok(None);
    }

    let bytes = fs::read(&path)?;
    let fmt = detect_supported_image(&bytes)?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);

    Ok(Some(BusinessLogoData {
        filename,
        mime_type: fmt.mime_type.to_string(),
        data_url: format!("data:{};base64,{encoded}", fmt.mime_type),
    }))
}

/// Removes the business logo reference and deletes the managed file from storage.
pub async fn remove_business_logo(
    pool: &SqlitePool,
    logos_dir: &Path,
) -> AppResult<BusinessProfile> {
    let existing = business_repo::get(pool).await?.ok_or_else(|| {
        AppError::NotFound("Business profile has not been initialized yet.".into())
    })?;

    let updated =
        business_repo::update_logo_filename(pool, &existing.id, None, &time::now_utc_iso()).await?;

    if let Some(old_filename) = existing.logo_filename {
        remove_managed_logo_file(logos_dir, &old_filename);
    }

    Ok(updated)
}
