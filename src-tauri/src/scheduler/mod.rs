use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::database::repositories::{appointment_repo, reminder_repo};
use crate::errors::AppResult;
use crate::models::appointment::AppointmentStatus;
use crate::models::reminder::{NotificationKind, ReminderRunSummary, ReminderType};
use crate::notifications::{NotificationDeliveryOutcome, NotificationSender};
use crate::services::{appointment_service, settings_service};
use crate::time;

#[derive(sqlx::FromRow)]
struct AppointmentSyncRow {
    start_utc: String,
    status: String,
}

async fn resolve_business_timezone_tx(tx: &mut Transaction<'_, Sqlite>) -> AppResult<String> {
    let tz: Option<String> =
        sqlx::query_scalar("SELECT timezone FROM business_profile WHERE singleton_key = 1")
            .fetch_optional(&mut **tx)
            .await?;
    Ok(tz.unwrap_or_else(|| "UTC".to_string()))
}

/// Transactional hook invoked whenever an appointment is created, updated, rescheduled,
/// completed, or cancelled.
///
/// - If the appointment is `completed` or `cancelled`, all pending reminders are cancelled.
/// - If the appointment is `scheduled`, existing pending reminders are cancelled and
///   `one_day_before` + `one_hour_before` reminders are generated/recalculated using the
///   persisted business IANA timezone. Already-sent reminders for the exact same schedule
///   are never duplicated.
pub async fn on_appointment_changed_tx(
    tx: &mut Transaction<'_, Sqlite>,
    appointment_id: &str,
) -> AppResult<()> {
    let now_utc = Utc::now();
    let now_iso = time::format_utc_iso(now_utc);

    let appt = sqlx::query_as::<_, AppointmentSyncRow>(
        "SELECT start_utc, status FROM appointments WHERE id = ?1",
    )
    .bind(appointment_id)
    .fetch_optional(&mut **tx)
    .await?;

    let Some(appt) = appt else {
        reminder_repo::cancel_pending_for_appointment_tx(tx, appointment_id, &now_iso).await?;
        return Ok(());
    };

    // Always cancel stale pending reminders first; valid ones for the current schedule
    // will be upserted/reactivated immediately below.
    reminder_repo::cancel_pending_for_appointment_tx(tx, appointment_id, &now_iso).await?;

    if appt.status != AppointmentStatus::Scheduled.as_str() {
        return Ok(());
    }

    let start_utc = time::parse_utc_iso(&appt.start_utc)?;
    if start_utc <= now_utc {
        // Appointment has already started; do not schedule reminders in the past.
        return Ok(());
    }

    let business_tz = resolve_business_timezone_tx(tx).await?;
    let expires_iso = time::format_utc_iso(start_utc);

    let one_day_due = time::compute_one_day_before_utc(start_utc, &business_tz)?;
    if one_day_due > now_utc {
        let one_day_iso = time::format_utc_iso(one_day_due);
        let one_day_key = format!(
            "{}:{}:{}",
            ReminderType::OneDayBefore.as_str(),
            appointment_id,
            one_day_iso
        );
        reminder_repo::upsert_pending_reminder_tx(
            tx,
            &Uuid::new_v4().to_string(),
            appointment_id,
            ReminderType::OneDayBefore.as_str(),
            &one_day_iso,
            &expires_iso,
            &one_day_key,
            &now_iso,
        )
        .await?;
    }

    let one_hour_due = time::compute_one_hour_before_utc(start_utc);
    if one_hour_due > now_utc {
        let one_hour_iso = time::format_utc_iso(one_hour_due);
        let one_hour_key = format!(
            "{}:{}:{}",
            ReminderType::OneHourBefore.as_str(),
            appointment_id,
            one_hour_iso
        );
        reminder_repo::upsert_pending_reminder_tx(
            tx,
            &Uuid::new_v4().to_string(),
            appointment_id,
            ReminderType::OneHourBefore.as_str(),
            &one_hour_iso,
            &expires_iso,
            &one_hour_key,
            &now_iso,
        )
        .await?;
    }

    Ok(())
}

/// Recalculates pending reminders for all future scheduled appointments (for example,
/// when the operator changes the business timezone in Settings).
pub async fn recalculate_all_scheduled_reminders(pool: &SqlitePool) -> AppResult<u64> {
    let now_iso = time::now_utc_iso();
    let ids: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM appointments WHERE status = 'scheduled' AND start_utc > ?1",
    )
    .bind(&now_iso)
    .fetch_all(pool)
    .await?;

    let count = ids.len() as u64;
    if count == 0 {
        return Ok(0);
    }

    let mut tx = pool.begin().await?;
    for id in &ids {
        on_appointment_changed_tx(&mut tx, id).await?;
    }
    tx.commit().await?;

    tracing::info!(
        recalculated_appointments = count,
        "Recalculated scheduled reminders after timezone update"
    );
    Ok(count)
}

async fn summarize_services(pool: &SqlitePool, appointment_id: &str) -> AppResult<String> {
    let items = appointment_repo::list_services_for_appointment(pool, appointment_id).await?;
    let names: Vec<String> = items.into_iter().map(|s| s.service_name_snapshot).collect();
    if names.is_empty() {
        Ok("Appointment".to_string())
    } else {
        Ok(names.join(", "))
    }
}

fn format_local_time_short(local_dt: &str) -> &str {
    // Expects YYYY-MM-DDTHH:MM; returns HH:MM safely.
    local_dt.get(11..16).unwrap_or(local_dt)
}

fn build_timed_notification_content(
    reminder_type: &str,
    client_name: &str,
    start_local: &str,
    services_summary: &str,
) -> (String, String) {
    let time_part = format_local_time_short(start_local);
    let date_part = start_local.get(0..10).unwrap_or(start_local);

    if reminder_type == ReminderType::OneDayBefore.as_str() {
        (
            "Appointment Tomorrow".to_string(),
            format!("{client_name} — {date_part} at {time_part} ({services_summary})"),
        )
    } else {
        (
            "Appointment in 1 Hour".to_string(),
            format!("{client_name} — today at {time_part} ({services_summary})"),
        )
    }
}

/// Processes expired reminders and due timed reminders (`one_day_before`, `one_hour_before`).
/// Safe to invoke repeatedly on every scheduler tick without producing duplicates.
pub async fn process_scheduler_tick(
    pool: &SqlitePool,
    now_utc: DateTime<Utc>,
    sender: &dyn NotificationSender,
) -> AppResult<ReminderRunSummary> {
    let mut summary = ReminderRunSummary::default();
    let now_iso = time::format_utc_iso(now_utc);
    let settings = settings_service::get_settings(pool).await?;
    let business_tz = appointment_service::resolve_business_timezone(pool).await?;

    summary.expired_count =
        reminder_repo::expire_and_cleanup_stale_reminders(pool, &now_iso).await?;

    if !settings.reminders_enabled {
        return Ok(summary);
    }

    let due_rows = reminder_repo::list_due_pending_reminders(pool, &now_iso).await?;

    for row in due_rows {
        let enabled_for_type = match row.reminder_type.as_str() {
            "one_day_before" => settings.reminder_one_day_before,
            "one_hour_before" => settings.reminder_one_hour_before,
            _ => true,
        };
        if !enabled_for_type {
            continue;
        }
        let services_summary = summarize_services(pool, &row.appointment_id).await?;
        let start_local = time::format_utc_in_timezone(&row.appointment_start_utc, &business_tz)?;
        let (title, body) = build_timed_notification_content(
            &row.reminder_type,
            &row.client_name,
            &start_local,
            &services_summary,
        );

        let notif_id = Uuid::new_v4().to_string();
        let notif_key = format!("notif:{}", row.idempotency_key);

        let mut tx = pool.begin().await?;
        let marked = reminder_repo::mark_reminder_sent_tx(&mut tx, &row.id, &now_iso).await?;
        if !marked {
            tx.rollback().await?;
            continue;
        }

        let inserted = reminder_repo::insert_notification_if_new_tx(
            &mut tx,
            &notif_id,
            Some(&row.id),
            Some(&row.appointment_id),
            &row.reminder_type,
            &notif_key,
            &title,
            &body,
            &now_iso,
        )
        .await?;
        tx.commit().await?;

        if inserted {
            summary.timed_sent_count += 1;
            let outcome = if settings.os_notifications_enabled {
                sender.send(&title, &body)
            } else {
                NotificationDeliveryOutcome::Skipped(
                    "System notifications are disabled in Settings.".into(),
                )
            };
            match &outcome {
                NotificationDeliveryOutcome::Delivered => summary.os_delivered_count += 1,
                NotificationDeliveryOutcome::Failed(_) => summary.os_failed_count += 1,
                NotificationDeliveryOutcome::Skipped(_) => {}
            }
            reminder_repo::update_notification_os_delivery(
                pool,
                &notif_id,
                outcome.status_str(),
                outcome.error_message(),
            )
            .await?;

            tracing::info!(
                reminder_id = %row.id,
                appointment_id = %row.appointment_id,
                reminder_type = %row.reminder_type,
                os_status = outcome.status_str(),
                "Timed reminder processed"
            );
        }
    }

    Ok(summary)
}

/// Runs startup catch-up:
/// 1. Expires stale reminders and delivers any due non-expired timed reminders.
/// 2. Evaluates the **day-of first-launch** rule once per business-local calendar date
///    using `reminder_daily_runs`, notifying for scheduled appointments starting today
///    that have not yet ended (`end_utc > now_utc`).
pub async fn process_startup_catchup(
    pool: &SqlitePool,
    now_utc: DateTime<Utc>,
    sender: &dyn NotificationSender,
) -> AppResult<ReminderRunSummary> {
    let mut summary = process_scheduler_tick(pool, now_utc, sender).await?;
    let settings = settings_service::get_settings(pool).await?;
    if !settings.reminders_enabled || !settings.reminder_day_of_launch {
        return Ok(summary);
    }

    let business_tz = appointment_service::resolve_business_timezone(pool).await?;
    let today_local = time::local_date_for_utc(now_utc, &business_tz)?;
    let today_str = time::format_local_date(today_local);

    if reminder_repo::has_daily_run_completed(pool, &today_str).await? {
        return Ok(summary);
    }

    let (day_start_utc, day_end_utc_exclusive) =
        time::local_date_range_to_utc(today_local, today_local, &business_tz)?;
    let day_start_iso = time::format_utc_iso(day_start_utc);
    let day_end_iso = time::format_utc_iso(day_end_utc_exclusive);
    let now_iso = time::format_utc_iso(now_utc);

    let rows = appointment_repo::list_rows_in_utc_range(
        pool,
        &day_start_iso,
        &day_end_iso,
        Some(AppointmentStatus::Scheduled),
    )
    .await?;

    // Only include appointments starting on today_local that haven't already finished.
    let mut candidates = Vec::new();
    for row in rows {
        let start_dt = time::parse_utc_iso(&row.start_utc)?;
        let end_dt = time::parse_utc_iso(&row.end_utc)?;
        let appt_local_date = time::local_date_for_utc(start_dt, &business_tz)?;
        if appt_local_date == today_local && end_dt > now_utc {
            let services_summary = summarize_services(pool, &row.id).await?;
            let start_local = time::format_utc_in_timezone(&row.start_utc, &business_tz)?;
            candidates.push((row, start_local, services_summary));
        }
    }

    let mut newly_created: Vec<(String, String, String, String)> = Vec::new();
    let mut tx = pool.begin().await?;

    for (row, start_local, services_summary) in candidates {
        let time_part = format_local_time_short(&start_local);
        let title = "Appointment Today".to_string();
        let body = format!(
            "{} — today at {time_part} ({services_summary})",
            row.client_name
        );
        let notif_id = Uuid::new_v4().to_string();
        let key = format!(
            "notif:{}:{}:{}",
            NotificationKind::DayOfLaunch.as_str(),
            row.id,
            today_str
        );

        let inserted = reminder_repo::insert_notification_if_new_tx(
            &mut tx,
            &notif_id,
            None,
            Some(&row.id),
            NotificationKind::DayOfLaunch.as_str(),
            &key,
            &title,
            &body,
            &now_iso,
        )
        .await?;

        if inserted {
            newly_created.push((notif_id, row.id, title, body));
        }
    }

    reminder_repo::record_daily_run_tx(&mut tx, &today_str, &now_iso, newly_created.len() as i64)
        .await?;
    tx.commit().await?;

    for (notif_id, appointment_id, title, body) in newly_created {
        summary.day_of_sent_count += 1;
        let outcome = if settings.os_notifications_enabled {
            sender.send(&title, &body)
        } else {
            NotificationDeliveryOutcome::Skipped(
                "System notifications are disabled in Settings.".into(),
            )
        };
        match &outcome {
            NotificationDeliveryOutcome::Delivered => summary.os_delivered_count += 1,
            NotificationDeliveryOutcome::Failed(_) => summary.os_failed_count += 1,
            NotificationDeliveryOutcome::Skipped(_) => {}
        }
        reminder_repo::update_notification_os_delivery(
            pool,
            &notif_id,
            outcome.status_str(),
            outcome.error_message(),
        )
        .await?;

        tracing::info!(
            appointment_id = %appointment_id,
            local_date = %today_str,
            os_status = outcome.status_str(),
            "Day-of-launch reminder processed"
        );
    }

    tracing::info!(
        local_date = %today_str,
        expired = summary.expired_count,
        timed_sent = summary.timed_sent_count,
        day_of_sent = summary.day_of_sent_count,
        "Startup reminder catch-up completed"
    );

    Ok(summary)
}
