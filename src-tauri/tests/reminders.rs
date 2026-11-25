use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Duration, Utc};
use my_schedule_app_lib::database;
use my_schedule_app_lib::models::appointment::{
    CreateAppointmentInput, RescheduleAppointmentInput,
};
use my_schedule_app_lib::models::business::CreateBusinessProfileInput;
use my_schedule_app_lib::models::client::CreateClientInput;
use my_schedule_app_lib::models::service::CreateServiceInput;
use my_schedule_app_lib::notifications::{
    NoopNotificationSender, NotificationDeliveryOutcome, NotificationSender,
};
use my_schedule_app_lib::platform;
use my_schedule_app_lib::scheduler;
use my_schedule_app_lib::services::{
    appointment_service, business_service, client_service, reminder_service,
    service_catalog_service,
};
use my_schedule_app_lib::time;

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "my_schedule_app_rem_{label}_{}_{}_{}",
            std::process::id(),
            nanos,
            seq
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        Self { path: dir }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct FailingNotificationSender;

impl NotificationSender for FailingNotificationSender {
    fn send(&self, _title: &str, _body: &str) -> NotificationDeliveryOutcome {
        NotificationDeliveryOutcome::Failed("OS notification permission denied.".into())
    }
}

async fn setup_reminder_fixture(
    temp: &TestDir,
    timezone: &str,
) -> (database::connection::DatabaseConnection, String, String) {
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");
    let (conn, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool = &conn.require_pool().expect("pool");

    business_service::create_business_profile(
        pool,
        &paths.logos_dir,
        CreateBusinessProfileInput {
            name: "Reminder Studio".into(),
            address: None,
            city: None,
            region: None,
            postal_code: None,
            country: None,
            phone: None,
            email: None,
            website: None,
            timezone: timezone.into(),
            logo_bytes: None,
        },
    )
    .await
    .expect("create business");

    let client = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "Marcus Holloway".into(),
            phone: Some("+34 600 000 001".into()),
            email: None,
            address: None,
            notes: Some("Private note that must not leak".into()),
        },
    )
    .await
    .expect("create client");

    let service = service_catalog_service::create_service(
        pool,
        CreateServiceInput {
            name: "Strategy Review".into(),
            description: None,
            default_duration_minutes: 60,
        },
    )
    .await
    .expect("create service");

    (conn, client.id, service.id)
}

#[test]
fn due_time_calculation_and_dst_transitions() {
    // 1. Standard day: 2026-06-15T10:00 Europe/Madrid (UTC+2) -> 2026-06-14T10:00 Europe/Madrid (08:00Z)
    let start_std =
        time::resolve_local_datetime_to_utc("2026-06-15T10:00", "Europe/Madrid", None).unwrap();
    let one_day_std = time::compute_one_day_before_utc(start_std, "Europe/Madrid").unwrap();
    assert_eq!(
        time::format_utc_in_timezone(&time::format_utc_iso(one_day_std), "Europe/Madrid").unwrap(),
        "2026-06-14T10:00"
    );

    // 2. Preceding day falls into DST spring-forward gap:
    // Appointment at 2026-03-30T02:30 Europe/Madrid (CEST, UTC+2).
    // Preceding calendar day 2026-03-29T02:30 does not exist (02:00 jumps to 03:00 CEST).
    // Policy advances to first valid local instant: 2026-03-29T03:00 CEST (01:00Z).
    let start_after_spring =
        time::resolve_local_datetime_to_utc("2026-03-30T02:30", "Europe/Madrid", None).unwrap();
    let one_day_spring =
        time::compute_one_day_before_utc(start_after_spring, "Europe/Madrid").unwrap();
    assert_eq!(
        time::format_utc_in_timezone(&time::format_utc_iso(one_day_spring), "Europe/Madrid")
            .unwrap(),
        "2026-03-29T03:00"
    );

    // 3. Preceding day falls into DST fall-back ambiguity:
    // Appointment at 2026-10-26T02:30 Europe/Madrid (CET, UTC+1).
    // Preceding day 2026-10-25T02:30 occurs twice; policy picks earlier occurrence (00:30Z CEST).
    let start_after_fall =
        time::resolve_local_datetime_to_utc("2026-10-26T02:30", "Europe/Madrid", None).unwrap();
    let one_day_fall = time::compute_one_day_before_utc(start_after_fall, "Europe/Madrid").unwrap();
    assert_eq!(
        time::format_utc_iso(one_day_fall),
        "2026-10-25T00:30:00.000Z"
    );

    // 4. One hour before across DST spring-forward:
    // Appointment at 2026-03-29T03:30 CEST (01:30Z) -> 1 real hour before is 00:30Z (01:30 CET).
    let start_spring_day =
        time::resolve_local_datetime_to_utc("2026-03-29T03:30", "Europe/Madrid", None).unwrap();
    let one_hour_spring = time::compute_one_hour_before_utc(start_spring_day);
    assert_eq!(
        time::format_utc_in_timezone(&time::format_utc_iso(one_hour_spring), "Europe/Madrid")
            .unwrap(),
        "2026-03-29T01:30"
    );
}

#[tokio::test]
async fn appointment_lifecycle_reschedule_cancel_complete_and_stale_expiry() {
    let temp = TestDir::new("lifecycle");
    let (conn, client_id, service_id) = setup_reminder_fixture(&temp, "Europe/Madrid").await;
    let pool = &conn.require_pool().expect("pool");

    // Schedule an appointment 5 days in the future so both reminders are pending
    let future_base = Utc::now() + Duration::days(5);
    let start_local_1 =
        time::format_utc_in_timezone(&time::format_utc_iso(future_base), "Europe/Madrid").unwrap();
    let end_local_1 = time::format_utc_in_timezone(
        &time::format_utc_iso(future_base + Duration::hours(1)),
        "Europe/Madrid",
    )
    .unwrap();

    let appt1 = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: start_local_1,
            end_local: end_local_1,
            dst_disambiguation: None,
            service_ids: vec![service_id.clone()],
            notes: None,
        },
    )
    .await
    .expect("create appt1")
    .appointment;

    let upcoming = reminder_service::list_upcoming_reminders(pool)
        .await
        .expect("list upcoming");
    assert_eq!(upcoming.len(), 2);

    // Reschedule appt1 to +7 days -> old pending reminders cancelled, 2 new pending reminders created
    let rescheduled_base = Utc::now() + Duration::days(7);
    let start_local_2 =
        time::format_utc_in_timezone(&time::format_utc_iso(rescheduled_base), "Europe/Madrid")
            .unwrap();
    let end_local_2 = time::format_utc_in_timezone(
        &time::format_utc_iso(rescheduled_base + Duration::hours(1)),
        "Europe/Madrid",
    )
    .unwrap();

    let rescheduled = appointment_service::reschedule_appointment(
        pool,
        RescheduleAppointmentInput {
            id: appt1.id.clone(),
            start_local: start_local_2,
            end_local: end_local_2,
            dst_disambiguation: None,
        },
    )
    .await
    .expect("reschedule appt1")
    .appointment;

    let upcoming_after_resched = reminder_service::list_upcoming_reminders(pool)
        .await
        .expect("list upcoming after reschedule");
    assert_eq!(upcoming_after_resched.len(), 2);
    assert!(upcoming_after_resched
        .iter()
        .all(|r| r.appointment_start_local == rescheduled.start_local));

    // Cancel appt1 -> all pending reminders invalidated
    appointment_service::cancel_appointment(pool, &appt1.id)
        .await
        .expect("cancel appt1");
    let upcoming_after_cancel = reminder_service::list_upcoming_reminders(pool)
        .await
        .expect("list upcoming after cancel");
    assert!(upcoming_after_cancel.is_empty());

    // Create appt2 and complete it -> reminders invalidated
    let appt2 = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: rescheduled.start_local.clone(),
            end_local: rescheduled.end_local.clone(),
            dst_disambiguation: None,
            service_ids: vec![service_id.clone()],
            notes: None,
        },
    )
    .await
    .expect("create appt2")
    .appointment;
    assert_eq!(
        reminder_service::list_upcoming_reminders(pool)
            .await
            .unwrap()
            .len(),
        2
    );

    appointment_service::complete_appointment(pool, &appt2.id)
        .await
        .expect("complete appt2");
    assert!(reminder_service::list_upcoming_reminders(pool)
        .await
        .unwrap()
        .is_empty());

    // Create appt3 and test stale expiry: if now is after appointment start_utc, reminders expire without sending
    let appt3 = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_id),
            inline_client: None,
            session_plan_id: None,
            start_local: rescheduled.start_local,
            end_local: rescheduled.end_local,
            dst_disambiguation: None,
            service_ids: vec![service_id],
            notes: None,
        },
    )
    .await
    .expect("create appt3")
    .appointment;

    let appt3_start = time::parse_utc_iso(&appt3.start_utc).unwrap();
    let after_start = appt3_start + Duration::minutes(10);
    let summary = scheduler::process_scheduler_tick(pool, after_start, &NoopNotificationSender)
        .await
        .expect("stale tick");
    assert_eq!(summary.expired_count, 2);
    assert_eq!(summary.timed_sent_count, 0);
    assert!(reminder_service::list_notifications(pool, true)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn catchup_duplicate_prevention_day_of_once_per_day_and_os_failure_resilience() {
    let temp = TestDir::new("catchup");
    let (conn, client_id, service_id) = setup_reminder_fixture(&temp, "Europe/Madrid").await;
    let pool = &conn.require_pool().expect("pool");

    // Schedule an appointment 10 days from now at 14:00 business-local time
    let target_day = (Utc::now() + Duration::days(10)).date_naive();
    let start_local = format!("{}T14:00", target_day.format("%Y-%m-%d"));
    let end_local = format!("{}T15:00", target_day.format("%Y-%m-%d"));

    let appt = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_id),
            inline_client: None,
            session_plan_id: None,
            start_local,
            end_local,
            dst_disambiguation: None,
            service_ids: vec![service_id],
            notes: None,
        },
    )
    .await
    .expect("create appt")
    .appointment;

    let start_utc = time::parse_utc_iso(&appt.start_utc).unwrap();
    let one_day_due = time::compute_one_day_before_utc(start_utc, "Europe/Madrid").unwrap();
    let one_hour_due = time::compute_one_hour_before_utc(start_utc);

    // 1. Trigger 1-day-before reminder with FailingNotificationSender -> in-app notification still saved!
    let run1 = scheduler::process_scheduler_tick(
        pool,
        one_day_due + Duration::minutes(5),
        &FailingNotificationSender,
    )
    .await
    .expect("one-day tick");
    assert_eq!(run1.timed_sent_count, 1);
    assert_eq!(run1.os_failed_count, 1);

    // Repeated tick at the same time -> zero duplicates
    let run1_dup = scheduler::process_scheduler_tick(
        pool,
        one_day_due + Duration::minutes(6),
        &FailingNotificationSender,
    )
    .await
    .expect("dup tick");
    assert_eq!(run1_dup.timed_sent_count, 0);

    // 2. First app launch on the appointment's business-local day (09:00 local) -> fires day_of_launch once
    let morning_of_day = time::resolve_local_datetime_to_utc(
        &format!("{}T09:00", target_day.format("%Y-%m-%d")),
        "Europe/Madrid",
        None,
    )
    .unwrap();
    let launch1 = scheduler::process_startup_catchup(pool, morning_of_day, &NoopNotificationSender)
        .await
        .expect("first launch of day");
    assert_eq!(launch1.day_of_sent_count, 1);
    assert_eq!(launch1.timed_sent_count, 0);

    // Second launch later on the same local day (10:30 local) -> day_of_launch NOT duplicated
    let launch2 = scheduler::process_startup_catchup(
        pool,
        morning_of_day + Duration::minutes(90),
        &NoopNotificationSender,
    )
    .await
    .expect("second launch same day");
    assert_eq!(launch2.day_of_sent_count, 0);

    // 3. One-hour-before tick -> fires once, no duplicates on repeat
    let run_1h = scheduler::process_scheduler_tick(
        pool,
        one_hour_due + Duration::minutes(1),
        &NoopNotificationSender,
    )
    .await
    .expect("one-hour tick");
    assert_eq!(run_1h.timed_sent_count, 1);

    let run_1h_dup = scheduler::process_scheduler_tick(
        pool,
        one_hour_due + Duration::minutes(2),
        &NoopNotificationSender,
    )
    .await
    .expect("one-hour dup tick");
    assert_eq!(run_1h_dup.timed_sent_count, 0);

    // Total persisted notifications = 3 (one_day_before, day_of_launch, one_hour_before)
    let all = reminder_service::list_notifications(pool, false)
        .await
        .expect("list notifications");
    assert_eq!(all.len(), 3);
    assert_eq!(reminder_service::get_unread_count(pool).await.unwrap(), 3);

    // Verify OS failure state was recorded cleanly on the one_day_before notification
    let failed_item = all
        .iter()
        .find(|n| n.kind == "one_day_before")
        .expect("one_day_before item");
    assert_eq!(failed_item.os_delivery_status, "failed");
    assert!(failed_item.os_delivery_error.is_some());

    // Verify read & dismiss state transitions
    let marked = reminder_service::mark_notification_read(pool, &all[0].id)
        .await
        .expect("mark read");
    assert!(marked.is_read);
    assert_eq!(reminder_service::get_unread_count(pool).await.unwrap(), 2);

    let dismissed = reminder_service::dismiss_notification(pool, &all[1].id)
        .await
        .expect("dismiss");
    assert!(dismissed.is_dismissed);
    assert_eq!(
        reminder_service::list_notifications(pool, false)
            .await
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        reminder_service::list_notifications(pool, true)
            .await
            .unwrap()
            .len(),
        3
    );

    let remaining_marked = reminder_service::mark_all_notifications_read(pool)
        .await
        .expect("mark all read");
    assert_eq!(remaining_marked, 1);
    assert_eq!(reminder_service::get_unread_count(pool).await.unwrap(), 0);
}

#[tokio::test]
async fn same_day_booking_does_not_schedule_past_one_day_before_reminder() {
    let temp = TestDir::new("same_day_booking");
    let (conn, client_id, service_id) = setup_reminder_fixture(&temp, "Europe/Madrid").await;
    let pool = &conn.require_pool().expect("pool");

    // Book an appointment 3 hours from now -> 1-day-before is in the past, only 1-hour-before should be scheduled.
    let soon_start = Utc::now() + Duration::hours(3);
    let start_local =
        time::format_utc_in_timezone(&time::format_utc_iso(soon_start), "Europe/Madrid").unwrap();
    let end_local = time::format_utc_in_timezone(
        &time::format_utc_iso(soon_start + Duration::hours(1)),
        "Europe/Madrid",
    )
    .unwrap();

    appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_id),
            inline_client: None,
            session_plan_id: None,
            start_local,
            end_local,
            dst_disambiguation: None,
            service_ids: vec![service_id],
            notes: None,
        },
    )
    .await
    .expect("create same-day appt");

    let upcoming = reminder_service::list_upcoming_reminders(pool)
        .await
        .expect("list upcoming");
    assert_eq!(upcoming.len(), 1);
    assert_eq!(upcoming[0].reminder_type, "one_hour_before");
}
