use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use my_schedule_app_lib::database;
use my_schedule_app_lib::errors::AppError;
use my_schedule_app_lib::models::appointment::{
    CreateAppointmentInput, ListAppointmentsParams, RescheduleAppointmentInput,
};
use my_schedule_app_lib::models::business::CreateBusinessProfileInput;
use my_schedule_app_lib::models::client::{CreateClientInput, ListClientsParams};
use my_schedule_app_lib::models::service::{CreateServiceInput, UpdateServiceInput};
use my_schedule_app_lib::models::session_plan::{
    CreateSessionPlanInput, CreateSessionPlanWithAppointmentsInput, PlanSessionSlotInput,
};
use my_schedule_app_lib::platform;
use my_schedule_app_lib::services::{
    appointment_service, business_service, client_service, service_catalog_service,
    session_plan_service,
};
use my_schedule_app_lib::time::DstDisambiguation;

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
            "my_schedule_app_appt_{label}_{}_{}_{}",
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

async fn setup_business_and_services(
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
            name: "Studio Test".into(),
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

    let s1 = service_catalog_service::create_service(
        pool,
        CreateServiceInput {
            name: "Consultation".into(),
            description: Some("Initial meeting".into()),
            default_duration_minutes: 60,
        },
    )
    .await
    .expect("create s1");

    let s2 = service_catalog_service::create_service(
        pool,
        CreateServiceInput {
            name: "Follow-Up".into(),
            description: None,
            default_duration_minutes: 30,
        },
    )
    .await
    .expect("create s2");

    (conn, s1.id, s2.id)
}

#[tokio::test]
async fn inline_client_creation_multi_service_snapshots_and_archive_preservation() {
    let temp = TestDir::new("inline_snapshots");
    let (conn, s1_id, s2_id) = setup_business_and_services(&temp, "Europe/Madrid").await;
    let pool = &conn.require_pool().expect("pool");

    // Inline client creation fails transactionally if services are empty -> no orphan client created
    let bad_err = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: None,
            inline_client: Some(CreateClientInput {
                full_name: "Orphan Client".into(),
                phone: None,
                email: None,
                address: None,
                notes: None,
            }),
            session_plan_id: None,
            start_local: "2026-11-26T10:00".into(),
            end_local: "2026-11-26T11:00".into(),
            dst_disambiguation: None,
            service_ids: vec![],
            notes: None,
        },
    )
    .await
    .expect_err("empty service list must fail");
    assert!(matches!(bad_err, AppError::Validation(_)));

    let clients_after_fail = client_service::list_clients(pool, ListClientsParams::default())
        .await
        .expect("list clients");
    assert_eq!(clients_after_fail.total, 0);

    // Valid inline client creation with 2 services
    let result = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: None,
            inline_client: Some(CreateClientInput {
                full_name: "Lucia Gomez".into(),
                phone: Some("+34 600 111 222".into()),
                email: None,
                address: None,
                notes: None,
            }),
            session_plan_id: None,
            start_local: "2026-11-26T10:00".into(),
            end_local: "2026-11-26T11:30".into(),
            dst_disambiguation: None,
            service_ids: vec![s1_id.clone(), s2_id.clone()],
            notes: Some("Bring notes".into()),
        },
    )
    .await
    .expect("create appointment with inline client");

    assert!(result.overlap_warnings.is_empty());
    let appt = result.appointment;
    assert_eq!(appt.client_name, "Lucia Gomez");
    assert_eq!(appt.start_local, "2026-11-26T10:00");
    assert_eq!(appt.end_local, "2026-11-26T11:30");
    assert_eq!(appt.start_utc, "2026-11-26T09:00:00.000Z"); // Europe/Madrid is UTC+1 in Nov
    assert_eq!(appt.services.len(), 2);
    assert_eq!(appt.services[0].service_name_snapshot, "Consultation");
    assert_eq!(appt.services[1].service_name_snapshot, "Follow-Up");

    // Rename and deactivate s1, archive client -> historical appointment still shows original snapshots
    service_catalog_service::update_service(
        pool,
        UpdateServiceInput {
            id: s1_id.clone(),
            name: "Renamed Service".into(),
            description: None,
            default_duration_minutes: 90,
        },
    )
    .await
    .expect("rename s1");
    service_catalog_service::deactivate_service(pool, &s1_id)
        .await
        .expect("deactivate s1");
    client_service::archive_client(pool, &appt.client_id)
        .await
        .expect("archive client");

    let reloaded = appointment_service::get_appointment(pool, &appt.id)
        .await
        .expect("reload historical appointment");
    assert_eq!(reloaded.client_name, "Lucia Gomez");
    assert!(reloaded.client_is_archived);
    assert_eq!(reloaded.services[0].service_name_snapshot, "Consultation");
    assert_eq!(reloaded.services[0].duration_minutes_snapshot, 60);

    conn.close().await;
}

#[tokio::test]
async fn dst_gap_ambiguity_and_time_bounds_are_handled_deliberately() {
    let temp = TestDir::new("dst_validation");
    let (conn, s1_id, _) = setup_business_and_services(&temp, "Europe/Madrid").await;
    let pool = &conn.require_pool().expect("pool");

    let client = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "Diego Silva".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect("create client");

    // End before start rejected
    let err = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client.id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-06-10T11:00".into(),
            end_local: "2026-06-10T10:00".into(),
            dst_disambiguation: None,
            service_ids: vec![s1_id.clone()],
            notes: None,
        },
    )
    .await
    .expect_err("end <= start must fail");
    assert!(matches!(err, AppError::Validation(_)));

    // DST spring-forward nonexistent local time (2026-03-29 02:30 in Europe/Madrid)
    let gap_err = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client.id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-03-29T02:30".into(),
            end_local: "2026-03-29T03:30".into(),
            dst_disambiguation: None,
            service_ids: vec![s1_id.clone()],
            notes: None,
        },
    )
    .await
    .expect_err("DST gap must be rejected without silently shifting");
    assert!(matches!(gap_err, AppError::Validation(_)));

    // DST fall-back ambiguous time (2026-10-25 02:30 in Europe/Madrid) without disambiguation
    let ambig_err = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client.id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-10-25T02:30".into(),
            end_local: "2026-10-25T03:30".into(),
            dst_disambiguation: None,
            service_ids: vec![s1_id.clone()],
            notes: None,
        },
    )
    .await
    .expect_err("DST ambiguity without choice must fail");
    assert!(matches!(ambig_err, AppError::Validation(_)));

    // Same ambiguous time with explicit Earlier vs Later disambiguation
    let earlier = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client.id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-10-25T02:30".into(),
            end_local: "2026-10-25T03:30".into(),
            dst_disambiguation: Some(DstDisambiguation::Earlier),
            service_ids: vec![s1_id.clone()],
            notes: None,
        },
    )
    .await
    .expect("earlier ambiguous time should succeed");
    assert_eq!(earlier.appointment.start_utc, "2026-10-25T00:30:00.000Z"); // CEST UTC+2

    let later = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client.id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-10-25T02:30".into(),
            end_local: "2026-10-25T03:30".into(),
            dst_disambiguation: Some(DstDisambiguation::Later),
            service_ids: vec![s1_id],
            notes: None,
        },
    )
    .await
    .expect("later ambiguous time should succeed");
    assert_eq!(later.appointment.start_utc, "2026-10-25T01:30:00.000Z"); // CET UTC+1

    conn.close().await;
}

#[tokio::test]
async fn session_plan_consistency_status_transitions_reschedule_and_overlaps() {
    let temp = TestDir::new("plans_status_overlaps");
    let (conn, s1_id, _) = setup_business_and_services(&temp, "UTC").await;
    let pool = &conn.require_pool().expect("pool");

    let client_a = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "Client A".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect("client A");

    let client_b = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "Client B".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect("client B");

    let plan_a = session_plan_service::create_session_plan(
        pool,
        CreateSessionPlanInput {
            client_id: client_a.id.clone(),
            title: "5-Session Package".into(),
            description: Some("Structured 5-part series".into()),
            planned_sessions: 5,
        },
    )
    .await
    .expect("create plan A");

    // Linking Client A's plan to Client B's appointment must fail
    let mismatch_err = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_b.id.clone()),
            inline_client: None,
            session_plan_id: Some(plan_a.id.clone()),
            start_local: "2026-12-01T10:00".into(),
            end_local: "2026-12-01T11:00".into(),
            dst_disambiguation: None,
            service_ids: vec![s1_id.clone()],
            notes: None,
        },
    )
    .await
    .expect_err("plan/client mismatch must fail");
    assert!(matches!(mismatch_err, AppError::Validation(_)));

    // Create appointment 1 linked to plan A
    let appt1 = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_a.id.clone()),
            inline_client: None,
            session_plan_id: Some(plan_a.id.clone()),
            start_local: "2026-12-01T10:00".into(),
            end_local: "2026-12-01T11:00".into(),
            dst_disambiguation: None,
            service_ids: vec![s1_id.clone()],
            notes: None,
        },
    )
    .await
    .expect("create appt1")
    .appointment;

    // Create overlapping appointment 2 for Client B -> succeeds with overlap warning
    let appt2_res = appointment_service::create_appointment(
        pool,
        CreateAppointmentInput {
            client_id: Some(client_b.id.clone()),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-12-01T10:30".into(),
            end_local: "2026-12-01T11:30".into(),
            dst_disambiguation: None,
            service_ids: vec![s1_id.clone()],
            notes: None,
        },
    )
    .await
    .expect("overlapping appointment should warn, not silently block");
    assert_eq!(appt2_res.overlap_warnings.len(), 1);
    assert_eq!(appt2_res.overlap_warnings[0].appointment_id, appt1.id);

    // Reschedule appt1 to a non-overlapping slot
    let rescheduled = appointment_service::reschedule_appointment(
        pool,
        RescheduleAppointmentInput {
            id: appt1.id.clone(),
            start_local: "2026-12-01T14:00".into(),
            end_local: "2026-12-01T15:00".into(),
            dst_disambiguation: None,
        },
    )
    .await
    .expect("reschedule appt1");
    assert!(rescheduled.overlap_warnings.is_empty());
    assert_eq!(rescheduled.appointment.start_local, "2026-12-01T14:00");

    // Complete appt1 -> cannot cancel or reschedule after completion
    let completed = appointment_service::complete_appointment(pool, &appt1.id)
        .await
        .expect("complete appt1");
    assert_eq!(completed.status, "completed");

    let cancel_after_complete = appointment_service::cancel_appointment(pool, &appt1.id)
        .await
        .expect_err("cannot cancel completed appointment");
    assert!(matches!(cancel_after_complete, AppError::Validation(_)));

    let reschedule_after_complete = appointment_service::reschedule_appointment(
        pool,
        RescheduleAppointmentInput {
            id: appt1.id.clone(),
            start_local: "2026-12-02T14:00".into(),
            end_local: "2026-12-02T15:00".into(),
            dst_disambiguation: None,
        },
    )
    .await
    .expect_err("cannot reschedule completed appointment");
    assert!(matches!(reschedule_after_complete, AppError::Validation(_)));

    // Cancel appt2 -> cannot complete after cancellation
    let cancelled = appointment_service::cancel_appointment(pool, &appt2_res.appointment.id)
        .await
        .expect("cancel appt2");
    assert_eq!(cancelled.status, "cancelled");

    let complete_after_cancel =
        appointment_service::complete_appointment(pool, &appt2_res.appointment.id)
            .await
            .expect_err("cannot complete cancelled appointment");
    assert!(matches!(complete_after_cancel, AppError::Validation(_)));

    // Verify plan A summary counts and listing appointments by plan
    let plan_summary = session_plan_service::get_session_plan(pool, &plan_a.id)
        .await
        .expect("plan summary");
    assert_eq!(plan_summary.linked_total, 1);
    assert_eq!(plan_summary.linked_completed, 1);
    assert_eq!(plan_summary.linked_scheduled, 0);

    let plan_appts = appointment_service::list_appointments(
        pool,
        ListAppointmentsParams {
            session_plan_id: Some(plan_a.id.clone()),
            ..Default::default()
        },
    )
    .await
    .expect("list plan appts");
    assert_eq!(plan_appts.len(), 1);

    conn.close().await;
}

#[tokio::test]
async fn create_session_plan_with_multiple_appointments_is_atomic() {
    let temp = TestDir::new("plan_multi_appts");
    let (conn, s1_id, s2_id) = setup_business_and_services(&temp, "Europe/Madrid").await;
    let pool = &conn.require_pool().expect("pool");

    let client = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "Lucia Caminos".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect("create Lucia Caminos");

    // 1. Mismatched session count must fail without creating an orphan plan
    let mismatch_err = session_plan_service::create_session_plan_with_appointments(
        pool,
        CreateSessionPlanWithAppointmentsInput {
            client_id: client.id.clone(),
            title: "8-Session Plan".into(),
            description: None,
            planned_sessions: 8,
            service_ids: vec![s1_id.clone()],
            sessions: vec![PlanSessionSlotInput {
                start_local: "2026-12-01T10:00".into(),
                end_local: "2026-12-01T11:00".into(),
                dst_disambiguation: None,
                notes: None,
            }],
        },
    )
    .await
    .expect_err("mismatched session slot count must fail");
    assert!(matches!(mismatch_err, AppError::Validation(_)));
    assert!(
        session_plan_service::list_session_plans(pool, Default::default())
            .await
            .expect("list plans")
            .is_empty()
    );

    // 2. Valid 8-session plan creates the plan and all 8 individual linked appointments
    let sessions: Vec<PlanSessionSlotInput> = (1..=8)
        .map(|day| PlanSessionSlotInput {
            start_local: format!("2026-12-{day:02}T10:00"),
            end_local: format!("2026-12-{day:02}T11:30"),
            dst_disambiguation: None,
            notes: None,
        })
        .collect();

    let result = session_plan_service::create_session_plan_with_appointments(
        pool,
        CreateSessionPlanWithAppointmentsInput {
            client_id: client.id.clone(),
            title: "Lucia 8-Session Series".into(),
            description: Some("Full 8-session coaching plan".into()),
            planned_sessions: 8,
            service_ids: vec![s1_id, s2_id],
            sessions,
        },
    )
    .await
    .expect("create 8-session plan with appointments");

    assert_eq!(result.plan.planned_sessions, 8);
    assert_eq!(result.plan.linked_total, 8);
    assert_eq!(result.plan.linked_scheduled, 8);
    assert_eq!(result.appointments.len(), 8);
    for (idx, appt) in result.appointments.iter().enumerate() {
        assert_eq!(appt.client_id, client.id);
        assert_eq!(
            appt.session_plan_id.as_deref(),
            Some(result.plan.id.as_str())
        );
        assert_eq!(appt.services.len(), 2);
        assert!(appt
            .notes
            .as_deref()
            .unwrap_or("")
            .contains(&format!("Session {} of 8", idx + 1)));
    }

    conn.close().await;
}

#[tokio::test]
async fn delete_appointment_updates_session_plan_and_delete_session_plan_works() {
    let temp = TestDir::new("delete_appts_and_plans");
    let (conn, s1_id, _) = setup_business_and_services(&temp, "Europe/Madrid").await;
    let pool = &conn.require_pool().expect("pool");

    let client = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "Lucia Caminos".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect("create client");

    // 1. Create a 3-session plan with 3 appointments
    let plan_result = session_plan_service::create_session_plan_with_appointments(
        pool,
        CreateSessionPlanWithAppointmentsInput {
            client_id: client.id.clone(),
            title: "3-Session Massage Pack".into(),
            description: None,
            planned_sessions: 3,
            service_ids: vec![s1_id.clone()],
            sessions: vec![
                PlanSessionSlotInput {
                    start_local: "2026-12-01T10:00".into(),
                    end_local: "2026-12-01T11:00".into(),
                    dst_disambiguation: None,
                    notes: None,
                },
                PlanSessionSlotInput {
                    start_local: "2026-12-08T10:00".into(),
                    end_local: "2026-12-08T11:00".into(),
                    dst_disambiguation: None,
                    notes: None,
                },
                PlanSessionSlotInput {
                    start_local: "2026-12-15T10:00".into(),
                    end_local: "2026-12-15T11:00".into(),
                    dst_disambiguation: None,
                    notes: None,
                },
            ],
        },
    )
    .await
    .expect("create 3-session plan");

    let plan_id = plan_result.plan.id.clone();
    let appt_0_id = plan_result.appointments[0].id.clone();
    let appt_1_id = plan_result.appointments[1].id.clone();
    let appt_2_id = plan_result.appointments[2].id.clone();

    // 2. Delete the first appointment from the session plan -> appointment gone, plan updated to 2 planned / 2 linked
    appointment_service::delete_appointment(pool, &appt_0_id)
        .await
        .expect("delete linked appointment");

    let missing_err = appointment_service::get_appointment(pool, &appt_0_id)
        .await
        .expect_err("deleted appointment must not exist");
    assert!(matches!(missing_err, AppError::NotFound(_)));

    let updated_plan = session_plan_service::get_session_plan(pool, &plan_id)
        .await
        .expect("reload session plan");
    assert_eq!(updated_plan.planned_sessions, 2);
    assert_eq!(updated_plan.linked_total, 2);
    assert_eq!(updated_plan.linked_scheduled, 2);

    // 3. Delete the session plan while KEEPING linked appointments as independent
    session_plan_service::delete_session_plan(pool, &plan_id, false)
        .await
        .expect("delete session plan keeping appointments");

    let missing_plan = session_plan_service::get_session_plan(pool, &plan_id)
        .await
        .expect_err("deleted session plan must not exist");
    assert!(matches!(missing_plan, AppError::NotFound(_)));

    let remaining_1 = appointment_service::get_appointment(pool, &appt_1_id)
        .await
        .expect("unlinked appt 1 remains");
    assert!(remaining_1.session_plan_id.is_none());
    assert!(remaining_1.session_plan_title.is_none());

    // Delete standalone appointment (even after marking completed)
    appointment_service::complete_appointment(pool, &appt_1_id)
        .await
        .expect("complete appt 1");
    appointment_service::delete_appointment(pool, &appt_1_id)
        .await
        .expect("delete completed appointment");
    appointment_service::delete_appointment(pool, &appt_2_id)
        .await
        .expect("delete scheduled appointment");

    // 4. Create a 1-session plan, delete its only appointment (planned_sessions stays >= 1), then delete plan
    let single_plan = session_plan_service::create_session_plan_with_appointments(
        pool,
        CreateSessionPlanWithAppointmentsInput {
            client_id: client.id.clone(),
            title: "Single Session Plan".into(),
            description: None,
            planned_sessions: 1,
            service_ids: vec![s1_id.clone()],
            sessions: vec![PlanSessionSlotInput {
                start_local: "2026-12-20T10:00".into(),
                end_local: "2026-12-20T11:00".into(),
                dst_disambiguation: None,
                notes: None,
            }],
        },
    )
    .await
    .expect("create 1-session plan");

    appointment_service::delete_appointment(pool, &single_plan.appointments[0].id)
        .await
        .expect("delete only appointment in 1-session plan");

    let reloaded_single = session_plan_service::get_session_plan(pool, &single_plan.plan.id)
        .await
        .expect("reload single plan");
    assert_eq!(reloaded_single.planned_sessions, 1);
    assert_eq!(reloaded_single.linked_total, 0);

    session_plan_service::delete_session_plan(pool, &single_plan.plan.id, false)
        .await
        .expect("delete empty single plan");

    // 5. Create a 2-session plan and delete BOTH plan and linked appointments
    let cascade_plan = session_plan_service::create_session_plan_with_appointments(
        pool,
        CreateSessionPlanWithAppointmentsInput {
            client_id: client.id,
            title: "Cascade Plan".into(),
            description: None,
            planned_sessions: 2,
            service_ids: vec![s1_id],
            sessions: vec![
                PlanSessionSlotInput {
                    start_local: "2026-12-22T10:00".into(),
                    end_local: "2026-12-22T11:00".into(),
                    dst_disambiguation: None,
                    notes: None,
                },
                PlanSessionSlotInput {
                    start_local: "2026-12-29T10:00".into(),
                    end_local: "2026-12-29T11:00".into(),
                    dst_disambiguation: None,
                    notes: None,
                },
            ],
        },
    )
    .await
    .expect("create 2-session cascade plan");

    session_plan_service::delete_session_plan(pool, &cascade_plan.plan.id, true)
        .await
        .expect("delete plan with linked appointments");

    let remaining_appts =
        appointment_service::list_appointments(pool, ListAppointmentsParams::default())
            .await
            .expect("list remaining appointments");
    assert!(remaining_appts.is_empty());

    // 6. Deleting non-existent IDs returns NotFound
    assert!(matches!(
        appointment_service::delete_appointment(pool, "missing-id").await,
        Err(AppError::NotFound(_))
    ));
    assert!(matches!(
        session_plan_service::delete_session_plan(pool, "missing-id", false).await,
        Err(AppError::NotFound(_))
    ));

    conn.close().await;
}
