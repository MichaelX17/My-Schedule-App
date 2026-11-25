use std::collections::HashSet;

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::repositories::{
    appointment_repo, business_repo, client_repo, service_repo, session_plan_repo,
};
use crate::errors::{AppError, AppResult};
use crate::models::appointment::{
    AppointmentDetail, AppointmentMutationResult, AppointmentRow, AppointmentServiceItem,
    AppointmentStatus, CheckOverlapsInput, CreateAppointmentInput, ListAppointmentsParams,
    OverlapWarning, RescheduleAppointmentInput, UpdateAppointmentInput,
};
use crate::models::session_plan::SessionPlanStatus;
use crate::scheduler;
use crate::services::client_service;
use crate::time::{self, DstDisambiguation};

pub const MIN_APPOINTMENT_MINUTES: i64 = 5;
pub const MAX_APPOINTMENT_MINUTES: i64 = 1440;
const MAX_NOTES_LEN: usize = 4000;

pub(crate) async fn resolve_business_timezone(pool: &SqlitePool) -> AppResult<String> {
    if let Some(profile) = business_repo::get(pool).await? {
        return Ok(profile.timezone);
    }
    Ok("UTC".to_string())
}

fn validate_time_window(
    start_local: &str,
    end_local: &str,
    business_tz: &str,
    disambiguation: Option<DstDisambiguation>,
) -> AppResult<(DateTime<Utc>, DateTime<Utc>)> {
    let start_utc = time::resolve_local_datetime_to_utc(start_local, business_tz, disambiguation)?;
    let end_utc = time::resolve_local_datetime_to_utc(end_local, business_tz, disambiguation)?;

    if end_utc <= start_utc {
        return Err(AppError::Validation(
            "Appointment end time must be after start time.".into(),
        ));
    }

    let duration_minutes = (end_utc - start_utc).num_minutes();
    if duration_minutes < MIN_APPOINTMENT_MINUTES {
        return Err(AppError::Validation(format!(
            "Appointment duration must be at least {MIN_APPOINTMENT_MINUTES} minutes."
        )));
    }
    if duration_minutes > MAX_APPOINTMENT_MINUTES {
        return Err(AppError::Validation(format!(
            "Appointment duration must be {MAX_APPOINTMENT_MINUTES} minutes (24 hours) or fewer."
        )));
    }

    Ok((start_utc, end_utc))
}

fn normalize_notes(raw: Option<String>) -> AppResult<Option<String>> {
    let Some(val) = raw else {
        return Ok(None);
    };
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MAX_NOTES_LEN {
        return Err(AppError::Validation(format!(
            "Notes must be {MAX_NOTES_LEN} characters or fewer."
        )));
    }
    Ok(Some(trimmed.to_string()))
}

async fn build_service_snapshots(
    pool: &SqlitePool,
    appointment_id: &str,
    service_ids: &[String],
    allow_existing_inactive_ids: &HashSet<String>,
) -> AppResult<Vec<AppointmentServiceItem>> {
    if service_ids.is_empty() {
        return Err(AppError::Validation(
            "Please select at least one service for the appointment.".into(),
        ));
    }

    let mut seen = HashSet::new();
    let mut snapshots = Vec::with_capacity(service_ids.len());

    for (idx, raw_id) in service_ids.iter().enumerate() {
        let id = raw_id.trim();
        if id.is_empty() {
            return Err(AppError::Validation("Invalid service selection.".into()));
        }
        if !seen.insert(id.to_string()) {
            return Err(AppError::Validation(
                "Duplicate service selected for the same appointment.".into(),
            ));
        }

        let service = service_repo::get_by_id(pool, id).await?.ok_or_else(|| {
            AppError::NotFound("One or more selected services were not found.".into())
        })?;

        let is_preserved_existing = allow_existing_inactive_ids.contains(id);
        if !is_preserved_existing && (service.is_archived || !service.is_active) {
            return Err(AppError::Validation(format!(
                "Service '{}' is inactive or archived and cannot be added to new appointments.",
                service.name
            )));
        }

        let snap = service.to_snapshot();
        snapshots.push(AppointmentServiceItem {
            id: Uuid::new_v4().to_string(),
            appointment_id: appointment_id.to_string(),
            service_id: snap.service_id,
            service_name_snapshot: snap.service_name,
            service_description_snapshot: snap.service_description,
            duration_minutes_snapshot: snap.duration_minutes,
            sort_order: idx as i64,
        });
    }

    Ok(snapshots)
}

async fn validate_session_plan_for_client(
    pool: &SqlitePool,
    plan_id: Option<&str>,
    client_id: &str,
) -> AppResult<Option<String>> {
    let Some(raw) = plan_id else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let plan = session_plan_repo::get_by_id(pool, trimmed)
        .await?
        .ok_or_else(|| AppError::NotFound("Selected session plan was not found.".into()))?;

    if plan.client_id != client_id {
        return Err(AppError::Validation(
            "Selected session plan belongs to a different client.".into(),
        ));
    }

    if plan.status == SessionPlanStatus::Archived.as_str() {
        return Err(AppError::Validation(
            "Cannot link an appointment to an archived session plan.".into(),
        ));
    }

    Ok(Some(plan.id))
}

pub(crate) async fn hydrate_detail(
    pool: &SqlitePool,
    row: AppointmentRow,
    display_tz: &str,
) -> AppResult<AppointmentDetail> {
    let services = appointment_repo::list_services_for_appointment(pool, &row.id).await?;
    let start_local = time::format_utc_in_timezone(&row.start_utc, display_tz)?;
    let end_local = time::format_utc_in_timezone(&row.end_utc, display_tz)?;

    Ok(AppointmentDetail {
        id: row.id,
        client_id: row.client_id,
        client_name: row.client_name,
        client_is_archived: row.client_is_archived,
        session_plan_id: row.session_plan_id,
        session_plan_title: row.session_plan_title,
        start_utc: row.start_utc,
        end_utc: row.end_utc,
        start_local,
        end_local,
        timezone: display_tz.to_string(),
        status: row.status,
        notes: row.notes,
        services,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

async fn collect_overlap_warnings(
    pool: &SqlitePool,
    start_utc: &str,
    end_utc: &str,
    exclude_id: Option<&str>,
    display_tz: &str,
) -> AppResult<Vec<OverlapWarning>> {
    let rows =
        appointment_repo::find_overlapping_scheduled(pool, start_utc, end_utc, exclude_id).await?;

    let mut warnings = Vec::with_capacity(rows.len());
    for row in rows {
        let start_local = time::format_utc_in_timezone(&row.start_utc, display_tz)?;
        let end_local = time::format_utc_in_timezone(&row.end_utc, display_tz)?;
        warnings.push(OverlapWarning {
            appointment_id: row.id,
            client_name: row.client_name,
            start_utc: row.start_utc,
            end_utc: row.end_utc,
            start_local,
            end_local,
        });
    }
    Ok(warnings)
}

/// Lists appointments with optional search, status, client, session plan, and UTC range filters.
pub async fn list_appointments(
    pool: &SqlitePool,
    params: ListAppointmentsParams,
) -> AppResult<Vec<AppointmentDetail>> {
    let business_tz = resolve_business_timezone(pool).await?;
    let rows = appointment_repo::list_rows(pool, &params).await?;

    let mut details = Vec::with_capacity(rows.len());
    for row in rows {
        details.push(hydrate_detail(pool, row, &business_tz).await?);
    }
    Ok(details)
}

/// Retrieves full appointment detail by ID, including historical service snapshots.
pub async fn get_appointment(pool: &SqlitePool, id: &str) -> AppResult<AppointmentDetail> {
    let business_tz = resolve_business_timezone(pool).await?;
    let row = appointment_repo::get_row_by_id(pool, id.trim())
        .await?
        .ok_or_else(|| AppError::NotFound("Appointment not found.".into()))?;
    hydrate_detail(pool, row, &business_tz).await
}

/// Checks for overlapping scheduled appointments without blocking creation.
pub async fn check_overlaps(
    pool: &SqlitePool,
    input: CheckOverlapsInput,
) -> AppResult<Vec<OverlapWarning>> {
    let business_tz = resolve_business_timezone(pool).await?;
    let (start_dt, end_dt) = validate_time_window(
        &input.start_local,
        &input.end_local,
        &business_tz,
        input.dst_disambiguation,
    )?;
    let start_iso = time::format_utc_iso(start_dt);
    let end_iso = time::format_utc_iso(end_dt);
    collect_overlap_warnings(
        pool,
        &start_iso,
        &end_iso,
        input.exclude_appointment_id.as_deref(),
        &business_tz,
    )
    .await
}

/// Creates an appointment (optionally creating a new client inline inside the same transaction),
/// captures immutable service snapshots, runs the transactional reminder hook, and returns
/// any non-blocking overlap warnings.
pub async fn create_appointment(
    pool: &SqlitePool,
    input: CreateAppointmentInput,
) -> AppResult<AppointmentMutationResult> {
    let business_tz = resolve_business_timezone(pool).await?;
    let (start_dt, end_dt) = validate_time_window(
        &input.start_local,
        &input.end_local,
        &business_tz,
        input.dst_disambiguation,
    )?;
    let start_iso = time::format_utc_iso(start_dt);
    let end_iso = time::format_utc_iso(end_dt);
    let notes = normalize_notes(input.notes)?;

    // Determine client: either existing active client or inline creation (mutually exclusive).
    let inline_client_to_insert = match (
        input
            .client_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
        input.inline_client,
    ) {
        (Some(_), Some(_)) => {
            return Err(AppError::Validation(
                "Provide either an existing client or inline client details, not both.".into(),
            ));
        }
        (None, None) => {
            return Err(AppError::Validation(
                "Please select a client or create one inline.".into(),
            ));
        }
        (Some(existing_id), None) => {
            let client = client_repo::get_by_id(pool, existing_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Selected client not found.".into()))?;
            if client.is_archived {
                return Err(AppError::Validation(
                    "Archived clients cannot be selected for new appointments. Restore the client first."
                        .into(),
                ));
            }
            (client.id, None)
        }
        (None, Some(inline_input)) => {
            if input
                .session_plan_id
                .as_deref()
                .map(str::trim)
                .is_some_and(|s| !s.is_empty())
            {
                return Err(AppError::Validation(
                    "Cannot attach an existing session plan to a brand-new inline client.".into(),
                ));
            }
            let validated_client = client_service::build_validated_client(inline_input)?;
            (validated_client.id.clone(), Some(validated_client))
        }
    };

    let (client_id, inline_client) = inline_client_to_insert;
    let session_plan_id =
        validate_session_plan_for_client(pool, input.session_plan_id.as_deref(), &client_id)
            .await?;

    let appointment_id = Uuid::new_v4().to_string();
    let snapshots =
        build_service_snapshots(pool, &appointment_id, &input.service_ids, &HashSet::new()).await?;

    let now = time::now_utc_iso();
    let mut tx = pool.begin().await?;

    if let Some(ref new_client) = inline_client {
        client_repo::insert_tx(&mut tx, new_client).await?;
    }

    appointment_repo::insert_appointment_tx(
        &mut tx,
        &appointment_id,
        &client_id,
        session_plan_id.as_deref(),
        &start_iso,
        &end_iso,
        &business_tz,
        AppointmentStatus::Scheduled.as_str(),
        notes.as_deref(),
        &now,
        &now,
    )
    .await?;

    appointment_repo::replace_appointment_services_tx(&mut tx, &appointment_id, &snapshots).await?;
    scheduler::on_appointment_changed_tx(&mut tx, &appointment_id).await?;

    tx.commit().await?;

    let overlap_warnings = collect_overlap_warnings(
        pool,
        &start_iso,
        &end_iso,
        Some(&appointment_id),
        &business_tz,
    )
    .await?;
    let appointment = get_appointment(pool, &appointment_id).await?;

    tracing::info!(
        appointment_id = %appointment.id,
        overlaps = overlap_warnings.len(),
        "Appointment created"
    );

    Ok(AppointmentMutationResult {
        appointment,
        overlap_warnings,
    })
}

/// Updates a scheduled appointment's time, services, optional session plan, and notes transactionally.
pub async fn update_appointment(
    pool: &SqlitePool,
    input: UpdateAppointmentInput,
) -> AppResult<AppointmentMutationResult> {
    let existing = appointment_repo::get_row_by_id(pool, input.id.trim())
        .await?
        .ok_or_else(|| AppError::NotFound("Appointment not found.".into()))?;

    if existing.status != AppointmentStatus::Scheduled.as_str() {
        return Err(AppError::Validation(
            "Only scheduled appointments can be edited.".into(),
        ));
    }

    let business_tz = resolve_business_timezone(pool).await?;
    let (start_dt, end_dt) = validate_time_window(
        &input.start_local,
        &input.end_local,
        &business_tz,
        input.dst_disambiguation,
    )?;
    let start_iso = time::format_utc_iso(start_dt);
    let end_iso = time::format_utc_iso(end_dt);
    let notes = normalize_notes(input.notes)?;

    let session_plan_id = validate_session_plan_for_client(
        pool,
        input.session_plan_id.as_deref(),
        &existing.client_id,
    )
    .await?;

    let current_services =
        appointment_repo::list_services_for_appointment(pool, &existing.id).await?;
    let existing_service_ids: HashSet<String> =
        current_services.into_iter().map(|s| s.service_id).collect();

    let snapshots = build_service_snapshots(
        pool,
        &existing.id,
        &input.service_ids,
        &existing_service_ids,
    )
    .await?;

    let now = time::now_utc_iso();
    let mut tx = pool.begin().await?;

    appointment_repo::update_appointment_tx(
        &mut tx,
        &existing.id,
        session_plan_id.as_deref(),
        &start_iso,
        &end_iso,
        &business_tz,
        notes.as_deref(),
        &now,
    )
    .await?;
    appointment_repo::replace_appointment_services_tx(&mut tx, &existing.id, &snapshots).await?;
    scheduler::on_appointment_changed_tx(&mut tx, &existing.id).await?;

    tx.commit().await?;

    let overlap_warnings =
        collect_overlap_warnings(pool, &start_iso, &end_iso, Some(&existing.id), &business_tz)
            .await?;
    let appointment = get_appointment(pool, &existing.id).await?;

    tracing::info!(appointment_id = %appointment.id, "Appointment updated");

    Ok(AppointmentMutationResult {
        appointment,
        overlap_warnings,
    })
}

/// Reschedules a scheduled appointment to a new time window transactionally.
pub async fn reschedule_appointment(
    pool: &SqlitePool,
    input: RescheduleAppointmentInput,
) -> AppResult<AppointmentMutationResult> {
    let existing = appointment_repo::get_row_by_id(pool, input.id.trim())
        .await?
        .ok_or_else(|| AppError::NotFound("Appointment not found.".into()))?;

    if existing.status != AppointmentStatus::Scheduled.as_str() {
        return Err(AppError::Validation(
            "Only scheduled appointments can be rescheduled.".into(),
        ));
    }

    let business_tz = resolve_business_timezone(pool).await?;
    let (start_dt, end_dt) = validate_time_window(
        &input.start_local,
        &input.end_local,
        &business_tz,
        input.dst_disambiguation,
    )?;
    let start_iso = time::format_utc_iso(start_dt);
    let end_iso = time::format_utc_iso(end_dt);
    let now = time::now_utc_iso();

    let mut tx = pool.begin().await?;
    appointment_repo::update_schedule_tx(
        &mut tx,
        &existing.id,
        &start_iso,
        &end_iso,
        &business_tz,
        &now,
    )
    .await?;
    scheduler::on_appointment_changed_tx(&mut tx, &existing.id).await?;
    tx.commit().await?;

    let overlap_warnings =
        collect_overlap_warnings(pool, &start_iso, &end_iso, Some(&existing.id), &business_tz)
            .await?;
    let appointment = get_appointment(pool, &existing.id).await?;

    tracing::info!(appointment_id = %appointment.id, "Appointment rescheduled");

    Ok(AppointmentMutationResult {
        appointment,
        overlap_warnings,
    })
}

/// Marks a scheduled appointment as completed. Rejects invalid transitions from cancelled.
pub async fn complete_appointment(pool: &SqlitePool, id: &str) -> AppResult<AppointmentDetail> {
    let existing = appointment_repo::get_row_by_id(pool, id.trim())
        .await?
        .ok_or_else(|| AppError::NotFound("Appointment not found.".into()))?;

    if existing.status == AppointmentStatus::Cancelled.as_str() {
        return Err(AppError::Validation(
            "Cancelled appointments cannot be marked as completed.".into(),
        ));
    }
    if existing.status == AppointmentStatus::Completed.as_str() {
        return get_appointment(pool, &existing.id).await;
    }

    let now = time::now_utc_iso();
    let mut tx = pool.begin().await?;
    appointment_repo::update_status_tx(
        &mut tx,
        &existing.id,
        AppointmentStatus::Completed.as_str(),
        &now,
    )
    .await?;
    scheduler::on_appointment_changed_tx(&mut tx, &existing.id).await?;
    tx.commit().await?;

    tracing::info!(appointment_id = %existing.id, "Appointment completed");
    get_appointment(pool, &existing.id).await
}

/// Cancels a scheduled appointment. Rejects invalid transitions from completed.
pub async fn cancel_appointment(pool: &SqlitePool, id: &str) -> AppResult<AppointmentDetail> {
    let existing = appointment_repo::get_row_by_id(pool, id.trim())
        .await?
        .ok_or_else(|| AppError::NotFound("Appointment not found.".into()))?;

    if existing.status == AppointmentStatus::Completed.as_str() {
        return Err(AppError::Validation(
            "Completed appointments cannot be cancelled.".into(),
        ));
    }
    if existing.status == AppointmentStatus::Cancelled.as_str() {
        return get_appointment(pool, &existing.id).await;
    }

    let now = time::now_utc_iso();
    let mut tx = pool.begin().await?;
    appointment_repo::update_status_tx(
        &mut tx,
        &existing.id,
        AppointmentStatus::Cancelled.as_str(),
        &now,
    )
    .await?;
    scheduler::on_appointment_changed_tx(&mut tx, &existing.id).await?;
    tx.commit().await?;

    tracing::info!(appointment_id = %existing.id, "Appointment cancelled");
    get_appointment(pool, &existing.id).await
}
