use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signer, SigningKey};
use my_schedule_app_lib::backup::{self, BackupType};
use my_schedule_app_lib::database;
use my_schedule_app_lib::errors::AppError;
use my_schedule_app_lib::export::{
    self, CsvEntity, VersionedBusinessExport, EXPORT_SCHEMA_VERSION,
};
use my_schedule_app_lib::models::appointment::{
    CreateAppointmentInput, RescheduleAppointmentInput,
};
use my_schedule_app_lib::models::business::{
    CreateBusinessProfileInput, UpdateBusinessProfileInput,
};
use my_schedule_app_lib::models::calendar::{CalendarRangeInput, CalendarView};
use my_schedule_app_lib::models::client::{CreateClientInput, UpdateClientInput};
use my_schedule_app_lib::models::reports::{
    DashboardPeriod, OperationalReportInput, ReportPeriodType,
};
use my_schedule_app_lib::models::service::{CreateServiceInput, UpdateServiceInput};
use my_schedule_app_lib::models::session_plan::CreateSessionPlanInput;
use my_schedule_app_lib::models::settings::UpdateAppSettingsInput;
use my_schedule_app_lib::notifications::NoopNotificationSender;
use my_schedule_app_lib::platform;
use my_schedule_app_lib::scheduler;
use my_schedule_app_lib::services::{
    appointment_service, business_service, calendar_service, client_service, reminder_service,
    report_service, service_catalog_service, session_plan_service, settings_service,
};
use my_schedule_app_lib::updater::{self, PlatformArtifact, UpdateManifest, GITHUB_REPO};

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
            "my_schedule_app_e2e_{label}_{}_{}_{}",
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

#[tokio::test]
async fn release_candidate_full_operator_lifecycle_end_to_end() {
    let temp = TestDir::new("rc_lifecycle");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create app directories");

    // =========================================================================
    // 1. Fresh launch & onboarding
    // =========================================================================
    let (conn, _migration_status) = database::initialize(&paths.database_path)
        .await
        .expect("init database");
    let pool = conn.require_pool().expect("db pool");

    let before_onboarding = business_service::get_onboarding_status(&pool)
        .await
        .expect("onboarding status before setup");
    assert!(!before_onboarding.completed);
    assert!(before_onboarding.business.is_none());

    let created_business = business_service::create_business_profile(
        &pool,
        &paths.logos_dir,
        CreateBusinessProfileInput {
            name: "Atlas Studio".into(),
            address: Some("100 Harbor Way".into()),
            city: Some("Madrid".into()),
            region: None,
            postal_code: Some("28001".into()),
            country: Some("Spain".into()),
            phone: Some("+34 600 100 200".into()),
            email: Some("desk@atlas.example".into()),
            website: Some("https://atlas.example".into()),
            timezone: "Europe/Madrid".into(),
            logo_bytes: None,
        },
    )
    .await
    .expect("create business profile");
    assert_eq!(created_business.name, "Atlas Studio");
    assert_eq!(created_business.timezone, "Europe/Madrid");

    // 2. Business profile edit
    let edited_business = business_service::update_business_profile(
        &pool,
        UpdateBusinessProfileInput {
            name: "Atlas Operations Studio".into(),
            address: Some("100 Harbor Way".into()),
            city: Some("Madrid".into()),
            region: None,
            postal_code: Some("28001".into()),
            country: Some("Spain".into()),
            phone: Some("+34 600 100 200".into()),
            email: Some("desk@atlas.example".into()),
            website: Some("https://atlas.example".into()),
            timezone: "Europe/Madrid".into(),
        },
    )
    .await
    .expect("update business profile");
    assert_eq!(edited_business.name, "Atlas Operations Studio");

    // =========================================================================
    // 3. Client create, edit, archive, restore
    // =========================================================================
    let client_a = client_service::create_client(
        &pool,
        CreateClientInput {
            full_name: "Elena Navarro".into(),
            phone: Some("+34 611 222 333".into()),
            email: Some("elena@example.com".into()),
            address: None,
            notes: Some("Prefers morning appointments".into()),
        },
    )
    .await
    .expect("create client A");

    let client_a = client_service::update_client(
        &pool,
        UpdateClientInput {
            id: client_a.id.clone(),
            full_name: "Elena Navarro Ruiz".into(),
            phone: client_a.phone.clone(),
            email: client_a.email.clone(),
            address: Some("Calle Mayor 12".into()),
            notes: client_a.notes.clone(),
        },
    )
    .await
    .expect("update client A");
    assert_eq!(client_a.full_name, "Elena Navarro Ruiz");

    let archived = client_service::archive_client(&pool, &client_a.id)
        .await
        .expect("archive client A");
    assert!(archived.is_archived);

    let restored = client_service::restore_client(&pool, &client_a.id)
        .await
        .expect("restore client A");
    assert!(!restored.is_archived);

    // =========================================================================
    // 4. Service create, edit, deactivate & snapshot preservation
    // =========================================================================
    let svc_primary = service_catalog_service::create_service(
        &pool,
        CreateServiceInput {
            name: "Initial Assessment".into(),
            description: Some("60-minute structured onboarding".into()),
            default_duration_minutes: 60,
        },
    )
    .await
    .expect("create primary service");

    let svc_followup = service_catalog_service::create_service(
        &pool,
        CreateServiceInput {
            name: "Follow-up Review".into(),
            description: Some("30-minute check-in".into()),
            default_duration_minutes: 30,
        },
    )
    .await
    .expect("create follow-up service");

    // =========================================================================
    // 5. Session plan + independent & plan-linked + inline client + multi-service
    // =========================================================================
    let plan = session_plan_service::create_session_plan(
        &pool,
        CreateSessionPlanInput {
            client_id: client_a.id.clone(),
            title: "Q4 Program".into(),
            description: Some("4-session sequence".into()),
            planned_sessions: 4,
        },
    )
    .await
    .expect("create session plan");

    // Pick a deterministic near-future business-local date so reminders & reports work predictably
    let tz: chrono_tz::Tz = "Europe/Madrid".parse().expect("parse tz");
    let now_utc = Utc::now();
    let target_local_date = (now_utc + Duration::days(2))
        .with_timezone(&tz)
        .date_naive();
    let date_str = target_local_date.format("%Y-%m-%d").to_string();

    // Plan-linked multi-service appointment
    let appt_plan = appointment_service::create_appointment(
        &pool,
        CreateAppointmentInput {
            client_id: Some(client_a.id.clone()),
            inline_client: None,
            session_plan_id: Some(plan.id.clone()),
            start_local: format!("{date_str}T10:00"),
            end_local: format!("{date_str}T11:30"),
            dst_disambiguation: None,
            service_ids: vec![svc_primary.id.clone(), svc_followup.id.clone()],
            notes: Some("Session 1 of Q4 Program".into()),
        },
    )
    .await
    .expect("create plan-linked appointment")
    .appointment;
    assert_eq!(appt_plan.services.len(), 2);

    // Independent appointment with inline client creation
    let appt_inline = appointment_service::create_appointment(
        &pool,
        CreateAppointmentInput {
            client_id: None,
            inline_client: Some(CreateClientInput {
                full_name: "Marco Silva".into(),
                phone: Some("+34 622 333 444".into()),
                email: None,
                address: None,
                notes: None,
            }),
            session_plan_id: None,
            start_local: format!("{date_str}T14:00"),
            end_local: format!("{date_str}T15:00"),
            dst_disambiguation: None,
            service_ids: vec![svc_primary.id.clone()],
            notes: None,
        },
    )
    .await
    .expect("create inline-client appointment")
    .appointment;
    assert_eq!(appt_inline.client_name, "Marco Silva");

    // Edit & deactivate svc_primary -> historical snapshot on appt_plan must remain "Initial Assessment"
    service_catalog_service::update_service(
        &pool,
        UpdateServiceInput {
            id: svc_primary.id.clone(),
            name: "Renamed Assessment".into(),
            description: None,
            default_duration_minutes: 90,
        },
    )
    .await
    .expect("rename primary service");
    service_catalog_service::deactivate_service(&pool, &svc_primary.id)
        .await
        .expect("deactivate primary service");

    let reloaded_appt_plan = appointment_service::get_appointment(&pool, &appt_plan.id)
        .await
        .expect("reload appt_plan");
    assert_eq!(
        reloaded_appt_plan.services[0].service_name_snapshot,
        "Initial Assessment"
    );
    assert_eq!(reloaded_appt_plan.services[0].duration_minutes_snapshot, 60);

    // =========================================================================
    // 6. Reschedule, complete, cancel, and terminal-transition enforcement
    // =========================================================================
    let rescheduled = appointment_service::reschedule_appointment(
        &pool,
        RescheduleAppointmentInput {
            id: appt_plan.id.clone(),
            start_local: format!("{date_str}T11:00"),
            end_local: format!("{date_str}T12:30"),
            dst_disambiguation: None,
        },
    )
    .await
    .expect("reschedule appt_plan");
    assert_eq!(
        rescheduled.appointment.start_local,
        format!("{date_str}T11:00")
    );
    let original_start_utc = rescheduled.appointment.start_utc.clone();

    let completed = appointment_service::complete_appointment(&pool, &appt_plan.id)
        .await
        .expect("complete appt_plan");
    assert_eq!(completed.status, "completed");
    assert!(matches!(
        appointment_service::cancel_appointment(&pool, &appt_plan.id).await,
        Err(AppError::Validation(_))
    ));

    let cancelled = appointment_service::cancel_appointment(&pool, &appt_inline.id)
        .await
        .expect("cancel appt_inline");
    assert_eq!(cancelled.status, "cancelled");
    assert!(matches!(
        appointment_service::complete_appointment(&pool, &appt_inline.id).await,
        Err(AppError::Validation(_))
    ));

    // Create one more active scheduled appointment for reminders, calendar & reports
    let active_appt = appointment_service::create_appointment(
        &pool,
        CreateAppointmentInput {
            client_id: Some(client_a.id.clone()),
            inline_client: None,
            session_plan_id: Some(plan.id.clone()),
            start_local: format!("{date_str}T16:00"),
            end_local: format!("{date_str}T16:30"),
            dst_disambiguation: None,
            service_ids: vec![svc_followup.id.clone()],
            notes: Some("Upcoming follow-up".into()),
        },
    )
    .await
    .expect("create active scheduled appointment")
    .appointment;

    // =========================================================================
    // 7. Calendar Day, Week, Month, Agenda views
    // =========================================================================
    for view in [
        CalendarView::Day,
        CalendarView::Week,
        CalendarView::Month,
        CalendarView::Agenda,
    ] {
        let cal = calendar_service::get_calendar_range(
            &pool,
            CalendarRangeInput {
                view,
                anchor_date: Some(date_str.clone()),
                status: None,
            },
        )
        .await
        .expect("calendar query");
        assert!(
            cal.appointments.iter().any(|a| a.id == active_appt.id),
            "active appointment must appear in calendar view"
        );
    }

    // =========================================================================
    // 8. Reminder engine: startup catch-up + timed tick + duplicate prevention
    // =========================================================================
    let appt_start_dt = DateTime::parse_from_rfc3339(&active_appt.start_utc)
        .expect("parse start_utc")
        .with_timezone(&Utc);
    let sender = NoopNotificationSender;

    // Simulate launch on the appointment's business-local morning (triggers day_of_launch once)
    let launch_morning_utc = appt_start_dt - Duration::hours(6);
    let first_catchup = scheduler::process_startup_catchup(&pool, launch_morning_utc, &sender)
        .await
        .expect("first startup catchup");
    assert!(first_catchup.day_of_sent_count >= 1);

    // Repeating startup catchup on the same business-local day must NOT duplicate day_of_launch
    let second_catchup = scheduler::process_startup_catchup(
        &pool,
        launch_morning_utc + Duration::minutes(10),
        &sender,
    )
    .await
    .expect("second startup catchup");
    assert_eq!(second_catchup.day_of_sent_count, 0);

    // Simulate tick at 55 minutes before appointment (triggers one_hour_before once)
    let tick_time = appt_start_dt - Duration::minutes(55);
    let tick_1 = scheduler::process_scheduler_tick(&pool, tick_time, &sender)
        .await
        .expect("first tick");
    assert!(tick_1.timed_sent_count >= 1);

    let tick_2 =
        scheduler::process_scheduler_tick(&pool, tick_time + Duration::minutes(1), &sender)
            .await
            .expect("second tick");
    assert_eq!(tick_2.timed_sent_count, 0);

    let inbox = reminder_service::list_notifications(&pool, false)
        .await
        .expect("list notifications");
    assert!(!inbox.is_empty());

    // =========================================================================
    // 9. Dashboard summary & Weekly/Monthly operational reports (non-financial)
    // =========================================================================
    let dashboard = report_service::get_dashboard_summary(&pool, DashboardPeriod::ThisMonth)
        .await
        .expect("dashboard summary");
    assert!(dashboard.active_clients_count >= 2);
    assert!(dashboard.active_session_plans_count >= 1);

    let weekly_report = report_service::get_operational_report(
        &pool,
        OperationalReportInput {
            period_type: ReportPeriodType::Weekly,
            anchor_date: Some(date_str.clone()),
        },
    )
    .await
    .expect("weekly report");
    assert_eq!(weekly_report.counts.total, 3);
    assert_eq!(weekly_report.counts.completed, 1);
    assert_eq!(weekly_report.counts.scheduled, 1);
    assert_eq!(weekly_report.counts.cancelled, 1);
    // Cancelled appointments must be excluded from active minutes
    assert_eq!(weekly_report.total_active_minutes, 120); // 90m completed + 30m scheduled

    let monthly_report = report_service::get_operational_report(
        &pool,
        OperationalReportInput {
            period_type: ReportPeriodType::Monthly,
            anchor_date: Some(date_str.clone()),
        },
    )
    .await
    .expect("monthly report");
    assert_eq!(monthly_report.counts.total, 3);

    // =========================================================================
    // 10. Versioned JSON export + all 4 CSV exports
    // =========================================================================
    let json_export = export::export_json(&pool, &paths.exports_dir, None)
        .await
        .expect("export JSON");
    let json_raw = fs::read_to_string(&json_export.file_path).expect("read JSON export");
    let parsed: VersionedBusinessExport =
        serde_json::from_str(&json_raw).expect("parse JSON export");
    assert_eq!(parsed.export_schema_version, EXPORT_SCHEMA_VERSION);
    assert_eq!(parsed.appointments.len(), 3);
    assert_eq!(parsed.clients.len(), 2);

    for entity in [
        CsvEntity::Clients,
        CsvEntity::Services,
        CsvEntity::Appointments,
        CsvEntity::AppointmentServices,
    ] {
        let csv_res = export::export_csv(&pool, &paths.exports_dir, entity, None)
            .await
            .expect("export CSV");
        assert!(Path::new(&csv_res.file_path).is_file());
        assert!(csv_res.record_count >= 2);
    }

    // =========================================================================
    // 11. Manual & automatic backups, validation, and restore with safety backup
    // =========================================================================
    let manual_bkp = backup::create_backup(
        &pool,
        &paths,
        BackupType::Manual,
        Some("RC manual backup".into()),
        None,
    )
    .await
    .expect("create manual backup");
    let manual_path =
        backup::resolve_managed_backup_path(&paths, BackupType::Manual, &manual_bkp.filename)
            .expect("resolve manual backup path");
    let validation = backup::validate_backup_file(&manual_path)
        .await
        .expect("validate manual backup");
    assert!(validation.valid);
    assert!(validation.integrity_ok);
    assert!(validation.schema_compatible);

    // Mutate DB after backup (add a temporary client), then restore manual backup
    let temp_client = client_service::create_client(
        &pool,
        CreateClientInput {
            full_name: "Temporary Post-Backup Client".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect("create post-backup client");
    assert!(!temp_client.id.is_empty());

    let restore_outcome =
        backup::restore_backup(&conn, &paths, BackupType::Manual, &manual_bkp.filename)
            .await
            .expect("restore manual backup");
    assert_eq!(
        restore_outcome.safety_backup.backup_type,
        BackupType::Safety
    );
    let safety_path = backup::resolve_managed_backup_path(
        &paths,
        BackupType::Safety,
        &restore_outcome.safety_backup.filename,
    )
    .expect("resolve safety backup path");
    assert!(safety_path.is_file());

    // After restore, the post-backup client is gone while pre-backup data is intact
    let pool_after_restore = conn.require_pool().expect("reopened pool");
    let clients_after_restore =
        client_service::list_clients(&pool_after_restore, Default::default())
            .await
            .expect("list clients after restore");
    assert_eq!(clients_after_restore.total, 2);

    // =========================================================================
    // 12. Settings persistence + timezone change preserving UTC instants across restart
    // =========================================================================
    settings_service::update_settings(
        &pool_after_restore,
        Some(&paths.automatic_backups_dir),
        UpdateAppSettingsInput {
            theme_mode: Some("dark".into()),
            accent_color: Some("emerald".into()),
            auto_backup_retention: Some(14),
            ..Default::default()
        },
    )
    .await
    .expect("update settings");

    business_service::update_business_profile(
        &pool_after_restore,
        UpdateBusinessProfileInput {
            name: "Atlas Operations Studio".into(),
            address: Some("100 Harbor Way".into()),
            city: Some("Madrid".into()),
            region: None,
            postal_code: Some("28001".into()),
            country: Some("Spain".into()),
            phone: Some("+34 600 100 200".into()),
            email: Some("desk@atlas.example".into()),
            website: Some("https://atlas.example".into()),
            timezone: "America/New_York".into(),
        },
    )
    .await
    .expect("change business timezone");

    // Simulate clean app restart
    conn.close().await;
    let (conn_restarted, _) = database::initialize(&paths.database_path)
        .await
        .expect("reopen database after restart");
    let pool_restarted = conn_restarted.require_pool().expect("restarted pool");

    let persisted_settings = settings_service::get_settings(&pool_restarted)
        .await
        .expect("read settings after restart");
    assert_eq!(persisted_settings.theme_mode, "dark");
    assert_eq!(persisted_settings.accent_color, "emerald");
    assert_eq!(persisted_settings.auto_backup_retention, 14);

    let appt_after_tz = appointment_service::get_appointment(&pool_restarted, &appt_plan.id)
        .await
        .expect("read appt after tz change");
    assert_eq!(
        appt_after_tz.start_utc, original_start_utc,
        "UTC instant must remain invariant across timezone changes"
    );

    // =========================================================================
    // 13. Multi-platform signed updater verification (Windows + Linux) & rollback
    // =========================================================================
    let signing_key = SigningKey::from_bytes(&[77u8; 32]);
    let pubkey_b64 = BASE64.encode(signing_key.verifying_key().to_bytes());
    let win_bytes = b"RC-WINDOWS-EXE";
    let linux_bytes = b"RC-LINUX-APPIMAGE";
    let mut platforms = BTreeMap::new();
    platforms.insert(
        "windows-x86_64".into(),
        PlatformArtifact {
            artifact_filename: "MyScheduleApp-v0.2.0-windows-x64.exe".into(),
            download_url: format!(
                "https://github.com/{GITHUB_REPO}/releases/download/v0.2.0/MyScheduleApp-v0.2.0-windows-x64.exe"
            ),
            sha256: updater::compute_sha256_hex(win_bytes),
        },
    );
    platforms.insert(
        "linux-x86_64".into(),
        PlatformArtifact {
            artifact_filename: "MyScheduleApp-v0.2.0-linux-x86_64.AppImage".into(),
            download_url: format!(
                "https://github.com/{GITHUB_REPO}/releases/download/v0.2.0/MyScheduleApp-v0.2.0-linux-x86_64.AppImage"
            ),
            sha256: updater::compute_sha256_hex(linux_bytes),
        },
    );
    let manifest = UpdateManifest {
        version: "0.2.0".into(),
        pub_date: "2026-12-01T12:00:00Z".into(),
        notes: "Release candidate verification".into(),
        target_schema_version: Some(7),
        platforms,
    };
    let manifest_bytes = serde_json::to_vec(&manifest).expect("serialize manifest");
    let sig_b64 = BASE64.encode(signing_key.sign(&manifest_bytes).to_bytes());

    let verified = updater::parse_and_verify_manifest(&manifest_bytes, &sig_b64, &pubkey_b64)
        .expect("verify multi-platform signed manifest");
    let staged_win = updater::stage_verified_update_bytes(
        &pool_restarted,
        &paths,
        &verified,
        "windows-x86_64",
        win_bytes,
    )
    .await
    .expect("stage Windows update");
    assert!(staged_win.apply_script_path.ends_with("apply-update.ps1"));

    let staged_linux = updater::stage_verified_update_bytes(
        &pool_restarted,
        &paths,
        &verified,
        "linux-x86_64",
        linux_bytes,
    )
    .await
    .expect("stage Linux update");
    assert!(staged_linux.apply_script_path.ends_with("apply-update.sh"));

    // Verify executable rollback preserves data/database.sqlite untouched
    let fake_exe = temp.path().join("MyScheduleApp.exe");
    fs::write(&fake_exe, b"V0.1.0").expect("write v0.1.0 exe");
    let rollback_err = updater::replace_executable_with_rollback(&fake_exe, b"BAD_V0.2.0", |_| {
        Err(AppError::Internal("Simulated crash".into()))
    });
    assert!(rollback_err.is_err());
    assert_eq!(fs::read(&fake_exe).unwrap(), b"V0.1.0");
    assert!(paths.database_path.is_file());

    conn_restarted.close().await;
}
