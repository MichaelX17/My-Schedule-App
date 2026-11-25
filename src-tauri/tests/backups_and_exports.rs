use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use my_schedule_app_lib::backup::{self, BackupType};
use my_schedule_app_lib::database;
use my_schedule_app_lib::errors::AppError;
use my_schedule_app_lib::export::{
    self, CsvEntity, VersionedBusinessExport, EXPORT_SCHEMA_VERSION,
};
use my_schedule_app_lib::models::appointment::CreateAppointmentInput;
use my_schedule_app_lib::models::business::CreateBusinessProfileInput;
use my_schedule_app_lib::models::client::{CreateClientInput, ListClientsParams};
use my_schedule_app_lib::models::service::CreateServiceInput;
use my_schedule_app_lib::platform;
use my_schedule_app_lib::services::{
    appointment_service, business_service, client_service, service_catalog_service,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

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
            "my_schedule_app_bkp_{label}_{}_{}_{}",
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

async fn setup_fixture(
    temp: &TestDir,
) -> (database::connection::DatabaseConnection, platform::AppPaths) {
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
            name: "Backup Test Studio".into(),
            address: Some("123 Main St".into()),
            city: Some("Madrid".into()),
            region: None,
            postal_code: None,
            country: Some("Spain".into()),
            phone: None,
            email: Some("hello@example.com".into()),
            website: None,
            timezone: "Europe/Madrid".into(),
            logo_bytes: None,
        },
    )
    .await
    .expect("create business");

    (conn, paths)
}

#[tokio::test]
async fn csv_escaping_and_json_versioned_export() {
    // 1. Unit assertions for RFC 4180 CSV escaping
    assert_eq!(export::escape_csv_field("plain"), "plain");
    assert_eq!(export::escape_csv_field("a,b"), "\"a,b\"");
    assert_eq!(export::escape_csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
    assert_eq!(export::escape_csv_field("line1\nline2"), "\"line1\nline2\"");

    // 2. End-to-end JSON + CSV exports
    let temp = TestDir::new("exports");
    let (conn, paths) = setup_fixture(&temp).await;
    let pool = conn.require_pool().expect("pool");

    let client = client_service::create_client(
        &pool,
        CreateClientInput {
            full_name: "O'Connor, \"Ana\"".into(),
            phone: Some("+34 600 111 222".into()),
            email: Some("ana@example.com".into()),
            address: None,
            notes: Some("Line 1\nLine 2, with comma".into()),
        },
    )
    .await
    .expect("create client");

    let service = service_catalog_service::create_service(
        &pool,
        CreateServiceInput {
            name: "Initial Review".into(),
            description: Some("Detailed, \"thorough\" review".into()),
            default_duration_minutes: 60,
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
            start_local: "2026-07-10T10:00".into(),
            end_local: "2026-07-10T11:00".into(),
            dst_disambiguation: None,
            service_ids: vec![service.id],
            notes: Some("Session note".into()),
        },
    )
    .await
    .expect("create appointment");

    // JSON export
    let json_res = export::export_json(&pool, &paths.exports_dir, Some("snapshot_v1"))
        .await
        .expect("export json");
    assert_eq!(json_res.filename, "snapshot_v1.json");
    assert!(Path::new(&json_res.file_path).is_file());

    let raw_json = fs::read_to_string(&json_res.file_path).expect("read json");
    let parsed: VersionedBusinessExport = serde_json::from_str(&raw_json).expect("parse json");
    assert_eq!(parsed.export_schema_version, EXPORT_SCHEMA_VERSION);
    assert_eq!(parsed.clients.len(), 1);
    assert_eq!(parsed.services.len(), 1);
    assert_eq!(parsed.appointments.len(), 1);
    assert_eq!(parsed.appointment_services.len(), 1);
    // Ensure path traversal is rejected in custom filename
    let bad_name = export::export_json(&pool, &paths.exports_dir, Some("../evil.json")).await;
    assert!(matches!(bad_name, Err(AppError::Validation(_))));

    // CSV exports for all 4 practical entities
    for entity in [
        CsvEntity::Clients,
        CsvEntity::Services,
        CsvEntity::Appointments,
        CsvEntity::AppointmentServices,
    ] {
        let csv_res = export::export_csv(&pool, &paths.exports_dir, entity, None)
            .await
            .expect("export csv");
        assert_eq!(csv_res.record_count, 1);
        let content = fs::read_to_string(&csv_res.file_path).expect("read csv");
        if entity == CsvEntity::Clients {
            assert!(content.contains("\"O'Connor, \"\"Ana\"\"\""));
            assert!(content.contains("\"Line 1\nLine 2, with comma\""));
        }
    }
}

#[tokio::test]
async fn sqlite_safe_backup_integrity_automatic_retention_and_incompatible_schema_rejection() {
    let temp = TestDir::new("retention");
    let (conn, paths) = setup_fixture(&temp).await;
    let pool = conn.require_pool().expect("pool");

    // 1. Manual backup succeeds and passes validation
    let manual = backup::create_backup(
        &pool,
        &paths,
        BackupType::Manual,
        Some("Manual test".into()),
        None,
    )
    .await
    .expect("create manual backup");
    assert_eq!(manual.integrity_status, "ok");
    assert_eq!(
        manual.schema_version,
        backup::current_supported_schema_version()
    );

    let manual_path =
        backup::resolve_managed_backup_path(&paths, BackupType::Manual, &manual.filename).unwrap();
    let valid = backup::validate_backup_file(&manual_path)
        .await
        .expect("validate manual");
    assert!(valid.valid);
    assert!(valid.integrity_ok);
    assert!(valid.schema_compatible);

    // 2. Automatic retention: create 5 automatic backups with retention = 3 -> only 3 remain, manual untouched
    for i in 0..5 {
        backup::create_backup(
            &pool,
            &paths,
            BackupType::Automatic,
            Some(format!("Auto {i}")),
            Some(3),
        )
        .await
        .expect("create auto backup");
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }

    let all_backups = backup::list_backups(&paths).expect("list backups");
    let auto_count = all_backups
        .iter()
        .filter(|b| b.backup_type == BackupType::Automatic)
        .count();
    let manual_count = all_backups
        .iter()
        .filter(|b| b.backup_type == BackupType::Manual)
        .count();
    assert_eq!(auto_count, 3);
    assert_eq!(manual_count, 1);

    // 3. Corrupted backup file is rejected cleanly
    let corrupt_path = paths.manual_backups_dir.join("corrupt_test.sqlite");
    fs::write(&corrupt_path, b"this is not a valid sqlite file").expect("write corrupt");
    let corrupt_res = backup::validate_backup_file(&corrupt_path).await;
    assert!(matches!(corrupt_res, Err(AppError::DatabaseCorrupted(_))));

    // 4. Backup with a newer schema version (9999) is flagged incompatible and rejected before restore
    let future_path = paths.manual_backups_dir.join("future_schema.sqlite");
    fs::copy(&manual_path, &future_path).expect("copy backup");
    {
        let opts = SqliteConnectOptions::new().filename(&future_path);
        let tmp_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .expect("open future copy");
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) VALUES (9999, 'future_migration', CURRENT_TIMESTAMP, 1, X'00', 1)",
        )
        .execute(&tmp_pool)
        .await
        .expect("insert future version");
        tmp_pool.close().await;
    }

    let future_validation = backup::validate_backup_file(&future_path)
        .await
        .expect("validate future");
    assert!(!future_validation.valid);
    assert!(!future_validation.schema_compatible);
    assert_eq!(future_validation.backup_schema_version, 9999);

    let restore_err =
        backup::restore_backup(&conn, &paths, BackupType::Manual, "future_schema.sqlite").await;
    assert!(matches!(restore_err, Err(AppError::Validation(_))));
}

#[tokio::test]
async fn restore_creates_safety_backup_replaces_data_and_preserves_recovery() {
    let temp = TestDir::new("restore");
    let (conn, paths) = setup_fixture(&temp).await;

    // Create Client A, then take backup 1
    {
        let pool = conn.require_pool().expect("pool");
        client_service::create_client(
            &pool,
            CreateClientInput {
                full_name: "Client Alpha".into(),
                phone: None,
                email: None,
                address: None,
                notes: None,
            },
        )
        .await
        .expect("create alpha");
    }

    let backup_1 = {
        let pool = conn.require_pool().expect("pool");
        backup::create_backup(
            &pool,
            &paths,
            BackupType::Manual,
            Some("Before Beta".into()),
            None,
        )
        .await
        .expect("create backup 1")
    };

    // Create Client Beta after backup 1
    {
        let pool = conn.require_pool().expect("pool");
        client_service::create_client(
            &pool,
            CreateClientInput {
                full_name: "Client Beta".into(),
                phone: None,
                email: None,
                address: None,
                notes: None,
            },
        )
        .await
        .expect("create beta");

        let before = client_service::list_clients(&pool, ListClientsParams::default())
            .await
            .expect("list before restore");
        assert_eq!(before.total, 2);
    }

    // Restore backup 1 -> safety backup created automatically, Client Beta disappears, Client Alpha remains
    let restore_res = backup::restore_backup(&conn, &paths, BackupType::Manual, &backup_1.filename)
        .await
        .expect("restore backup 1");

    assert_eq!(restore_res.safety_backup.backup_type, BackupType::Safety);
    {
        let pool = conn.require_pool().expect("pool after restore");
        let after = client_service::list_clients(&pool, ListClientsParams::default())
            .await
            .expect("list after restore");
        assert_eq!(after.total, 1);
        assert_eq!(after.items[0].full_name, "Client Alpha");
    }

    // Restore the safety backup -> Client Beta is recovered!
    backup::restore_backup(
        &conn,
        &paths,
        BackupType::Safety,
        &restore_res.safety_backup.filename,
    )
    .await
    .expect("restore safety backup");

    {
        let pool = conn.require_pool().expect("pool after safety restore");
        let recovered = client_service::list_clients(&pool, ListClientsParams::default())
            .await
            .expect("list recovered");
        assert_eq!(recovered.total, 2);
    }
}
