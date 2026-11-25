use sqlx::{QueryBuilder, Sqlite, SqlitePool, Transaction};

use crate::errors::{AppError, AppResult};
use crate::models::session_plan::{ListSessionPlansParams, SessionPlan, SessionPlanSummary};

const SUMMARY_SELECT: &str = r#"
SELECT
    sp.id,
    sp.client_id,
    c.full_name AS client_name,
    sp.title,
    sp.description,
    sp.planned_sessions,
    sp.status,
    COUNT(a.id) AS linked_total,
    COALESCE(SUM(CASE WHEN a.status = 'scheduled' THEN 1 ELSE 0 END), 0) AS linked_scheduled,
    COALESCE(SUM(CASE WHEN a.status = 'completed' THEN 1 ELSE 0 END), 0) AS linked_completed,
    COALESCE(SUM(CASE WHEN a.status = 'cancelled' THEN 1 ELSE 0 END), 0) AS linked_cancelled,
    sp.created_at,
    sp.updated_at
FROM session_plans sp
INNER JOIN clients c ON c.id = sp.client_id
LEFT JOIN appointments a ON a.session_plan_id = sp.id
"#;

pub async fn list(
    pool: &SqlitePool,
    params: &ListSessionPlansParams,
) -> AppResult<Vec<SessionPlanSummary>> {
    let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(SUMMARY_SELECT);
    let mut has_where = false;

    if let Some(ref client_id) = params.client_id {
        let trimmed = client_id.trim();
        if !trimmed.is_empty() {
            qb.push(" WHERE sp.client_id = ");
            qb.push_bind(trimmed.to_string());
            has_where = true;
        }
    }

    if let Some(status) = params.status {
        qb.push(if has_where { " AND " } else { " WHERE " });
        qb.push("sp.status = ");
        qb.push_bind(status.as_str());
    }

    qb.push(" GROUP BY sp.id ORDER BY sp.updated_at DESC, sp.title COLLATE NOCASE ASC");

    let items = qb
        .build_query_as::<SessionPlanSummary>()
        .fetch_all(pool)
        .await?;
    Ok(items)
}

pub async fn get_by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<SessionPlan>> {
    let plan = sqlx::query_as::<_, SessionPlan>(
        "SELECT id, client_id, title, description, planned_sessions, status, created_at, updated_at FROM session_plans WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(plan)
}

pub async fn get_summary_by_id(
    pool: &SqlitePool,
    id: &str,
) -> AppResult<Option<SessionPlanSummary>> {
    let query = format!("{SUMMARY_SELECT} WHERE sp.id = ?1 GROUP BY sp.id");
    let summary = sqlx::query_as::<_, SessionPlanSummary>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(summary)
}

pub async fn insert_tx(tx: &mut Transaction<'_, Sqlite>, plan: &SessionPlan) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO session_plans (
            id, client_id, title, description, planned_sessions, status, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(&plan.id)
    .bind(&plan.client_id)
    .bind(&plan.title)
    .bind(&plan.description)
    .bind(plan.planned_sessions)
    .bind(&plan.status)
    .bind(&plan.created_at)
    .bind(&plan.updated_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn insert(pool: &SqlitePool, plan: &SessionPlan) -> AppResult<SessionPlanSummary> {
    sqlx::query(
        r#"
        INSERT INTO session_plans (
            id, client_id, title, description, planned_sessions, status, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(&plan.id)
    .bind(&plan.client_id)
    .bind(&plan.title)
    .bind(&plan.description)
    .bind(plan.planned_sessions)
    .bind(&plan.status)
    .bind(&plan.created_at)
    .bind(&plan.updated_at)
    .execute(pool)
    .await?;

    get_summary_by_id(pool, &plan.id)
        .await?
        .ok_or_else(|| AppError::Internal("Inserted session plan could not be reloaded.".into()))
}

pub async fn update(pool: &SqlitePool, plan: &SessionPlan) -> AppResult<SessionPlanSummary> {
    let result = sqlx::query(
        r#"
        UPDATE session_plans
        SET title = ?1,
            description = ?2,
            planned_sessions = ?3,
            status = ?4,
            updated_at = ?5
        WHERE id = ?6
        "#,
    )
    .bind(&plan.title)
    .bind(&plan.description)
    .bind(plan.planned_sessions)
    .bind(&plan.status)
    .bind(&plan.updated_at)
    .bind(&plan.id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Session plan not found.".into()));
    }

    get_summary_by_id(pool, &plan.id)
        .await?
        .ok_or_else(|| AppError::Internal("Updated session plan could not be reloaded.".into()))
}

pub async fn decrement_planned_sessions_tx(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    updated_at: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE session_plans
        SET planned_sessions = CASE
                WHEN planned_sessions > 1 THEN planned_sessions - 1
                ELSE 1
            END,
            updated_at = ?1
        WHERE id = ?2
        "#,
    )
    .bind(updated_at)
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn delete_tx(tx: &mut Transaction<'_, Sqlite>, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM session_plans WHERE id = ?1")
        .bind(id)
        .execute(&mut **tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Session plan not found.".into()));
    }
    Ok(())
}
