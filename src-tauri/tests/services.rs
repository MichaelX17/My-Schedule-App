use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use my_schedule_app_lib::database;
use my_schedule_app_lib::errors::AppError;
use my_schedule_app_lib::models::service::{
    CreateServiceInput, ListServicesParams, ServiceSortBy, ServiceStatusFilter, UpdateServiceInput,
};
use my_schedule_app_lib::platform;
use my_schedule_app_lib::services::service_catalog_service;

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
            "my_schedule_app_svc_{label}_{}_{}_{}",
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
async fn service_validation_rejects_blank_name_and_out_of_range_durations() {
    let temp = TestDir::new("validation");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    let (conn, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool = &conn.require_pool().expect("pool");

    // Blank name rejected
    let err = service_catalog_service::create_service(
        pool,
        CreateServiceInput {
            name: "   ".into(),
            description: None,
            default_duration_minutes: 30,
        },
    )
    .await
    .expect_err("blank name must fail");
    assert!(matches!(err, AppError::Validation(_)));

    // Duration too short (< 5)
    let err = service_catalog_service::create_service(
        pool,
        CreateServiceInput {
            name: "Quick Check".into(),
            description: None,
            default_duration_minutes: 4,
        },
    )
    .await
    .expect_err("duration < 5 must fail");
    assert!(matches!(err, AppError::Validation(_)));

    // Duration too long (> 1440)
    let err = service_catalog_service::create_service(
        pool,
        CreateServiceInput {
            name: "Marathon Session".into(),
            description: None,
            default_duration_minutes: 1441,
        },
    )
    .await
    .expect_err("duration > 1440 must fail");
    assert!(matches!(err, AppError::Validation(_)));

    // Boundary durations (5 and 1440) succeed
    let short = service_catalog_service::create_service(
        pool,
        CreateServiceInput {
            name: "  Quick Consult  ".into(),
            description: Some("  ".into()),
            default_duration_minutes: 5,
        },
    )
    .await
    .expect("5 min service should succeed");
    assert_eq!(short.name, "Quick Consult");
    assert_eq!(short.description, None);
    assert_eq!(short.default_duration_minutes, 5);

    let full_day = service_catalog_service::create_service(
        pool,
        CreateServiceInput {
            name: "Full Day Workshop".into(),
            description: Some("All-day session".into()),
            default_duration_minutes: 1440,
        },
    )
    .await
    .expect("1440 min service should succeed");
    assert_eq!(full_day.default_duration_minutes, 1440);

    conn.close().await;
}

#[tokio::test]
async fn service_active_inactive_archive_filtering_snapshot_and_restart() {
    let temp = TestDir::new("lifecycle");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    let (conn1, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool1 = &conn1.require_pool().expect("pool1");

    let s1 = service_catalog_service::create_service(
        pool1,
        CreateServiceInput {
            name: "Initial Consultation".into(),
            description: Some("First-time intake session".into()),
            default_duration_minutes: 60,
        },
    )
    .await
    .expect("create s1");

    let s2 = service_catalog_service::create_service(
        pool1,
        CreateServiceInput {
            name: "Follow-Up Session".into(),
            description: None,
            default_duration_minutes: 30,
        },
    )
    .await
    .expect("create s2");

    // Capture snapshot before later edits
    let original_snapshot = s1.to_snapshot();
    assert_eq!(original_snapshot.service_name, "Initial Consultation");
    assert_eq!(original_snapshot.duration_minutes, 60);

    // Rename s1 and verify snapshot remains unaffected
    let updated_s1 = service_catalog_service::update_service(
        pool1,
        UpdateServiceInput {
            id: s1.id.clone(),
            name: "Discovery Session".into(),
            description: Some("Updated intake".into()),
            default_duration_minutes: 45,
        },
    )
    .await
    .expect("update s1");
    assert_eq!(updated_s1.name, "Discovery Session");
    assert_eq!(updated_s1.default_duration_minutes, 45);
    assert_eq!(original_snapshot.service_name, "Initial Consultation");

    // Deactivate s1 -> excluded from default Active list, retrievable by ID and Inactive filter
    let deactivated = service_catalog_service::deactivate_service(pool1, &s1.id)
        .await
        .expect("deactivate s1");
    assert!(!deactivated.is_active);
    assert!(!deactivated.is_archived);

    // Archive s2 -> excluded from default Active list, cannot be activated directly while archived
    let archived = service_catalog_service::archive_service(pool1, &s2.id)
        .await
        .expect("archive s2");
    assert!(archived.is_archived);
    assert!(!archived.is_active);
    assert!(archived.archived_at.is_some());

    let activate_archived_err = service_catalog_service::activate_service(pool1, &s2.id)
        .await
        .expect_err("archived service must be restored before activating");
    assert!(matches!(activate_archived_err, AppError::Validation(_)));

    // Default Active list returns 0 items, counts reflect 0 active / 1 inactive / 1 archived
    let active_list = service_catalog_service::list_services(pool1, ListServicesParams::default())
        .await
        .expect("list active");
    assert_eq!(active_list.items.len(), 0);
    assert_eq!(active_list.active_count, 0);
    assert_eq!(active_list.inactive_count, 1);
    assert_eq!(active_list.archived_count, 1);

    // Inactive and Archived services remain retrievable by ID for historical appointments
    assert_eq!(
        service_catalog_service::get_service(pool1, &s1.id)
            .await
            .expect("get inactive by id")
            .name,
        "Discovery Session"
    );
    assert_eq!(
        service_catalog_service::get_service(pool1, &s2.id)
            .await
            .expect("get archived by id")
            .name,
        "Follow-Up Session"
    );

    conn1.close().await;

    // Reopen database (simulate restart)
    let (conn2, _) = database::initialize(&paths.database_path)
        .await
        .expect("reopen db");
    let pool2 = &conn2.require_pool().expect("pool2");

    let all = service_catalog_service::list_services(
        pool2,
        ListServicesParams {
            filter: Some(ServiceStatusFilter::All),
            sort_by: Some(ServiceSortBy::DurationAsc),
            ..Default::default()
        },
    )
    .await
    .expect("list all after restart");
    assert_eq!(all.items.len(), 2);
    assert_eq!(all.items[0].name, "Follow-Up Session"); // 30 min
    assert_eq!(all.items[1].name, "Discovery Session"); // 45 min

    // Reactivate s1 and restore s2
    let reactivated = service_catalog_service::activate_service(pool2, &s1.id)
        .await
        .expect("reactivate s1");
    assert!(reactivated.is_active);

    let restored = service_catalog_service::restore_service(pool2, &s2.id)
        .await
        .expect("restore s2");
    assert!(!restored.is_archived);
    assert!(restored.is_active);
    assert!(restored.archived_at.is_none());

    let active_after = service_catalog_service::list_services(
        pool2,
        ListServicesParams {
            search: Some("Discovery".into()),
            ..Default::default()
        },
    )
    .await
    .expect("search active");
    assert_eq!(active_after.items.len(), 1);
    assert_eq!(active_after.items[0].id, s1.id);

    conn2.close().await;
}
