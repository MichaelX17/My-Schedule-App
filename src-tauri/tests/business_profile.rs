use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use my_schedule_app_lib::database;
use my_schedule_app_lib::database::repositories::business_repo;
use my_schedule_app_lib::errors::AppError;
use my_schedule_app_lib::models::business::{
    BusinessProfile, CreateBusinessProfileInput, UpdateBusinessProfileInput,
};
use my_schedule_app_lib::platform;
use my_schedule_app_lib::services::business_service;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Minimal valid PNG header + IHDR bytes for magic-byte validation tests.
const SAMPLE_PNG_BYTES: &[u8] = &[
    0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, b'I', b'H', b'D', b'R',
];

/// Minimal valid JPEG header bytes.
const SAMPLE_JPEG_BYTES: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F'];

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
            "my_schedule_app_biz_{label}_{}_{}_{}",
            std::process::id(),
            nanos,
            seq
        ));
        fs::create_dir_all(&dir).expect("create test temp dir");
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
async fn fresh_install_reports_onboarding_incomplete_and_detected_timezone() {
    let temp = TestDir::new("fresh_onboarding");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    let (conn, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool = &conn.require_pool().expect("pool");

    let status = business_service::get_onboarding_status(pool)
        .await
        .expect("get onboarding status");

    assert!(!status.completed);
    assert!(status.business.is_none());
    assert!(!status.detected_timezone.is_empty());

    conn.close().await;
}

#[tokio::test]
async fn validation_rejects_blank_name_invalid_timezone_and_bad_email() {
    let temp = TestDir::new("validation");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    let (conn, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool = &conn.require_pool().expect("pool");

    // Blank business name
    let err = business_service::create_business_profile(
        pool,
        &paths.logos_dir,
        CreateBusinessProfileInput {
            name: "   ".into(),
            address: None,
            city: None,
            region: None,
            postal_code: None,
            country: None,
            phone: None,
            email: None,
            website: None,
            timezone: "UTC".into(),
            logo_bytes: None,
        },
    )
    .await
    .expect_err("blank name must fail");
    assert!(matches!(err, AppError::Validation(_)));

    // Invalid IANA timezone
    let err = business_service::create_business_profile(
        pool,
        &paths.logos_dir,
        CreateBusinessProfileInput {
            name: "Studio Norte".into(),
            address: None,
            city: None,
            region: None,
            postal_code: None,
            country: None,
            phone: None,
            email: None,
            website: None,
            timezone: "Mars/Olympus_Mons".into(),
            logo_bytes: None,
        },
    )
    .await
    .expect_err("invalid timezone must fail");
    assert!(matches!(err, AppError::Validation(_)));

    // Invalid email
    let err = business_service::create_business_profile(
        pool,
        &paths.logos_dir,
        CreateBusinessProfileInput {
            name: "Studio Norte".into(),
            address: None,
            city: None,
            region: None,
            postal_code: None,
            country: None,
            phone: None,
            email: Some("not-an-email".into()),
            website: None,
            timezone: "Europe/Madrid".into(),
            logo_bytes: None,
        },
    )
    .await
    .expect_err("invalid email must fail");
    assert!(matches!(err, AppError::Validation(_)));

    // Unsupported logo format (SVG / text)
    let err = business_service::create_business_profile(
        pool,
        &paths.logos_dir,
        CreateBusinessProfileInput {
            name: "Studio Norte".into(),
            address: None,
            city: None,
            region: None,
            postal_code: None,
            country: None,
            phone: None,
            email: None,
            website: None,
            timezone: "Europe/Madrid".into(),
            logo_bytes: Some(b"<svg xmlns='http://www.w3.org/2000/svg'></svg>".to_vec()),
        },
    )
    .await
    .expect_err("SVG logo must be rejected");
    assert!(matches!(err, AppError::Validation(_)));

    // Oversized logo (> 2 MiB)
    let mut huge_png = SAMPLE_PNG_BYTES.to_vec();
    huge_png.resize(business_service::MAX_LOGO_SIZE_BYTES + 1, 0);
    let err = business_service::create_business_profile(
        pool,
        &paths.logos_dir,
        CreateBusinessProfileInput {
            name: "Studio Norte".into(),
            address: None,
            city: None,
            region: None,
            postal_code: None,
            country: None,
            phone: None,
            email: None,
            website: None,
            timezone: "Europe/Madrid".into(),
            logo_bytes: Some(huge_png),
        },
    )
    .await
    .expect_err("oversized logo must be rejected");
    assert!(matches!(err, AppError::Validation(_)));

    conn.close().await;
}

#[tokio::test]
async fn one_business_invariant_enforced_in_service_and_database() {
    let temp = TestDir::new("singleton");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    let (conn, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool = &conn.require_pool().expect("pool");

    let created = business_service::create_business_profile(
        pool,
        &paths.logos_dir,
        CreateBusinessProfileInput {
            name: "First Business".into(),
            address: None,
            city: None,
            region: None,
            postal_code: None,
            country: None,
            phone: None,
            email: None,
            website: None,
            timezone: "America/New_York".into(),
            logo_bytes: None,
        },
    )
    .await
    .expect("first business creation should succeed");

    // Service layer rejects second business
    let service_err = business_service::create_business_profile(
        pool,
        &paths.logos_dir,
        CreateBusinessProfileInput {
            name: "Second Business".into(),
            address: None,
            city: None,
            region: None,
            postal_code: None,
            country: None,
            phone: None,
            email: None,
            website: None,
            timezone: "UTC".into(),
            logo_bytes: None,
        },
    )
    .await
    .expect_err("second business must fail at service layer");
    assert!(matches!(service_err, AppError::AlreadyExists(_)));

    // Direct repository / SQLite constraint also rejects a second row
    let duplicate = BusinessProfile {
        id: "second-id".into(),
        name: "Direct Duplicate".into(),
        logo_filename: None,
        address: None,
        city: None,
        region: None,
        postal_code: None,
        country: None,
        phone: None,
        email: None,
        website: None,
        timezone: "UTC".into(),
        onboarding_completed: true,
        created_at: created.created_at.clone(),
        updated_at: created.updated_at.clone(),
    };
    let db_err = business_repo::insert(pool, &duplicate)
        .await
        .expect_err("second business must fail at SQLite unique constraint");
    assert!(matches!(db_err, AppError::AlreadyExists(_)));

    conn.close().await;
}

#[tokio::test]
async fn create_update_logo_lifecycle_and_persistence_across_restart() {
    let temp = TestDir::new("lifecycle_restart");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    // Simulate an external source logo file that will be deleted after import.
    let external_logo = temp.path().join("source_logo.png");
    fs::write(&external_logo, SAMPLE_PNG_BYTES).expect("write external logo");
    let source_bytes = fs::read(&external_logo).expect("read external logo");

    let (conn1, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool1 = &conn1.require_pool().expect("pool1");

    let created = business_service::create_business_profile(
        pool1,
        &paths.logos_dir,
        CreateBusinessProfileInput {
            name: "  Atelier Sol  ".into(),
            address: Some("".into()),
            city: Some("Madrid".into()),
            region: None,
            postal_code: None,
            country: Some("Spain".into()),
            phone: None,
            email: Some("hello@ateliersol.example".into()),
            website: None,
            timezone: "Europe/Madrid".into(),
            logo_bytes: Some(source_bytes),
        },
    )
    .await
    .expect("create profile with logo");

    assert_eq!(created.name, "Atelier Sol");
    assert_eq!(created.address, None); // blank string normalized to None
    assert_eq!(created.city.as_deref(), Some("Madrid"));
    assert!(created.onboarding_completed);
    let first_logo_name = created
        .logo_filename
        .clone()
        .expect("managed logo filename should be stored");
    assert!(first_logo_name.ends_with(".png"));
    assert!(!first_logo_name.contains('/') && !first_logo_name.contains('\\'));

    // Delete original source file to prove managed logo storage is independent.
    fs::remove_file(&external_logo).expect("delete original source file");

    // Update profile fields
    let updated = business_service::update_business_profile(
        pool1,
        UpdateBusinessProfileInput {
            name: "Atelier Sol Studio".into(),
            address: Some("Calle Mayor 10".into()),
            city: Some("Madrid".into()),
            region: Some("Madrid".into()),
            postal_code: Some("28013".into()),
            country: Some("Spain".into()),
            phone: Some("+34 600 123 456".into()),
            email: Some("team@ateliersol.example".into()),
            website: Some("https://ateliersol.example".into()),
            timezone: "UTC".into(),
        },
    )
    .await
    .expect("update profile");

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "Atelier Sol Studio");
    assert_eq!(updated.timezone, "UTC");
    assert_eq!(
        updated.logo_filename.as_deref(),
        Some(first_logo_name.as_str())
    );

    // Replace logo with JPEG and confirm old managed file is removed
    let replaced_logo =
        business_service::upload_business_logo(pool1, &paths.logos_dir, SAMPLE_JPEG_BYTES.to_vec())
            .await
            .expect("replace logo with jpeg");
    assert_eq!(replaced_logo.mime_type, "image/jpeg");
    assert!(replaced_logo.filename.ends_with(".jpg"));
    assert!(!paths.logos_dir.join(&first_logo_name).exists());
    assert!(paths.logos_dir.join(&replaced_logo.filename).exists());

    conn1.close().await;

    // Simulate app restart by reopening the database
    let (conn2, _) = database::initialize(&paths.database_path)
        .await
        .expect("reopen db");
    let pool2 = &conn2.require_pool().expect("pool2");

    let status_after_restart = business_service::get_onboarding_status(pool2)
        .await
        .expect("onboarding status after restart");
    assert!(status_after_restart.completed);
    let reloaded = status_after_restart
        .business
        .expect("business should persist across restart");
    assert_eq!(reloaded.name, "Atelier Sol Studio");
    assert_eq!(reloaded.timezone, "UTC");
    assert_eq!(
        reloaded.logo_filename.as_deref(),
        Some(replaced_logo.filename.as_str())
    );

    let loaded_logo = business_service::get_business_logo(pool2, &paths.logos_dir)
        .await
        .expect("load managed logo")
        .expect("logo should exist after restart");
    assert_eq!(loaded_logo.mime_type, "image/jpeg");
    assert!(loaded_logo.data_url.starts_with("data:image/jpeg;base64,"));

    // Remove logo and confirm file is deleted
    let after_remove = business_service::remove_business_logo(pool2, &paths.logos_dir)
        .await
        .expect("remove logo");
    assert!(after_remove.logo_filename.is_none());
    assert!(!paths.logos_dir.join(&replaced_logo.filename).exists());

    conn2.close().await;
}
