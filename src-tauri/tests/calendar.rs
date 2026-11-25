use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::NaiveDate;
use my_schedule_app_lib::database;
use my_schedule_app_lib::models::appointment::{AppointmentStatus, CreateAppointmentInput};
use my_schedule_app_lib::models::business::CreateBusinessProfileInput;
use my_schedule_app_lib::models::calendar::{CalendarRangeInput, CalendarView};
use my_schedule_app_lib::models::client::CreateClientInput;
use my_schedule_app_lib::models::service::CreateServiceInput;
use my_schedule_app_lib::platform;
use my_schedule_app_lib::services::{
    appointment_service, business_service, calendar_service, client_service,
    service_catalog_service,
};
use my_schedule_app_lib::time::{self, DstDisambiguation};

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
            "my_schedule_app_cal_{label}_{}_{}_{}",
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

async fn setup_calendar_fixture(
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
            name: "Calendar Studio".into(),
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
            full_name: "Elena Vance".into(),
            phone: Some("+1 555 0100".into()),
            email: Some("elena@example.com".into()),
            address: None,
            notes: None,
        },
    )
    .await
    .expect("create client");

    let service = service_catalog_service::create_service(
        pool,
        CreateServiceInput {
            name: "Standard Session".into(),
            description: Some("60 minute session".into()),
            default_duration_minutes: 60,
        },
    )
    .await
    .expect("create service");

    (conn, client.id, service.id)
}

#[test]
fn calendar_date_range_conversion_and_dst_transitions() {
    // 1. View range calculations
    let wednesday = NaiveDate::from_ymd_opt(2026, 6, 17).unwrap();
    assert_eq!(
        calendar_service::compute_calendar_range(CalendarView::Day, wednesday).unwrap(),
        (wednesday, wednesday)
    );
    assert_eq!(
        calendar_service::compute_calendar_range(CalendarView::Week, wednesday).unwrap(),
        (
            NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 21).unwrap(),
        )
    );
    // June 2026 starts on Monday 2026-06-01 and ends on Tuesday 2026-06-30 -> grid ends Sunday 2026-07-05
    assert_eq!(
        calendar_service::compute_calendar_range(CalendarView::Month, wednesday).unwrap(),
        (
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 7, 5).unwrap(),
        )
    );
    assert_eq!(
        calendar_service::compute_calendar_range(CalendarView::Agenda, wednesday).unwrap(),
        (wednesday, NaiveDate::from_ymd_opt(2026, 6, 30).unwrap())
    );

    // 2. DST spring-forward day in Europe/Madrid (2026-03-29): 23-hour UTC span
    let spring_day = NaiveDate::from_ymd_opt(2026, 3, 29).unwrap();
    let (spring_from, spring_to) =
        time::local_date_range_to_utc(spring_day, spring_day, "Europe/Madrid").unwrap();
    assert_eq!((spring_to - spring_from).num_hours(), 23);

    // 3. DST fall-back day in Europe/Madrid (2026-10-25): 25-hour UTC span
    let fall_day = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
    let (fall_from, fall_to) =
        time::local_date_range_to_utc(fall_day, fall_day, "Europe/Madrid").unwrap();
    assert_eq!((fall_to - fall_from).num_hours(), 25);

    // 4. Midnight DST spring-forward in America/Havana (2026-03-08: 00:00 jumps to 01:00): 23-hour UTC span
    let havana_spring = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();
    let (hav_from, hav_to) =
        time::local_date_range_to_utc(havana_spring, havana_spring, "America/Havana").unwrap();
    assert_eq!((hav_to - hav_from).num_hours(), 23);
}

#[tokio::test]
async fn calendar_query_boundary_appointments_and_timezone_correctness() {
    let temp = TestDir::new("boundaries");
    let (conn, client_id, service_id) = setup_calendar_fixture(&temp, "America/New_York").await;
    let pool = &conn.require_pool().expect("pool");

    // 1. Ends exactly at local midnight 2026-06-15T00:00 -> belongs ONLY to June 14
    let ends_at_midnight = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-06-14T23:00".into(),
            end_local: "2026-06-15T00:00".into(),
            dst_disambiguation: None,
            service_ids: vec![service_id.clone()],
            notes: Some("Ends at midnight".into()),
        },
    )
    .await
    .expect("create ends_at_midnight")
    .appointment;

    // 2. Starts exactly at local midnight 2026-06-15T00:00 -> belongs ONLY to June 15
    let starts_at_midnight = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-06-15T00:00".into(),
            end_local: "2026-06-15T01:00".into(),
            dst_disambiguation: None,
            service_ids: vec![service_id.clone()],
            notes: Some("Starts at midnight".into()),
        },
    )
    .await
    .expect("create starts_at_midnight")
    .appointment;

    // 3. Late evening local 2026-06-15T22:00..23:00 (which is 2026-06-16T02:00Z..03:00Z in UTC!)
    // Must appear on June 15 in America/New_York, NOT June 16.
    let late_evening = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-06-15T22:00".into(),
            end_local: "2026-06-15T23:00".into(),
            dst_disambiguation: None,
            service_ids: vec![service_id.clone()],
            notes: Some("Late evening NY".into()),
        },
    )
    .await
    .expect("create late_evening")
    .appointment;

    // Mark late_evening completed to verify status filtering
    appointment_service::complete_appointment(pool, &late_evening.id)
        .await
        .expect("complete late_evening");

    // 4. Crosses midnight 2026-06-15T23:30..2026-06-16T00:30 -> appears on BOTH June 15 and June 16
    let crosses_midnight = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-06-15T23:30".into(),
            end_local: "2026-06-16T00:30".into(),
            dst_disambiguation: None,
            service_ids: vec![service_id.clone()],
            notes: Some("Crosses midnight".into()),
        },
    )
    .await
    .expect("create crosses_midnight")
    .appointment;

    // Query June 14
    let june_14 = calendar_service::get_calendar_range(
        pool,
        CalendarRangeInput {
            view: CalendarView::Day,
            anchor_date: Some("2026-06-14".into()),
            status: None,
        },
    )
    .await
    .expect("query june 14");
    assert_eq!(june_14.timezone, "America/New_York");
    let ids_14: Vec<&str> = june_14.appointments.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids_14, vec![ends_at_midnight.id.as_str()]);

    // Query June 15
    let june_15 = calendar_service::get_calendar_range(
        pool,
        CalendarRangeInput {
            view: CalendarView::Day,
            anchor_date: Some("2026-06-15".into()),
            status: None,
        },
    )
    .await
    .expect("query june 15");
    let ids_15: Vec<&str> = june_15.appointments.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(
        ids_15,
        vec![
            starts_at_midnight.id.as_str(),
            late_evening.id.as_str(),
            crosses_midnight.id.as_str(),
        ]
    );
    // Verify service snapshots are hydrated
    assert_eq!(
        june_15.appointments[0].services[0].service_name_snapshot,
        "Standard Session"
    );

    // Query June 15 with completed status filter
    let june_15_completed = calendar_service::get_calendar_range(
        pool,
        CalendarRangeInput {
            view: CalendarView::Day,
            anchor_date: Some("2026-06-15".into()),
            status: Some(AppointmentStatus::Completed),
        },
    )
    .await
    .expect("query june 15 completed");
    let ids_15_completed: Vec<&str> = june_15_completed
        .appointments
        .iter()
        .map(|a| a.id.as_str())
        .collect();
    assert_eq!(ids_15_completed, vec![late_evening.id.as_str()]);

    // Query June 16 -> ONLY crosses_midnight (late_evening ended at 23:00 local June 15)
    let june_16 = calendar_service::get_calendar_range(
        pool,
        CalendarRangeInput {
            view: CalendarView::Day,
            anchor_date: Some("2026-06-16".into()),
            status: None,
        },
    )
    .await
    .expect("query june 16");
    let ids_16: Vec<&str> = june_16.appointments.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids_16, vec![crosses_midnight.id.as_str()]);
}

#[tokio::test]
async fn calendar_views_week_month_agenda_across_dst_fall_back() {
    let temp = TestDir::new("dst_views");
    let (conn, client_id, service_id) = setup_calendar_fixture(&temp, "Europe/Madrid").await;
    let pool = &conn.require_pool().expect("pool");

    // Create both earlier and later ambiguous appointments on DST fall-back day 2026-10-25
    let earlier = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-10-25T02:00".into(),
            end_local: "2026-10-25T02:45".into(),
            dst_disambiguation: Some(DstDisambiguation::Earlier),
            service_ids: vec![service_id.clone()],
            notes: None,
        },
    )
    .await
    .expect("create earlier")
    .appointment;

    let later = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-10-25T02:00".into(),
            end_local: "2026-10-25T02:45".into(),
            dst_disambiguation: Some(DstDisambiguation::Later),
            service_ids: vec![service_id.clone()],
            notes: None,
        },
    )
    .await
    .expect("create later")
    .appointment;

    // Week view containing Sunday 2026-10-25 (Mon 2026-10-19 .. Sun 2026-10-25)
    let week = calendar_service::get_calendar_range(
        pool,
        CalendarRangeInput {
            view: CalendarView::Week,
            anchor_date: Some("2026-10-25".into()),
            status: None,
        },
    )
    .await
    .expect("week query");
    assert_eq!(week.range_start_local, "2026-10-19");
    assert_eq!(week.range_end_local, "2026-10-25");
    assert_eq!(week.appointments.len(), 2);
    assert_eq!(week.appointments[0].id, earlier.id);
    assert_eq!(week.appointments[1].id, later.id);

    // Month view for October 2026
    let month = calendar_service::get_calendar_range(
        pool,
        CalendarRangeInput {
            view: CalendarView::Month,
            anchor_date: Some("2026-10-15".into()),
            status: None,
        },
    )
    .await
    .expect("month query");
    assert_eq!(month.range_start_local, "2026-09-28");
    assert_eq!(month.range_end_local, "2026-11-01");
    assert_eq!(month.appointments.len(), 2);

    // Agenda view starting 2026-10-20 (14 days -> 2026-11-02)
    let agenda = calendar_service::get_calendar_range(
        pool,
        CalendarRangeInput {
            view: CalendarView::Agenda,
            anchor_date: Some("2026-10-20".into()),
            status: None,
        },
    )
    .await
    .expect("agenda query");
    assert_eq!(agenda.range_start_local, "2026-10-20");
    assert_eq!(agenda.range_end_local, "2026-11-02");
    assert_eq!(agenda.appointments.len(), 2);
}
