use std::path::Path;

use sqlx::SqlitePool;

use crate::backup;
use crate::errors::{AppError, AppResult};
use crate::models::settings::{AppSettings, UpdateAppSettingsInput};
use crate::time;

const VALID_THEMES: &[&str] = &["light", "dark", "system"];
const VALID_ACCENTS: &[&str] = &["slate", "blue", "emerald", "violet", "amber", "rose"];
pub const MIN_BACKUP_RETENTION: i64 = 1;
pub const MAX_BACKUP_RETENTION: i64 = 365;
pub const MIN_DEFAULT_DURATION: i64 = 5;
pub const MAX_DEFAULT_DURATION: i64 = 1440;

/// Loads the persisted singleton `app_settings` row (returning defaults if not yet seeded).
pub async fn get_settings(pool: &SqlitePool) -> AppResult<AppSettings> {
    let row = sqlx::query_as::<_, AppSettings>(
        r#"
        SELECT theme_mode, accent_color, reminders_enabled, reminder_one_day_before,
               reminder_one_hour_before, reminder_day_of_launch, os_notifications_enabled,
               auto_backup_enabled, auto_backup_retention,
               default_appointment_duration_minutes, updated_at
        FROM app_settings
        WHERE singleton_key = 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.unwrap_or_default())
}

/// Validates and persists updated application preferences. If `auto_backup_retention` is
/// updated and `automatic_backups_dir` is provided, immediately enforces the retention limit.
pub async fn update_settings(
    pool: &SqlitePool,
    automatic_backups_dir: Option<&Path>,
    input: UpdateAppSettingsInput,
) -> AppResult<AppSettings> {
    let current = get_settings(pool).await?;

    let theme_mode = match input.theme_mode {
        Some(raw) => {
            let norm = raw.trim().to_ascii_lowercase();
            if !VALID_THEMES.contains(&norm.as_str()) {
                return Err(AppError::Validation(
                    "Theme mode must be 'light', 'dark', or 'system'.".into(),
                ));
            }
            norm
        }
        None => current.theme_mode,
    };

    let accent_color = match input.accent_color {
        Some(raw) => {
            let norm = raw.trim().to_ascii_lowercase();
            if !VALID_ACCENTS.contains(&norm.as_str()) {
                return Err(AppError::Validation(
                    "Accent color must be one of: slate, blue, emerald, violet, amber, rose."
                        .into(),
                ));
            }
            norm
        }
        None => current.accent_color,
    };

    let auto_backup_retention = match input.auto_backup_retention {
        Some(val) => {
            if !(MIN_BACKUP_RETENTION..=MAX_BACKUP_RETENTION).contains(&val) {
                return Err(AppError::Validation(format!(
                    "Automatic backup retention must be between {MIN_BACKUP_RETENTION} and {MAX_BACKUP_RETENTION}."
                )));
            }
            val
        }
        None => current.auto_backup_retention,
    };

    let default_duration = match input.default_appointment_duration_minutes {
        Some(val) => {
            if !(MIN_DEFAULT_DURATION..=MAX_DEFAULT_DURATION).contains(&val) {
                return Err(AppError::Validation(format!(
                    "Default appointment duration must be between {MIN_DEFAULT_DURATION} and {MAX_DEFAULT_DURATION} minutes."
                )));
            }
            val
        }
        None => current.default_appointment_duration_minutes,
    };

    let reminders_enabled = input.reminders_enabled.unwrap_or(current.reminders_enabled);
    let reminder_one_day_before = input
        .reminder_one_day_before
        .unwrap_or(current.reminder_one_day_before);
    let reminder_one_hour_before = input
        .reminder_one_hour_before
        .unwrap_or(current.reminder_one_hour_before);
    let reminder_day_of_launch = input
        .reminder_day_of_launch
        .unwrap_or(current.reminder_day_of_launch);
    let os_notifications_enabled = input
        .os_notifications_enabled
        .unwrap_or(current.os_notifications_enabled);
    let auto_backup_enabled = input
        .auto_backup_enabled
        .unwrap_or(current.auto_backup_enabled);

    let now_iso = time::now_utc_iso();

    sqlx::query(
        r#"
        INSERT INTO app_settings (
            singleton_key, theme_mode, accent_color, reminders_enabled,
            reminder_one_day_before, reminder_one_hour_before, reminder_day_of_launch,
            os_notifications_enabled, auto_backup_enabled, auto_backup_retention,
            default_appointment_duration_minutes, updated_at
        )
        VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(singleton_key) DO UPDATE SET
            theme_mode = excluded.theme_mode,
            accent_color = excluded.accent_color,
            reminders_enabled = excluded.reminders_enabled,
            reminder_one_day_before = excluded.reminder_one_day_before,
            reminder_one_hour_before = excluded.reminder_one_hour_before,
            reminder_day_of_launch = excluded.reminder_day_of_launch,
            os_notifications_enabled = excluded.os_notifications_enabled,
            auto_backup_enabled = excluded.auto_backup_enabled,
            auto_backup_retention = excluded.auto_backup_retention,
            default_appointment_duration_minutes = excluded.default_appointment_duration_minutes,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&theme_mode)
    .bind(&accent_color)
    .bind(reminders_enabled)
    .bind(reminder_one_day_before)
    .bind(reminder_one_hour_before)
    .bind(reminder_day_of_launch)
    .bind(os_notifications_enabled)
    .bind(auto_backup_enabled)
    .bind(auto_backup_retention)
    .bind(default_duration)
    .bind(&now_iso)
    .execute(pool)
    .await?;

    if let Some(auto_dir) = automatic_backups_dir {
        let _ = backup::prune_automatic_backups(auto_dir, auto_backup_retention as usize);
    }

    tracing::info!(
        theme = %theme_mode,
        accent = %accent_color,
        auto_backup_enabled,
        auto_backup_retention,
        "Application settings updated"
    );

    get_settings(pool).await
}
