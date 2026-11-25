use std::collections::HashSet;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::repositories::{appointment_repo, client_repo, session_plan_repo};
use crate::errors::{AppError, AppResult};
use crate::models::appointment::AppointmentStatus;
use crate::models::session_plan::{
    CreateSessionPlanInput, CreateSessionPlanWithAppointmentsInput,
    CreateSessionPlanWithAppointmentsResult, ListSessionPlansParams, SessionPlan,
    SessionPlanStatus, SessionPlanSummary, UpdateSessionPlanInput,
};
use crate::scheduler;
use crate::services::appointment_service;
use crate::time;

const MAX_TITLE_LEN: usize = 120;
const MAX_DESC_LEN: usize = 1000;
const MIN_PLANNED_SESSIONS: i64 = 1;
const MAX_PLANNED_SESSIONS: i64 = 500;

fn validate_title(raw: &str) -> AppResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Session plan title is required.".into(),
        ));
    }
    if trimmed.chars().count() > MAX_TITLE_LEN {
        return Err(AppError::Validation(format!(
            "Session plan title must be {MAX_TITLE_LEN} characters or fewer."
        )));
    }
    Ok(trimmed.to_string())
}

fn normalize_description(raw: Option<String>) -> AppResult<Option<String>> {
    let Some(val) = raw else {
        return Ok(None);
    };
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MAX_DESC_LEN {
        return Err(AppError::Validation(format!(
            "Description must be {MAX_DESC_LEN} characters or fewer."
        )));
    }
    Ok(Some(trimmed.to_string()))
}

fn validate_planned_sessions(count: i64) -> AppResult<i64> {
    if !(MIN_PLANNED_SESSIONS..=MAX_PLANNED_SESSIONS).contains(&count) {
        return Err(AppError::Validation(format!(
            "Planned session count must be between {MIN_PLANNED_SESSIONS} and {MAX_PLANNED_SESSIONS}."
        )));
    }
    Ok(count)
}

pub async fn list_session_plans(
    pool: &SqlitePool,
    params: ListSessionPlansParams,
) -> AppResult<Vec<SessionPlanSummary>> {
    session_plan_repo::list(pool, &params).await
}

pub async fn get_session_plan(pool: &SqlitePool, id: &str) -> AppResult<SessionPlanSummary> {
    session_plan_repo::get_summary_by_id(pool, id.trim())
        .await?
        .ok_or_else(|| AppError::NotFound("Session plan not found.".into()))
}

pub async fn create_session_plan(
    pool: &SqlitePool,
    input: CreateSessionPlanInput,
) -> AppResult<SessionPlanSummary> {
    let client_id = input.client_id.trim();
    if client_id.is_empty() {
        return Err(AppError::Validation(
            "Client is required for a session plan.".into(),
        ));
    }

    let client = client_repo::get_by_id(pool, client_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Selected client not found.".into()))?;
    if client.is_archived {
        return Err(AppError::Validation(
            "Cannot create a new session plan for an archived client.".into(),
        ));
    }

    let now = time::now_utc_iso();
    let plan = SessionPlan {
        id: Uuid::new_v4().to_string(),
        client_id: client.id,
        title: validate_title(&input.title)?,
        description: normalize_description(input.description)?,
        planned_sessions: validate_planned_sessions(input.planned_sessions)?,
        status: SessionPlanStatus::Active.as_str().to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

    let created = session_plan_repo::insert(pool, &plan).await?;
    tracing::info!(session_plan_id = %created.id, "Session plan created");
    Ok(created)
}

/// Atomically creates a session plan and one individual scheduled appointment for every session
/// in the plan, preserving service snapshots and scheduling reminders for each session.
pub async fn create_session_plan_with_appointments(
    pool: &SqlitePool,
    input: CreateSessionPlanWithAppointmentsInput,
) -> AppResult<CreateSessionPlanWithAppointmentsResult> {
    let client_id = input.client_id.trim();
    if client_id.is_empty() {
        return Err(AppError::Validation(
            "Client is required for a session plan.".into(),
        ));
    }

    let client = client_repo::get_by_id(pool, client_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Selected client not found.".into()))?;
    if client.is_archived {
        return Err(AppError::Validation(
            "Cannot create a new session plan for an archived client.".into(),
        ));
    }

    let title = validate_title(&input.title)?;
    let description = normalize_description(input.description)?;
    let planned_sessions = validate_planned_sessions(input.planned_sessions)?;

    if input.sessions.len() != planned_sessions as usize {
        return Err(AppError::Validation(format!(
            "Please provide a date and time for each of the {planned_sessions} planned sessions."
        )));
    }

    let business_tz = appointment_service::resolve_business_timezone(pool).await?;

    // Pre-validate every session slot and service snapshot before opening the transaction.
    struct PreparedSession {
        appointment_id: String,
        start_iso: String,
        end_iso: String,
        notes: Option<String>,
        snapshots: Vec<crate::models::appointment::AppointmentServiceItem>,
    }

    let empty_allowed = HashSet::new();
    let mut prepared = Vec::with_capacity(input.sessions.len());
    for (idx, slot) in input.sessions.into_iter().enumerate() {
        let session_num = idx + 1;
        let (start_dt, end_dt) = appointment_service::validate_time_window(
            &slot.start_local,
            &slot.end_local,
            &business_tz,
            slot.dst_disambiguation,
        )
        .map_err(|err| match err {
            AppError::Validation(msg) => {
                AppError::Validation(format!("Session {session_num}: {msg}"))
            }
            other => other,
        })?;

        let appointment_id = Uuid::new_v4().to_string();
        let snapshots = appointment_service::build_service_snapshots(
            pool,
            &appointment_id,
            &input.service_ids,
            &empty_allowed,
        )
        .await?;

        let default_note = format!("Session {session_num} of {planned_sessions} — {title}");
        let notes = match appointment_service::normalize_notes(slot.notes)? {
            Some(custom) => Some(custom),
            None => Some(default_note),
        };

        prepared.push(PreparedSession {
            appointment_id,
            start_iso: time::format_utc_iso(start_dt),
            end_iso: time::format_utc_iso(end_dt),
            notes,
            snapshots,
        });
    }

    let now = time::now_utc_iso();
    let plan = SessionPlan {
        id: Uuid::new_v4().to_string(),
        client_id: client.id.clone(),
        title,
        description,
        planned_sessions,
        status: SessionPlanStatus::Active.as_str().to_string(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    let mut tx = pool.begin().await?;
    session_plan_repo::insert_tx(&mut tx, &plan).await?;

    let mut created_ids = Vec::with_capacity(prepared.len());
    for session in prepared {
        appointment_repo::insert_appointment_tx(
            &mut tx,
            &session.appointment_id,
            &plan.client_id,
            Some(&plan.id),
            &session.start_iso,
            &session.end_iso,
            &business_tz,
            AppointmentStatus::Scheduled.as_str(),
            session.notes.as_deref(),
            &now,
            &now,
        )
        .await?;

        appointment_repo::replace_appointment_services_tx(
            &mut tx,
            &session.appointment_id,
            &session.snapshots,
        )
        .await?;

        scheduler::on_appointment_changed_tx(&mut tx, &session.appointment_id).await?;

        created_ids.push(session.appointment_id);
    }

    tx.commit().await?;

    let plan_summary = session_plan_repo::get_summary_by_id(pool, &plan.id)
        .await?
        .ok_or_else(|| AppError::Internal("Created session plan could not be reloaded.".into()))?;

    let mut appointments = Vec::with_capacity(created_ids.len());
    for appt_id in created_ids {
        appointments.push(appointment_service::get_appointment(pool, &appt_id).await?);
    }

    tracing::info!(
        session_plan_id = %plan_summary.id,
        appointments_created = appointments.len(),
        "Session plan created with individual session appointments"
    );

    Ok(CreateSessionPlanWithAppointmentsResult {
        plan: plan_summary,
        appointments,
    })
}

pub async fn update_session_plan(
    pool: &SqlitePool,
    input: UpdateSessionPlanInput,
) -> AppResult<SessionPlanSummary> {
    let existing = session_plan_repo::get_by_id(pool, input.id.trim())
        .await?
        .ok_or_else(|| AppError::NotFound("Session plan not found.".into()))?;

    let status = input
        .status
        .map(|s| s.as_str().to_string())
        .unwrap_or(existing.status);

    let updated = SessionPlan {
        id: existing.id,
        client_id: existing.client_id,
        title: validate_title(&input.title)?,
        description: normalize_description(input.description)?,
        planned_sessions: validate_planned_sessions(input.planned_sessions)?,
        status,
        created_at: existing.created_at,
        updated_at: time::now_utc_iso(),
    };

    let saved = session_plan_repo::update(pool, &updated).await?;
    tracing::info!(session_plan_id = %saved.id, "Session plan updated");
    Ok(saved)
}

pub async fn archive_session_plan(pool: &SqlitePool, id: &str) -> AppResult<SessionPlanSummary> {
    let existing = session_plan_repo::get_by_id(pool, id.trim())
        .await?
        .ok_or_else(|| AppError::NotFound("Session plan not found.".into()))?;

    let updated = SessionPlan {
        status: SessionPlanStatus::Archived.as_str().to_string(),
        updated_at: time::now_utc_iso(),
        ..existing
    };

    let saved = session_plan_repo::update(pool, &updated).await?;
    tracing::info!(session_plan_id = %saved.id, "Session plan archived");
    Ok(saved)
}

/// Permanently deletes a session plan. When `delete_linked_appointments` is true, also deletes
/// every appointment linked to this plan inside the same transaction; otherwise unlinks them so
/// they remain as independent appointments.
pub async fn delete_session_plan(
    pool: &SqlitePool,
    id: &str,
    delete_linked_appointments: bool,
) -> AppResult<()> {
    let existing = session_plan_repo::get_by_id(pool, id.trim())
        .await?
        .ok_or_else(|| AppError::NotFound("Session plan not found.".into()))?;

    let now = time::now_utc_iso();
    let mut tx = pool.begin().await?;

    if delete_linked_appointments {
        appointment_repo::delete_by_session_plan_tx(&mut tx, &existing.id).await?;
    } else {
        appointment_repo::unlink_session_plan_tx(&mut tx, &existing.id, &now).await?;
    }

    session_plan_repo::delete_tx(&mut tx, &existing.id).await?;
    tx.commit().await?;

    tracing::info!(
        session_plan_id = %existing.id,
        delete_linked_appointments,
        "Session plan deleted"
    );
    Ok(())
}
