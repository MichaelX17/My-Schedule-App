use sqlx::{QueryBuilder, Sqlite, SqlitePool, Transaction};

use crate::errors::{AppError, AppResult};
use crate::models::appointment::{
    AppointmentRow, AppointmentServiceItem, AppointmentStatus, ListAppointmentsParams,
};

const APPOINTMENT_SELECT: &str = r#"
SELECT
    a.id,
    a.client_id,
    c.full_name AS client_name,
    c.is_archived AS client_is_archived,
    a.session_plan_id,
    sp.title AS session_plan_title,
    a.start_utc,
    a.end_utc,
    a.timezone,
    a.status,
    a.notes,
    a.created_at,
    a.updated_at
FROM appointments a
INNER JOIN clients c ON c.id = a.client_id
LEFT JOIN session_plans sp ON sp.id = a.session_plan_id
"#;

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

pub async fn list_rows(
    pool: &SqlitePool,
    params: &ListAppointmentsParams,
) -> AppResult<Vec<AppointmentRow>> {
    let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(APPOINTMENT_SELECT);
    let mut has_where = false;

    let mut push_cond = |qb: &mut QueryBuilder<'_, Sqlite>| {
        qb.push(if has_where { " AND " } else { " WHERE " });
        has_where = true;
    };

    if let Some(status) = params.status {
        push_cond(&mut qb);
        qb.push("a.status = ");
        qb.push_bind(status.as_str());
    }

    if let Some(ref client_id) = params.client_id {
        let trimmed = client_id.trim();
        if !trimmed.is_empty() {
            push_cond(&mut qb);
            qb.push("a.client_id = ");
            qb.push_bind(trimmed.to_string());
        }
    }

    if let Some(ref plan_id) = params.session_plan_id {
        let trimmed = plan_id.trim();
        if !trimmed.is_empty() {
            push_cond(&mut qb);
            qb.push("a.session_plan_id = ");
            qb.push_bind(trimmed.to_string());
        }
    }

    if let Some(ref from_utc) = params.from_utc {
        let trimmed = from_utc.trim();
        if !trimmed.is_empty() {
            push_cond(&mut qb);
            qb.push("a.end_utc >= ");
            qb.push_bind(trimmed.to_string());
        }
    }

    if let Some(ref to_utc) = params.to_utc {
        let trimmed = to_utc.trim();
        if !trimmed.is_empty() {
            push_cond(&mut qb);
            qb.push("a.start_utc <= ");
            qb.push_bind(trimmed.to_string());
        }
    }

    if let Some(pattern) = params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(escape_like_pattern)
    {
        push_cond(&mut qb);
        qb.push("(c.full_name LIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" ESCAPE '\\' OR a.notes LIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" ESCAPE '\\' OR EXISTS (SELECT 1 FROM appointment_services s WHERE s.appointment_id = a.id AND s.service_name_snapshot LIKE ");
        qb.push_bind(pattern);
        qb.push(" ESCAPE '\\'))");
    }

    qb.push(" ORDER BY a.start_utc ASC");

    let rows = qb
        .build_query_as::<AppointmentRow>()
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

/// Lists appointments overlapping the half-open UTC interval `[from_utc_inclusive, to_utc_exclusive)`.
///
/// An appointment overlaps iff `start_utc < to_utc_exclusive AND end_utc > from_utc_inclusive`.
pub async fn list_rows_in_utc_range(
    pool: &SqlitePool,
    from_utc_inclusive: &str,
    to_utc_exclusive: &str,
    status: Option<AppointmentStatus>,
) -> AppResult<Vec<AppointmentRow>> {
    let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(APPOINTMENT_SELECT);
    qb.push(" WHERE a.start_utc < ");
    qb.push_bind(to_utc_exclusive.to_string());
    qb.push(" AND a.end_utc > ");
    qb.push_bind(from_utc_inclusive.to_string());

    if let Some(status_filter) = status {
        qb.push(" AND a.status = ");
        qb.push_bind(status_filter.as_str());
    }

    qb.push(" ORDER BY a.start_utc ASC, a.id ASC");

    let rows = qb
        .build_query_as::<AppointmentRow>()
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_row_by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<AppointmentRow>> {
    let query = format!("{APPOINTMENT_SELECT} WHERE a.id = ?1");
    let row = sqlx::query_as::<_, AppointmentRow>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_services_for_appointment(
    pool: &SqlitePool,
    appointment_id: &str,
) -> AppResult<Vec<AppointmentServiceItem>> {
    let items = sqlx::query_as::<_, AppointmentServiceItem>(
        r#"
        SELECT id, appointment_id, service_id, service_name_snapshot,
               service_description_snapshot, duration_minutes_snapshot, sort_order
        FROM appointment_services
        WHERE appointment_id = ?1
        ORDER BY sort_order ASC, id ASC
        "#,
    )
    .bind(appointment_id)
    .fetch_all(pool)
    .await?;
    Ok(items)
}

pub async fn find_overlapping_scheduled(
    pool: &SqlitePool,
    start_utc: &str,
    end_utc: &str,
    exclude_appointment_id: Option<&str>,
) -> AppResult<Vec<AppointmentRow>> {
    let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(APPOINTMENT_SELECT);
    qb.push(" WHERE a.status = 'scheduled' AND a.start_utc < ");
    qb.push_bind(end_utc.to_string());
    qb.push(" AND a.end_utc > ");
    qb.push_bind(start_utc.to_string());

    if let Some(exclude_id) = exclude_appointment_id {
        qb.push(" AND a.id != ");
        qb.push_bind(exclude_id.to_string());
    }

    qb.push(" ORDER BY a.start_utc ASC");

    let rows = qb
        .build_query_as::<AppointmentRow>()
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_appointment_tx(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    client_id: &str,
    session_plan_id: Option<&str>,
    start_utc: &str,
    end_utc: &str,
    timezone: &str,
    status: &str,
    notes: Option<&str>,
    created_at: &str,
    updated_at: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO appointments (
            id, client_id, session_plan_id, start_utc, end_utc,
            timezone, status, notes, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
    )
    .bind(id)
    .bind(client_id)
    .bind(session_plan_id)
    .bind(start_utc)
    .bind(end_utc)
    .bind(timezone)
    .bind(status)
    .bind(notes)
    .bind(created_at)
    .bind(updated_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn replace_appointment_services_tx(
    tx: &mut Transaction<'_, Sqlite>,
    appointment_id: &str,
    services: &[AppointmentServiceItem],
) -> AppResult<()> {
    sqlx::query("DELETE FROM appointment_services WHERE appointment_id = ?1")
        .bind(appointment_id)
        .execute(&mut **tx)
        .await?;

    for item in services {
        sqlx::query(
            r#"
            INSERT INTO appointment_services (
                id, appointment_id, service_id, service_name_snapshot,
                service_description_snapshot, duration_minutes_snapshot, sort_order
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&item.id)
        .bind(&item.appointment_id)
        .bind(&item.service_id)
        .bind(&item.service_name_snapshot)
        .bind(&item.service_description_snapshot)
        .bind(item.duration_minutes_snapshot)
        .bind(item.sort_order)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_appointment_tx(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    session_plan_id: Option<&str>,
    start_utc: &str,
    end_utc: &str,
    timezone: &str,
    notes: Option<&str>,
    updated_at: &str,
) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE appointments
        SET session_plan_id = ?1,
            start_utc = ?2,
            end_utc = ?3,
            timezone = ?4,
            notes = ?5,
            updated_at = ?6
        WHERE id = ?7
        "#,
    )
    .bind(session_plan_id)
    .bind(start_utc)
    .bind(end_utc)
    .bind(timezone)
    .bind(notes)
    .bind(updated_at)
    .bind(id)
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Appointment not found.".into()));
    }
    Ok(())
}

pub async fn update_schedule_tx(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    start_utc: &str,
    end_utc: &str,
    timezone: &str,
    updated_at: &str,
) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE appointments
        SET start_utc = ?1,
            end_utc = ?2,
            timezone = ?3,
            updated_at = ?4
        WHERE id = ?5
        "#,
    )
    .bind(start_utc)
    .bind(end_utc)
    .bind(timezone)
    .bind(updated_at)
    .bind(id)
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Appointment not found.".into()));
    }
    Ok(())
}

pub async fn update_status_tx(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    status: &str,
    updated_at: &str,
) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        UPDATE appointments
        SET status = ?1,
            updated_at = ?2
        WHERE id = ?3
        "#,
    )
    .bind(status)
    .bind(updated_at)
    .bind(id)
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Appointment not found.".into()));
    }
    Ok(())
}

pub async fn delete_tx(tx: &mut Transaction<'_, Sqlite>, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM appointments WHERE id = ?1")
        .bind(id)
        .execute(&mut **tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Appointment not found.".into()));
    }
    Ok(())
}

pub async fn delete_by_session_plan_tx(
    tx: &mut Transaction<'_, Sqlite>,
    session_plan_id: &str,
) -> AppResult<u64> {
    let result = sqlx::query("DELETE FROM appointments WHERE session_plan_id = ?1")
        .bind(session_plan_id)
        .execute(&mut **tx)
        .await?;
    Ok(result.rows_affected())
}

pub async fn unlink_session_plan_tx(
    tx: &mut Transaction<'_, Sqlite>,
    session_plan_id: &str,
    updated_at: &str,
) -> AppResult<u64> {
    let result = sqlx::query(
        "UPDATE appointments SET session_plan_id = NULL, updated_at = ?1 WHERE session_plan_id = ?2",
    )
    .bind(updated_at)
    .bind(session_plan_id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected())
}
