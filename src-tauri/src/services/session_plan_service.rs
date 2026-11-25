use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::repositories::{client_repo, session_plan_repo};
use crate::errors::{AppError, AppResult};
use crate::models::session_plan::{
    CreateSessionPlanInput, ListSessionPlansParams, SessionPlan, SessionPlanStatus,
    SessionPlanSummary, UpdateSessionPlanInput,
};
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
