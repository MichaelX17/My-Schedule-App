use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Duration, Utc};
use my_schedule_app_lib::backup::{self, BackupType};
use my_schedule_app_lib::database;
use my_schedule_app_lib::errors::AppError;
use my_schedule_app_lib::models::appointment::CreateAppointmentInput;
use my_schedule_app_lib::models::business::{
    CreateBusinessProfileInput, UpdateBusinessProfileInput,
};
use my_schedule_app_lib::models::client::CreateClientInput;
use my_schedule_app_lib::models::service::CreateServiceInput;
use my_schedule_app_lib::models::settings::UpdateAppSettingsInput;
use my_schedule_app_lib::platform;
use my_schedule_app_lib::services::{
    appointment_service, business_service, client_service, reminder_service,
    service_catalog_service, settings_service,
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
            "my_schedule_app_set_{label}_{}_{}_{}",
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
async fn settings_persistence_validation_and_retention_pruning() {
    let temp = TestDir::new("persist");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    {
        let (conn, _) = database::initialize(&paths.database_path)
            .await
            .expect("init db");
        let pool = conn.require_pool().expect("pool");

        // 1. Defaults match confirmed product decisions
        let defaults = settings_service::get_settings(&pool)
            .await
            .expect("get defaults");
        assert_eq!(defaults.theme_mode, "system");
        assert_eq!(defaults.accent_color, "slate");
        assert_eq!(defaults.auto_backup_retention, 30);
        assert!(defaults.reminders_enabled);
        assert!(defaults.auto_backup_enabled);

        // 2. Reject invalid inputs
        assert!(matches!(
            settings_service::update_settings(
                &pool,
                None,
                UpdateAppSettingsInput {
                    theme_mode: Some("neon".into()),
                    ..Default::default()
                }
            )
            .await,
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            settings_service::update_settings(
                &pool,
                None,
                UpdateAppSettingsInput {
                    accent_color: Some("rainbow".into()),
                    ..Default::default()
                }
            )
            .await,
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            settings_service::update_settings(
                &pool,
                None,
                UpdateAppSettingsInput {
                    auto_backup_retention: Some(0),
                    ..Default::default()
                }
            )
            .await,
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            settings_service::update_settings(
                &pool,
                None,
                UpdateAppSettingsInput {
                    default_appointment_duration_minutes: Some(2000),
                    ..Default::default()
                }
            )
            .await,
            Err(AppError::Validation(_))
        ));

        // 3. Create 4 automatic backups, then lower retention to 2 via update_settings -> immediately prunes!
        for _ in 0..4 {
            backup::create_backup(&pool, &paths, BackupType::Automatic, None, Some(30))
                .await
                .expect("create auto backup");
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        assert_eq!(backup::list_backups(&paths).unwrap().len(), 4);

        let updated = settings_service::update_settings(
            &pool,
            Some(&paths.automatic_backups_dir),
            UpdateAppSettingsInput {
                theme_mode: Some("dark".into()),
                accent_color: Some("emerald".into()),
                os_notifications_enabled: Some(false),
                auto_backup_retention: Some(2),
                ..Default::default()
            },
        )
        .await
        .expect("update settings");

        assert_eq!(updated.theme_mode, "dark");
        assert_eq!(updated.accent_color, "emerald");
        assert!(!updated.os_notifications_enabled);
        assert_eq!(updated.auto_backup_retention, 2);
        assert_eq!(backup::list_backups(&paths).unwrap().len(), 2);

        conn.close().await;
    }

    // 4. Reopen DB and confirm settings survived restart
    let (conn2, _) = database::initialize(&paths.database_path)
        .await
        .expect("reopen db");
    let pool2 = conn2.require_pool().expect("pool2");
    let reloaded = settings_service::get_settings(&pool2)
        .await
        .expect("reloaded settings");
    assert_eq!(reloaded.theme_mode, "dark");
    assert_eq!(reloaded.accent_color, "emerald");
    assert!(!reloaded.os_notifications_enabled);
    assert_eq!(reloaded.auto_backup_retention, 2);
}

#[tokio::test]
async fn timezone_change_preserves_utc_instants_and_recalculates_pending_reminders() {
    let temp = TestDir::new("tz_change");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");
    let (conn, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool = conn.require_pool().expect("pool");

    business_service::create_business_profile(
        &pool,
        &paths.logos_dir,
        CreateBusinessProfileInput {
            name: "TZ Studio".into(),
            address: None,
            city: None,
            region: None,
            postal_code: None,
            country: None,
            phone: None,
            email: None,
            website: None,
            timezone: "Europe/Madrid".into(),
            logo_bytes: None,
        },
    )
    .await
    .expect("create business");

    let client = client_service::create_client(
        &pool,
        CreateClientInput {
            full_name: "Claire Redfield".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect("create client");

    let service = service_catalog_service::create_service(
        &pool,
        CreateServiceInput {
            name: "Consultation".into(),
            description: None,
            default_duration_minutes: 60,
        },
    )
    .await
    .expect("create service");

    let future_utc = Utc::now() + Duration::days(10);
    let start_local_madrid =
        time::format_utc_in_timezone(&time::format_utc_iso(future_utc), "Europe/Madrid").unwrap();
    let end_local_madrid = time::format_utc_in_timezone(
        &time::format_utc_iso(future_utc + Duration::hours(1)),
        "Europe/Madrid",
    )
    .unwrap();

    let appt = appointment_service::create_appointment(
        &pool,
        CreateAppointmentInput {
            client_id: Some(client.id),
            inline_client: None,
            session_plan_id: None,
            start_local: start_local_madrid,
            end_local: end_local_madrid,
            dst_disambiguation: None,
            service_ids: vec![service.id],
            notes: None,
        },
    )
    .await
    .expect("create appt")
    .appointment;

    let original_start_utc = appt.start_utc.clone();
    let original_end_utc = appt.end_utc.clone();

    // Switch business timezone to America/New_York
    business_service::update_business_profile(
        &pool,
        UpdateBusinessProfileInput {
            name: "TZ Studio".into(),
            address: None,
            city: None,
            region: None,
            postal_code: None,
            country: None,
            phone: None,
            email: None,
            website: None,
            timezone: "America/New_York".into(),
        },
    )
    .await
    .expect("update tz");

    // Stored UTC instants MUST NOT be rewritten, while display timezone updates to America/New_York
    let reloaded = appointment_service::get_appointment(&pool, &appt.id)
        .await
        .expect("get appt");
    assert_eq!(reloaded.start_utc, original_start_utc);
    assert_eq!(reloaded.end_utc, original_end_utc);
    assert_eq!(reloaded.timezone, "America/New_York");

    // Pending reminders remain valid and display in America/New_York
    let upcoming = reminder_service::list_upcoming_reminders(&pool)
        .await
        .expect("upcoming reminders");
    assert_eq!(upcoming.len(), 2);
    assert!(upcoming.iter().all(|r| r.timezone == "America/New_York"));
}
