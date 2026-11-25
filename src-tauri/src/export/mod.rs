use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::appointment::AppointmentServiceItem;
use crate::models::client::Client;
use crate::models::service::Service;
use crate::platform;
use crate::time;

/// Current version of the JSON business data export schema.
pub const EXPORT_SCHEMA_VERSION: u32 = 1;

/// Supported CSV export entity types.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CsvEntity {
    Clients,
    Services,
    Appointments,
    AppointmentServices,
}

impl CsvEntity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clients => "clients",
            Self::Services => "services",
            Self::Appointments => "appointments",
            Self::AppointmentServices => "appointment_services",
        }
    }
}

/// Result metadata returned to the UI after a successful JSON or CSV export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportResult {
    pub format: String,
    pub filename: String,
    pub file_path: String,
    pub size_bytes: u64,
    pub record_count: usize,
    pub exported_at_utc: String,
}

/// Sanitized business profile representation for JSON export (omits internal filesystem filenames).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportBusinessProfile {
    pub id: String,
    pub name: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub timezone: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportSessionPlan {
    pub id: String,
    pub client_id: String,
    pub title: String,
    pub description: Option<String>,
    pub planned_sessions: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportAppointment {
    pub id: String,
    pub client_id: String,
    pub session_plan_id: Option<String>,
    pub start_utc: String,
    pub end_utc: String,
    pub timezone: String,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Top-level versioned JSON export document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedBusinessExport {
    pub export_schema_version: u32,
    pub app_name: String,
    pub app_version: String,
    pub exported_at_utc: String,
    pub business_profile: Option<ExportBusinessProfile>,
    pub clients: Vec<Client>,
    pub services: Vec<Service>,
    pub session_plans: Vec<ExportSessionPlan>,
    pub appointments: Vec<ExportAppointment>,
    pub appointment_services: Vec<AppointmentServiceItem>,
}

/// Escapes a single CSV field per RFC 4180 (quotes fields containing `,`, `"`, `\n`, or `\r`).
pub fn escape_csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        let mut out = String::with_capacity(value.len() + 2);
        out.push('"');
        for ch in value.chars() {
            if ch == '"' {
                out.push('"');
            }
            out.push(ch);
        }
        out.push('"');
        out
    } else {
        value.to_string()
    }
}

fn format_csv_row(fields: &[&str]) -> String {
    let mut line = fields
        .iter()
        .map(|f| escape_csv_field(f))
        .collect::<Vec<_>>()
        .join(",");
    line.push_str("\r\n");
    line
}

fn resolve_export_filename(
    prefix: &str,
    ext: &str,
    custom_filename: Option<&str>,
) -> AppResult<String> {
    if let Some(raw) = custom_filename.map(str::trim).filter(|s| !s.is_empty()) {
        if raw.contains('/') || raw.contains('\\') || raw.contains("..") {
            return Err(AppError::Validation(
                "Export filename must not contain path separators.".into(),
            ));
        }
        let lower = raw.to_ascii_lowercase();
        let suffix = format!(".{ext}");
        if lower.ends_with(&suffix) {
            Ok(raw.to_string())
        } else {
            Ok(format!("{raw}.{ext}"))
        }
    } else {
        let stamp = Utc::now().format("%Y%m%d_%H%M%S");
        Ok(format!("{prefix}_{stamp}.{ext}"))
    }
}

fn write_file_atomically(dir: &Path, filename: &str, contents: &[u8]) -> AppResult<(String, u64)> {
    fs::create_dir_all(dir)?;
    let target_path = platform::safe_join_child(dir, filename)?;
    let tmp_name = format!(".{filename}.{}.tmp", Uuid::new_v4());
    let tmp_path = platform::safe_join_child(dir, &tmp_name)?;

    let write_res = (|| -> std::io::Result<()> {
        let mut file = File::create(&tmp_path)?;
        file.write_all(contents)?;
        file.sync_all()?;
        Ok(())
    })();

    if let Err(err) = write_res {
        let _ = fs::remove_file(&tmp_path);
        return Err(AppError::Io(err));
    }

    if let Err(err) = fs::rename(&tmp_path, &target_path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(AppError::Io(err));
    }

    Ok((
        target_path.to_string_lossy().into_owned(),
        contents.len() as u64,
    ))
}

/// Exports all business data to a versioned, sanitized JSON file in `exports_dir`.
pub async fn export_json(
    pool: &SqlitePool,
    exports_dir: &Path,
    custom_filename: Option<&str>,
) -> AppResult<ExportResult> {
    let business_profile = sqlx::query_as::<_, ExportBusinessProfile>(
        r#"
        SELECT id, name, address, city, region, postal_code, country,
               phone, email, website, timezone, created_at, updated_at
        FROM business_profile
        WHERE singleton_key = 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    let clients = sqlx::query_as::<_, Client>(
        r#"
        SELECT id, full_name, phone, email, address, notes,
               is_archived, archived_at, created_at, updated_at
        FROM clients
        ORDER BY full_name ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let services = sqlx::query_as::<_, Service>(
        r#"
        SELECT id, name, description, default_duration_minutes,
               is_active, is_archived, archived_at, created_at, updated_at
        FROM services
        ORDER BY name ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let session_plans = sqlx::query_as::<_, ExportSessionPlan>(
        r#"
        SELECT id, client_id, title, description, planned_sessions,
               status, created_at, updated_at
        FROM session_plans
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let appointments = sqlx::query_as::<_, ExportAppointment>(
        r#"
        SELECT id, client_id, session_plan_id, start_utc, end_utc,
               timezone, status, notes, created_at, updated_at
        FROM appointments
        ORDER BY start_utc ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let appointment_services = sqlx::query_as::<_, AppointmentServiceItem>(
        r#"
        SELECT id, appointment_id, service_id, service_name_snapshot,
               service_description_snapshot, duration_minutes_snapshot, sort_order
        FROM appointment_services
        ORDER BY appointment_id ASC, sort_order ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let exported_at_utc = time::now_utc_iso();
    let record_count = clients.len()
        + services.len()
        + session_plans.len()
        + appointments.len()
        + appointment_services.len()
        + usize::from(business_profile.is_some());

    let payload = VersionedBusinessExport {
        export_schema_version: EXPORT_SCHEMA_VERSION,
        app_name: "My Schedule App".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at_utc: exported_at_utc.clone(),
        business_profile,
        clients,
        services,
        session_plans,
        appointments,
        appointment_services,
    };

    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|e| AppError::Internal(format!("Failed to serialize JSON export: {e}")))?;
    let filename = resolve_export_filename("my_schedule_export", "json", custom_filename)?;
    let (file_path, size_bytes) = write_file_atomically(exports_dir, &filename, &bytes)?;

    tracing::info!(
        format = "json",
        records = record_count,
        size_bytes,
        "Business data exported to JSON"
    );

    Ok(ExportResult {
        format: "json".to_string(),
        filename,
        file_path,
        size_bytes,
        record_count,
        exported_at_utc,
    })
}

/// Exports a single business entity table to a standards-compliant UTF-8 CSV file in `exports_dir`.
pub async fn export_csv(
    pool: &SqlitePool,
    exports_dir: &Path,
    entity: CsvEntity,
    custom_filename: Option<&str>,
) -> AppResult<ExportResult> {
    let mut csv = String::new();
    let record_count: usize;

    match entity {
        CsvEntity::Clients => {
            csv.push_str(&format_csv_row(&[
                "id",
                "full_name",
                "phone",
                "email",
                "address",
                "notes",
                "is_archived",
                "archived_at",
                "created_at",
                "updated_at",
            ]));
            let rows = sqlx::query_as::<_, Client>(
                r#"
                SELECT id, full_name, phone, email, address, notes,
                       is_archived, archived_at, created_at, updated_at
                FROM clients
                ORDER BY full_name ASC, id ASC
                "#,
            )
            .fetch_all(pool)
            .await?;
            record_count = rows.len();
            for c in rows {
                let archived = if c.is_archived { "true" } else { "false" };
                csv.push_str(&format_csv_row(&[
                    &c.id,
                    &c.full_name,
                    c.phone.as_deref().unwrap_or(""),
                    c.email.as_deref().unwrap_or(""),
                    c.address.as_deref().unwrap_or(""),
                    c.notes.as_deref().unwrap_or(""),
                    archived,
                    c.archived_at.as_deref().unwrap_or(""),
                    &c.created_at,
                    &c.updated_at,
                ]));
            }
        }
        CsvEntity::Services => {
            csv.push_str(&format_csv_row(&[
                "id",
                "name",
                "description",
                "default_duration_minutes",
                "is_active",
                "is_archived",
                "archived_at",
                "created_at",
                "updated_at",
            ]));
            let rows = sqlx::query_as::<_, Service>(
                r#"
                SELECT id, name, description, default_duration_minutes,
                       is_active, is_archived, archived_at, created_at, updated_at
                FROM services
                ORDER BY name ASC, id ASC
                "#,
            )
            .fetch_all(pool)
            .await?;
            record_count = rows.len();
            for s in rows {
                let dur = s.default_duration_minutes.to_string();
                let active = if s.is_active { "true" } else { "false" };
                let archived = if s.is_archived { "true" } else { "false" };
                csv.push_str(&format_csv_row(&[
                    &s.id,
                    &s.name,
                    s.description.as_deref().unwrap_or(""),
                    &dur,
                    active,
                    archived,
                    s.archived_at.as_deref().unwrap_or(""),
                    &s.created_at,
                    &s.updated_at,
                ]));
            }
        }
        CsvEntity::Appointments => {
            csv.push_str(&format_csv_row(&[
                "id",
                "client_id",
                "session_plan_id",
                "start_utc",
                "end_utc",
                "timezone",
                "status",
                "notes",
                "created_at",
                "updated_at",
            ]));
            let rows = sqlx::query_as::<_, ExportAppointment>(
                r#"
                SELECT id, client_id, session_plan_id, start_utc, end_utc,
                       timezone, status, notes, created_at, updated_at
                FROM appointments
                ORDER BY start_utc ASC, id ASC
                "#,
            )
            .fetch_all(pool)
            .await?;
            record_count = rows.len();
            for a in rows {
                csv.push_str(&format_csv_row(&[
                    &a.id,
                    &a.client_id,
                    a.session_plan_id.as_deref().unwrap_or(""),
                    &a.start_utc,
                    &a.end_utc,
                    &a.timezone,
                    &a.status,
                    a.notes.as_deref().unwrap_or(""),
                    &a.created_at,
                    &a.updated_at,
                ]));
            }
        }
        CsvEntity::AppointmentServices => {
            csv.push_str(&format_csv_row(&[
                "id",
                "appointment_id",
                "service_id",
                "service_name_snapshot",
                "service_description_snapshot",
                "duration_minutes_snapshot",
                "sort_order",
            ]));
            let rows = sqlx::query_as::<_, AppointmentServiceItem>(
                r#"
                SELECT id, appointment_id, service_id, service_name_snapshot,
                       service_description_snapshot, duration_minutes_snapshot, sort_order
                FROM appointment_services
                ORDER BY appointment_id ASC, sort_order ASC, id ASC
                "#,
            )
            .fetch_all(pool)
            .await?;
            record_count = rows.len();
            for item in rows {
                let dur = item.duration_minutes_snapshot.to_string();
                let order = item.sort_order.to_string();
                csv.push_str(&format_csv_row(&[
                    &item.id,
                    &item.appointment_id,
                    &item.service_id,
                    &item.service_name_snapshot,
                    item.service_description_snapshot.as_deref().unwrap_or(""),
                    &dur,
                    &order,
                ]));
            }
        }
    }

    let exported_at_utc = time::now_utc_iso();
    let filename = resolve_export_filename(entity.as_str(), "csv", custom_filename)?;
    let (file_path, size_bytes) = write_file_atomically(exports_dir, &filename, csv.as_bytes())?;

    tracing::info!(
        format = "csv",
        entity = entity.as_str(),
        records = record_count,
        size_bytes,
        "Entity exported to CSV"
    );

    Ok(ExportResult {
        format: format!("csv:{}", entity.as_str()),
        filename,
        file_path,
        size_bytes,
        record_count,
        exported_at_utc,
    })
}
