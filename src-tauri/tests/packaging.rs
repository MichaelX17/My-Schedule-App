use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use my_schedule_app_lib::backup::{self, BackupType};
use my_schedule_app_lib::database;
use my_schedule_app_lib::export::{self, CsvEntity};
use my_schedule_app_lib::models::appointment::CreateAppointmentInput;
use my_schedule_app_lib::models::business::CreateBusinessProfileInput;
use my_schedule_app_lib::models::client::{CreateClientInput, ListClientsParams};
use my_schedule_app_lib::models::service::CreateServiceInput;
use my_schedule_app_lib::models::settings::UpdateAppSettingsInput;
use my_schedule_app_lib::platform::{self, StorageMode};
use my_schedule_app_lib::services::{
    appointment_service, business_service, client_service, service_catalog_service,
    settings_service,
};

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
            "my_schedule_app_pkg_{label}_{}_{}_{}",
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

#[test]
fn temp_extraction_directory_refuses_portable_and_falls_back_safely() {
    let temp = TestDir::new("temp_guard");
    let fake_temp_root = temp.path().join("TempRoot");
    let extracted_zip_dir = fake_temp_root.join("Temp1_MyScheduleApp_zip");
    let persistent_appdata = temp.path().join("AppDataRoaming");

    fs::create_dir_all(&extracted_zip_dir).expect("create fake zip dir");
    fs::create_dir_all(&persistent_appdata).expect("create persistent fallback dir");

    let resolved = platform::resolve_storage_paths_with_temp_guard(
        Some(&extracted_zip_dir),
        Some(&persistent_appdata),
        Some(&fake_temp_root),
    )
    .expect("should fall back to persistent appdata");

    assert_eq!(resolved.mode, StorageMode::Fallback);
    assert_eq!(resolved.paths.data_dir, persistent_appdata.join("data"));
    assert!(resolved
        .fallback_reason
        .as_deref()
        .unwrap_or_default()
        .contains("temporary"));
    // Confirm no data/ folder was created inside the volatile temp extraction folder
    assert!(!extracted_zip_dir.join("data").exists());
}

#[tokio::test]
async fn non_ascii_portable_path_supports_full_lifecycle_backup_restore_and_exports() {
    let temp = TestDir::new("non_ascii");
    let unicode_portable_dir = temp
        .path()
        .join("Mi Agenda — Clínica Café 日本語")
        .join("MyScheduleApp");
    let fallback_dir = temp.path().join("fallback");
    fs::create_dir_all(&unicode_portable_dir).expect("create unicode dir");

    let resolved =
        platform::resolve_storage_paths(Some(&unicode_portable_dir), Some(&fallback_dir))
            .expect("resolve unicode portable dir");
    assert_eq!(resolved.mode, StorageMode::Portable);
    resolved.paths.ensure_directories().expect("ensure dirs");

    // Verify all required directories exist under the non-ASCII path
    assert!(resolved.paths.backups_dir.is_dir());
    assert!(resolved.paths.automatic_backups_dir.is_dir());
    assert!(resolved.paths.manual_backups_dir.is_dir());
    assert!(resolved.paths.exports_dir.is_dir());
    assert!(resolved.paths.logos_dir.is_dir());
    assert!(resolved.paths.logs_dir.is_dir());
    assert!(resolved.paths.updates_dir.is_dir());

    let (conn, health) = database::initialize(&resolved.paths.database_path)
        .await
        .expect("init db in unicode path");
    assert!(health.ok);
    let pool = conn.require_pool().expect("pool");

    business_service::create_business_profile(
        &pool,
        &resolved.paths.logos_dir,
        CreateBusinessProfileInput {
            name: "Clínica Café".into(),
            address: None,
            city: Some("São Paulo".into()),
            region: None,
            postal_code: None,
            country: None,
            phone: None,
            email: None,
            website: None,
            timezone: "America/Sao_Paulo".into(),
            logo_bytes: None,
        },
    )
    .await
    .expect("create business");

    let client = client_service::create_client(
        &pool,
        CreateClientInput {
            full_name: "José Álvarez".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect("create client");

    let svc = service_catalog_service::create_service(
        &pool,
        CreateServiceInput {
            name: "Consulta Inicial".into(),
            description: None,
            default_duration_minutes: 45,
        },
    )
    .await
    .expect("create service");

    appointment_service::create_appointment(
        &pool,
        CreateAppointmentInput {
            client_id: Some(client.id),
            inline_client: None,
            session_plan_id: None,
            start_local: "2026-08-12T10:00".into(),
            end_local: "2026-08-12T10:45".into(),
            dst_disambiguation: None,
            service_ids: vec![svc.id],
            notes: None,
        },
    )
    .await
    .expect("create appointment");

    // Verify JSON/CSV export and SQLite VACUUM INTO backup + restore inside non-ASCII path
    let json_export = export::export_json(&pool, &resolved.paths.exports_dir, None)
        .await
        .expect("json export");
    assert!(Path::new(&json_export.file_path).is_file());

    let csv_export =
        export::export_csv(&pool, &resolved.paths.exports_dir, CsvEntity::Clients, None)
            .await
            .expect("csv export");
    assert!(Path::new(&csv_export.file_path).is_file());

    let manual_bkp = backup::create_backup(
        &pool,
        &resolved.paths,
        BackupType::Manual,
        Some("Unicode path backup".into()),
        None,
    )
    .await
    .expect("manual backup");
    drop(pool);

    let restored = backup::restore_backup(
        &conn,
        &resolved.paths,
        BackupType::Manual,
        &manual_bkp.filename,
    )
    .await
    .expect("restore in unicode path");
    assert_eq!(restored.restored_filename, manual_bkp.filename);
}

#[tokio::test]
async fn upgrade_over_existing_portable_data_preserves_all_records_and_backups() {
    let temp = TestDir::new("upgrade");
    let portable_dir = temp.path().join("MyScheduleApp");
    fs::create_dir_all(&portable_dir).expect("create portable dir");

    // Simulate Version N first launch & user activity
    let resolved_v1 = platform::resolve_storage_paths(Some(&portable_dir), None).expect("v1");
    resolved_v1.paths.ensure_directories().expect("dirs");
    {
        let (conn, _) = database::initialize(&resolved_v1.paths.database_path)
            .await
            .expect("init v1");
        let pool = conn.require_pool().expect("pool");

        business_service::create_business_profile(
            &pool,
            &resolved_v1.paths.logos_dir,
            CreateBusinessProfileInput {
                name: "Persistent Studio".into(),
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
        .expect("business");

        client_service::create_client(
            &pool,
            CreateClientInput {
                full_name: "Long-Term Client".into(),
                phone: None,
                email: None,
                address: None,
                notes: None,
            },
        )
        .await
        .expect("client");

        settings_service::update_settings(
            &pool,
            None,
            UpdateAppSettingsInput {
                theme_mode: Some("dark".into()),
                accent_color: Some("blue".into()),
                auto_backup_retention: Some(15),
                ..Default::default()
            },
        )
        .await
        .expect("settings");

        backup::create_backup(&pool, &resolved_v1.paths, BackupType::Manual, None, None)
            .await
            .expect("v1 backup");

        conn.close().await;
    }

    // Simulate replacing the executable with Version N+1 (touching only sibling files, never data/)
    fs::write(portable_dir.join("PORTABLE_README.txt"), b"Upgraded README").expect("write readme");

    // Relaunch over existing data/ directory
    let resolved_v2 = platform::resolve_storage_paths(Some(&portable_dir), None).expect("v2");
    assert_eq!(resolved_v2.mode, StorageMode::Portable);
    resolved_v2.paths.ensure_directories().expect("dirs v2");

    let (conn2, health2) = database::initialize(&resolved_v2.paths.database_path)
        .await
        .expect("reopen v2");
    assert!(health2.ok);
    let pool2 = conn2.require_pool().expect("pool2");

    let biz = business_service::get_business_profile(&pool2)
        .await
        .expect("get biz")
        .expect("biz exists");
    assert_eq!(biz.name, "Persistent Studio");

    let clients = client_service::list_clients(&pool2, ListClientsParams::default())
        .await
        .expect("list clients");
    assert_eq!(clients.total, 1);
    assert_eq!(clients.items[0].full_name, "Long-Term Client");

    let settings = settings_service::get_settings(&pool2)
        .await
        .expect("get settings");
    assert_eq!(settings.theme_mode, "dark");
    assert_eq!(settings.accent_color, "blue");
    assert_eq!(settings.auto_backup_retention, 15);

    let backups = backup::list_backups(&resolved_v2.paths).expect("list backups");
    assert_eq!(backups.len(), 1);
}
