use chrono::Utc;
use sqlx::SqlitePool;

use crate::database::repositories::{appointment_repo, reminder_repo};
use crate::errors::AppResult;
use crate::models::reminder::{NotificationItem, ReminderRunSummary, ReminderSummary};
use crate::notifications::NotificationSender;
use crate::scheduler;
use crate::services::appointment_service;
use crate::time;

const MAX_NOTIFICATIONS_RETURNED: i64 = 100;
const MAX_UPCOMING_REMINDERS: i64 = 50;

pub async fn list_notifications(
    pool: &SqlitePool,
    include_dismissed: bool,
) -> AppResult<Vec<NotificationItem>> {
    reminder_repo::list_notifications(pool, include_dismissed, MAX_NOTIFICATIONS_RETURNED).await
}

pub async fn get_unread_count(pool: &SqlitePool) -> AppResult<i64> {
    reminder_repo::count_unread_notifications(pool).await
}

pub async fn mark_notification_read(pool: &SqlitePool, id: &str) -> AppResult<NotificationItem> {
    let now_iso = time::now_utc_iso();
    reminder_repo::mark_notification_read(pool, id.trim(), &now_iso).await
}

pub async fn mark_all_notifications_read(pool: &SqlitePool) -> AppResult<u64> {
    let now_iso = time::now_utc_iso();
    reminder_repo::mark_all_notifications_read(pool, &now_iso).await
}

pub async fn dismiss_notification(pool: &SqlitePool, id: &str) -> AppResult<NotificationItem> {
    let now_iso = time::now_utc_iso();
    reminder_repo::dismiss_notification(pool, id.trim(), &now_iso).await
}

pub async fn list_upcoming_reminders(pool: &SqlitePool) -> AppResult<Vec<ReminderSummary>> {
    let business_tz = appointment_service::resolve_business_timezone(pool).await?;
    let rows = reminder_repo::list_upcoming_pending_reminders(pool, MAX_UPCOMING_REMINDERS).await?;

    let mut summaries = Vec::with_capacity(rows.len());
    for row in rows {
        let services =
            appointment_repo::list_services_for_appointment(pool, &row.appointment_id).await?;
        let services_summary = if services.is_empty() {
            "Appointment".to_string()
        } else {
            services
                .into_iter()
                .map(|s| s.service_name_snapshot)
                .collect::<Vec<_>>()
                .join(", ")
        };

        let scheduled_for_local =
            time::format_utc_in_timezone(&row.scheduled_for_utc, &business_tz)?;
        let appointment_start_local =
            time::format_utc_in_timezone(&row.appointment_start_utc, &business_tz)?;

        summaries.push(ReminderSummary {
            id: row.id,
            appointment_id: row.appointment_id,
            reminder_type: row.reminder_type,
            scheduled_for_utc: row.scheduled_for_utc,
            scheduled_for_local,
            appointment_start_local,
            timezone: business_tz.clone(),
            status: row.status,
            client_name: row.client_name,
            services_summary,
        });
    }

    Ok(summaries)
}

pub async fn run_reminder_check_now(
    pool: &SqlitePool,
    sender: &dyn NotificationSender,
) -> AppResult<ReminderRunSummary> {
    scheduler::process_startup_catchup(pool, Utc::now(), sender).await
}
