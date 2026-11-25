use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::NaiveDate;
use my_schedule_app_lib::database;
use my_schedule_app_lib::models::appointment::CreateAppointmentInput;
use my_schedule_app_lib::models::business::CreateBusinessProfileInput;
use my_schedule_app_lib::models::client::CreateClientInput;
use my_schedule_app_lib::models::reports::{
    DashboardPeriod, OperationalReportInput, ReportPeriodType,
};
use my_schedule_app_lib::models::service::CreateServiceInput;
use my_schedule_app_lib::models::session_plan::CreateSessionPlanInput;
use my_schedule_app_lib::platform;
use my_schedule_app_lib::services::{
    appointment_service, business_service, client_service, report_service, service_catalog_service,
    session_plan_service,
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
            "my_schedule_app_rep_{label}_{}_{}_{}",
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

async fn setup_report_fixture(
    temp: &TestDir,
    timezone: &str,
) -> (
    database::connection::DatabaseConnection,
    String,
    String,
    String,
) {
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
            name: "Reports Studio".into(),
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

    let c1 = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "Client One".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect("create c1");

    let c2 = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "Client Two".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect("create c2");

    let svc = service_catalog_service::create_service(
        pool,
        CreateServiceInput {
            name: "Core Consultation".into(),
            description: None,
            default_duration_minutes: 60,
        },
    )
    .await
    .expect("create svc");

    (conn, c1.id, c2.id, svc.id)
}

#[tokio::test]
async fn report_period_boundaries_empty_periods_and_dst_transitions() {
    // 1. Period calculation sanity checks
    assert_eq!(
        report_service::compute_report_period(
            ReportPeriodType::Monthly,
            NaiveDate::from_ymd_opt(2024, 2, 10).unwrap()
        )
        .unwrap(),
        (
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
        )
    );
    assert_eq!(
        report_service::compute_report_period(
            ReportPeriodType::Monthly,
            NaiveDate::from_ymd_opt(2026, 12, 15).unwrap()
        )
        .unwrap(),
        (
            NaiveDate::from_ymd_opt(2026, 12, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        )
    );

    // 2. Empty weekly and monthly reports in Europe/Madrid across DST fall-back week
    let temp = TestDir::new("empty_dst");
    let (conn, _c1, _c2, _svc) = setup_report_fixture(&temp, "Europe/Madrid").await;
    let pool = &conn.require_pool().expect("pool");

    let now =
        time::resolve_local_datetime_to_utc("2026-10-25T12:00", "Europe/Madrid", None).unwrap();

    // Week containing Sunday 2026-10-25 (DST fall-back) -> 7 days = 169 UTC hours
    let empty_week = report_service::get_operational_report_at(
        pool,
        OperationalReportInput {
            period_type: ReportPeriodType::Weekly,
            anchor_date: Some("2026-10-25".into()),
        },
        now,
    )
    .await
    .expect("empty weekly report");

    assert_eq!(empty_week.period_start_local, "2026-10-19");
    assert_eq!(empty_week.period_end_local, "2026-10-25");
    assert_eq!(empty_week.counts.total, 0);
    assert_eq!(empty_week.completion_rate_percent, None);
    assert_eq!(empty_week.total_active_minutes, 0);
    assert_eq!(empty_week.unique_clients_served, 0);
    assert_eq!(empty_week.daily_breakdown.len(), 7);
    assert!(empty_week.top_services.is_empty());

    let start_utc = time::parse_utc_iso(&empty_week.period_start_utc).unwrap();
    let end_utc = time::parse_utc_iso(&empty_week.period_end_utc_exclusive).unwrap();
    assert_eq!((end_utc - start_utc).num_hours(), 169);

    let empty_month = report_service::get_operational_report_at(
        pool,
        OperationalReportInput {
            period_type: ReportPeriodType::Monthly,
            anchor_date: Some("2026-10-15".into()),
        },
        now,
    )
    .await
    .expect("empty monthly report");
    assert_eq!(empty_month.daily_breakdown.len(), 31);
    assert_eq!(empty_month.counts.total, 0);
}

#[tokio::test]
async fn dashboard_and_operational_status_aggregation_excludes_cancelled_from_completed_and_upcoming(
) {
    let temp = TestDir::new("aggregation");
    let (conn, c1_id, c2_id, svc_id) = setup_report_fixture(&temp, "America/New_York").await;
    let pool = &conn.require_pool().expect("pool");

    // Create 1 active session plan
    session_plan_service::create_session_plan(
        pool,
        CreateSessionPlanInput {
            client_id: c1_id.clone(),
            title: "Onboarding Package".into(),
            description: None,
            planned_sessions: 4,
        },
    )
    .await
    .expect("create plan");

    // Reference "now": Wednesday 2026-06-17 at 12:00 America/New_York
    let now =
        time::resolve_local_datetime_to_utc("2026-06-17T12:00", "America/New_York", None).unwrap();

    // 1. Past-end scheduled appointment (Tuesday 2026-06-16 09:00..10:00) -> needs_outcome_count = 1
    appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(c1_id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-06-16T09:00".into(),
            end_local: "2026-06-16T10:00".into(),
            dst_disambiguation: None,
            service_ids: vec![svc_id.clone()],
            notes: None,
        },
    )
    .await
    .expect("create past-due scheduled");

    // 2. Today completed appointment (2026-06-17 09:00..10:00)
    let today_completed = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(c1_id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-06-17T09:00".into(),
            end_local: "2026-06-17T10:00".into(),
            dst_disambiguation: None,
            service_ids: vec![svc_id.clone()],
            notes: None,
        },
    )
    .await
    .expect("create today completed")
    .appointment;
    appointment_service::complete_appointment(pool, &today_completed.id)
        .await
        .expect("complete");

    // 3. Today cancelled appointment for Client Two (2026-06-17 15:00..16:00) ->
    // MUST NOT count as completed, upcoming, active time, unique clients served, or top services!
    let today_cancelled = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(c2_id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-06-17T15:00".into(),
            end_local: "2026-06-17T16:00".into(),
            dst_disambiguation: None,
            service_ids: vec![svc_id.clone()],
            notes: None,
        },
    )
    .await
    .expect("create today cancelled")
    .appointment;
    appointment_service::cancel_appointment(pool, &today_cancelled.id)
        .await
        .expect("cancel");

    // 4. Late Sunday night in NY (2026-06-21T22:00..23:00 = Monday 2026-06-22T02:00Z in UTC!)
    // Must be counted in week 2026-06-15..2026-06-21 and in upcoming_scheduled_count.
    let sunday_night = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(c1_id),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-06-21T22:00".into(),
            end_local: "2026-06-21T23:00".into(),
            dst_disambiguation: None,
            service_ids: vec![svc_id],
            notes: None,
        },
    )
    .await
    .expect("create sunday night")
    .appointment;

    // Verify Dashboard summary
    let dash = report_service::get_dashboard_summary_at(pool, DashboardPeriod::ThisWeek, now)
        .await
        .expect("dashboard");

    assert_eq!(dash.timezone, "America/New_York");
    assert_eq!(dash.today_local, "2026-06-17");
    assert_eq!(dash.today_counts.total, 2);
    assert_eq!(dash.today_counts.completed, 1);
    assert_eq!(dash.today_counts.cancelled, 1);
    assert_eq!(dash.today_counts.scheduled, 0);

    assert_eq!(dash.period_counts.total, 4);
    assert_eq!(dash.period_counts.scheduled, 2);
    assert_eq!(dash.period_counts.completed, 1);
    assert_eq!(dash.period_counts.cancelled, 1);

    assert_eq!(dash.upcoming_scheduled_count, 1);
    assert_eq!(dash.upcoming_appointments.len(), 1);
    assert_eq!(dash.upcoming_appointments[0].id, sunday_night.id);
    assert_eq!(dash.needs_outcome_count, 1);
    assert_eq!(dash.active_clients_count, 2);
    assert_eq!(dash.active_session_plans_count, 1);

    // Verify Weekly Operational Report for 2026-06-15..2026-06-21
    let week_report = report_service::get_operational_report_at(
        pool,
        OperationalReportInput {
            period_type: ReportPeriodType::Weekly,
            anchor_date: Some("2026-06-17".into()),
        },
        now,
    )
    .await
    .expect("weekly report");

    assert_eq!(week_report.counts.total, 4);
    assert_eq!(week_report.counts.scheduled, 2);
    assert_eq!(week_report.counts.completed, 1);
    assert_eq!(week_report.counts.cancelled, 1);
    assert_eq!(week_report.completion_rate_percent, Some(25.0));
    // 3 non-cancelled 60-min sessions = 180 minutes (cancelled excluded)
    assert_eq!(week_report.total_active_minutes, 180);
    // Only Client One had non-cancelled appointments (Client Two's was cancelled)
    assert_eq!(week_report.unique_clients_served, 1);
    assert_eq!(week_report.top_services.len(), 1);
    assert_eq!(
        week_report.top_services[0].service_name,
        "Core Consultation"
    );
    assert_eq!(week_report.top_services[0].appointment_count, 3);
    assert_eq!(week_report.top_services[0].total_minutes, 180);

    // Verify the next week (2026-06-22..2026-06-28) does NOT double-count Sunday night NY
    let next_week = report_service::get_operational_report_at(
        pool,
        OperationalReportInput {
            period_type: ReportPeriodType::Weekly,
            anchor_date: Some("2026-06-22".into()),
        },
        now,
    )
    .await
    .expect("next week report");
    assert_eq!(next_week.counts.total, 0);
}
