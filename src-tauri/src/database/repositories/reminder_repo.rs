use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::errors::{AppError, AppResult};
use crate::models::reminder::{DueReminderRow, NotificationItem};

/// Cancels all pending reminders for an appointment (used on cancel/complete/reschedule).
pub async fn cancel_pending_for_appointment_tx(
    tx: &mut Transaction<'_, Sqlite>,
    appointment_id: &str,
    now_iso: &str,
) -> AppResult<u64> {
    let result = sqlx::query(
        r#"
        UPDATE reminders
        SET status = 'cancelled', updated_at = ?1
        WHERE appointment_id = ?2 AND status = 'pending'
        "#,
    )
    .bind(now_iso)
    .bind(appointment_id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected())
}

/// Inserts a new pending reminder or reactivates a previously cancelled reminder with the
/// exact same `idempotency_key`. Never modifies reminders that are already `sent` or `expired`
/// so duplicates cannot occur across restarts or reschedules.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_pending_reminder_tx(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    appointment_id: &str,
    reminder_type: &str,
    scheduled_for_utc: &str,
    expires_at_utc: &str,
    idempotency_key: &str,
    now_iso: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO reminders (
            id, appointment_id, reminder_type, scheduled_for_utc,
            expires_at_utc, status, idempotency_key, sent_at, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, NULL, ?7, ?7)
        ON CONFLICT(idempotency_key) DO UPDATE SET
            scheduled_for_utc = excluded.scheduled_for_utc,
            expires_at_utc = excluded.expires_at_utc,
            status = 'pending',
            updated_at = excluded.updated_at
        WHERE reminders.status = 'cancelled'
        "#,
    )
    .bind(id)
    .bind(appointment_id)
    .bind(reminder_type)
    .bind(scheduled_for_utc)
    .bind(expires_at_utc)
    .bind(idempotency_key)
    .bind(now_iso)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Marks pending reminders as `cancelled` if their appointment is no longer `scheduled`,
/// and marks pending reminders as `expired` if `expires_at_utc <= now_utc_iso`.
pub async fn expire_and_cleanup_stale_reminders(
    pool: &SqlitePool,
    now_utc_iso: &str,
) -> AppResult<u64> {
    let cancelled_non_scheduled = sqlx::query(
        r#"
        UPDATE reminders
        SET status = 'cancelled', updated_at = ?1
        WHERE status = 'pending'
          AND appointment_id IN (SELECT id FROM appointments WHERE status != 'scheduled')
        "#,
    )
    .bind(now_utc_iso)
    .execute(pool)
    .await?
    .rows_affected();

    let expired = sqlx::query(
        r#"
        UPDATE reminders
        SET status = 'expired', updated_at = ?1
        WHERE status = 'pending'
          AND expires_at_utc <= ?1
        "#,
    )
    .bind(now_utc_iso)
    .execute(pool)
    .await?
    .rows_affected();

    Ok(cancelled_non_scheduled + expired)
}

/// Fetches all pending reminders that are due (`scheduled_for_utc <= now_utc_iso`)
/// and not yet expired (`expires_at_utc > now_utc_iso`) for active scheduled appointments.
pub async fn list_due_pending_reminders(
    pool: &SqlitePool,
    now_utc_iso: &str,
) -> AppResult<Vec<DueReminderRow>> {
    let rows = sqlx::query_as::<_, DueReminderRow>(
        r#"
        SELECT
            r.id,
            r.appointment_id,
            r.reminder_type,
            r.scheduled_for_utc,
            r.expires_at_utc,
            r.status,
            r.idempotency_key,
            a.start_utc AS appointment_start_utc,
            a.end_utc AS appointment_end_utc,
            a.status AS appointment_status,
            c.full_name AS client_name
        FROM reminders r
        INNER JOIN appointments a ON a.id = r.appointment_id
        INNER JOIN clients c ON c.id = a.client_id
        WHERE r.status = 'pending'
          AND r.scheduled_for_utc <= ?1
          AND r.expires_at_utc > ?1
          AND a.status = 'scheduled'
        ORDER BY r.scheduled_for_utc ASC, r.id ASC
        "#,
    )
    .bind(now_utc_iso)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Fetches upcoming pending reminders for display in the Reminders UI.
pub async fn list_upcoming_pending_reminders(
    pool: &SqlitePool,
    limit: i64,
) -> AppResult<Vec<DueReminderRow>> {
    let rows = sqlx::query_as::<_, DueReminderRow>(
        r#"
        SELECT
            r.id,
            r.appointment_id,
            r.reminder_type,
            r.scheduled_for_utc,
            r.expires_at_utc,
            r.status,
            r.idempotency_key,
            a.start_utc AS appointment_start_utc,
            a.end_utc AS appointment_end_utc,
            a.status AS appointment_status,
            c.full_name AS client_name
        FROM reminders r
        INNER JOIN appointments a ON a.id = r.appointment_id
        INNER JOIN clients c ON c.id = a.client_id
        WHERE r.status = 'pending'
          AND a.status = 'scheduled'
        ORDER BY r.scheduled_for_utc ASC, r.id ASC
        LIMIT ?1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Atomically marks a pending reminder as `sent` inside a transaction.
/// Returns `true` if this call transitioned the row from `pending` -> `sent`.
pub async fn mark_reminder_sent_tx(
    tx: &mut Transaction<'_, Sqlite>,
    reminder_id: &str,
    now_iso: &str,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"
        UPDATE reminders
        SET status = 'sent', sent_at = ?1, updated_at = ?1
        WHERE id = ?2 AND status = 'pending'
        "#,
    )
    .bind(now_iso)
    .bind(reminder_id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Inserts a notification if its `idempotency_key` does not already exist.
/// Returns `true` if a new notification row was inserted.
#[allow(clippy::too_many_arguments)]
pub async fn insert_notification_if_new_tx(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    reminder_id: Option<&str>,
    appointment_id: Option<&str>,
    kind: &str,
    idempotency_key: &str,
    title: &str,
    body: &str,
    now_iso: &str,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"
        INSERT OR IGNORE INTO notifications (
            id, reminder_id, appointment_id, kind, idempotency_key,
            title, body, is_read, is_dismissed, os_delivery_status,
            os_delivery_error, created_at, read_at, dismissed_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 0, 'pending', NULL, ?8, NULL, NULL)
        "#,
    )
    .bind(id)
    .bind(reminder_id)
    .bind(appointment_id)
    .bind(kind)
    .bind(idempotency_key)
    .bind(title)
    .bind(body)
    .bind(now_iso)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Updates OS notification delivery status and optional sanitized error message.
pub async fn update_notification_os_delivery(
    pool: &SqlitePool,
    notification_id: &str,
    status: &str,
    error_message: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE notifications
        SET os_delivery_status = ?1,
            os_delivery_error = ?2
        WHERE id = ?3
        "#,
    )
    .bind(status)
    .bind(error_message)
    .bind(notification_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Checks whether the day-of first-launch reminder pass has already run for `business_local_date`.
pub async fn has_daily_run_completed(
    pool: &SqlitePool,
    business_local_date: &str,
) -> AppResult<bool> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM reminder_daily_runs WHERE business_local_date = ?1 LIMIT 1",
    )
    .bind(business_local_date)
    .fetch_optional(pool)
    .await?;
    Ok(exists.is_some())
}

/// Persists the daily launch marker inside a transaction (`INSERT OR IGNORE`).
pub async fn record_daily_run_tx(
    tx: &mut Transaction<'_, Sqlite>,
    business_local_date: &str,
    now_iso: &str,
    appointments_notified: i64,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO reminder_daily_runs (
            business_local_date, processed_at, appointments_notified
        )
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(business_local_date)
    .bind(now_iso)
    .bind(appointments_notified)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Lists in-app notifications ordered newest first.
pub async fn list_notifications(
    pool: &SqlitePool,
    include_dismissed: bool,
    limit: i64,
) -> AppResult<Vec<NotificationItem>> {
    let rows = if include_dismissed {
        sqlx::query_as::<_, NotificationItem>(
            r#"
            SELECT id, reminder_id, appointment_id, kind, idempotency_key,
                   title, body, is_read, is_dismissed, os_delivery_status,
                   os_delivery_error, created_at, read_at, dismissed_at
            FROM notifications
            ORDER BY created_at DESC, id DESC
            LIMIT ?1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, NotificationItem>(
            r#"
            SELECT id, reminder_id, appointment_id, kind, idempotency_key,
                   title, body, is_read, is_dismissed, os_delivery_status,
                   os_delivery_error, created_at, read_at, dismissed_at
            FROM notifications
            WHERE is_dismissed = 0
            ORDER BY created_at DESC, id DESC
            LIMIT ?1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?
    };
    Ok(rows)
}

pub async fn get_notification_by_id(
    pool: &SqlitePool,
    id: &str,
) -> AppResult<Option<NotificationItem>> {
    let row = sqlx::query_as::<_, NotificationItem>(
        r#"
        SELECT id, reminder_id, appointment_id, kind, idempotency_key,
               title, body, is_read, is_dismissed, os_delivery_status,
               os_delivery_error, created_at, read_at, dismissed_at
        FROM notifications
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn count_unread_notifications(pool: &SqlitePool) -> AppResult<i64> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE is_dismissed = 0 AND is_read = 0",
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn mark_notification_read(
    pool: &SqlitePool,
    id: &str,
    now_iso: &str,
) -> AppResult<NotificationItem> {
    let result = sqlx::query(
        r#"
        UPDATE notifications
        SET is_read = 1, read_at = COALESCE(read_at, ?1)
        WHERE id = ?2
        "#,
    )
    .bind(now_iso)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification not found.".into()));
    }

    get_notification_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Notification not found.".into()))
}

pub async fn mark_all_notifications_read(pool: &SqlitePool, now_iso: &str) -> AppResult<u64> {
    let result = sqlx::query(
        r#"
        UPDATE notifications
        SET is_read = 1, read_at = COALESCE(read_at, ?1)
        WHERE is_dismissed = 0 AND is_read = 0
        "#,
    )
    .bind(now_iso)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn dismiss_notification(
    pool: &SqlitePool,
    id: &str,
    now_iso: &str,
) -> AppResult<NotificationItem> {
    let result = sqlx::query(
        r#"
        UPDATE notifications
        SET is_dismissed = 1,
            is_read = 1,
            read_at = COALESCE(read_at, ?1),
            dismissed_at = COALESCE(dismissed_at, ?1)
        WHERE id = ?2
        "#,
    )
    .bind(now_iso)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification not found.".into()));
    }

    get_notification_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Notification not found.".into()))
}
