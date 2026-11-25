use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use my_schedule_app_lib::database;
use my_schedule_app_lib::database::health;
use my_schedule_app_lib::database::migrations;
use my_schedule_app_lib::errors::AppError;
use my_schedule_app_lib::platform::{self, StorageMode};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Minimal RAII temporary directory helper (no external dev-dependency needed).
struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "my_schedule_app_test_{label}_{}_{}_{}",
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

impl Drop for TestTempDir {
    fn drop(&mut self) {
        // Restore write permissions on Unix before cleanup in case a test locked a subdir.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(entries) = fs::read_dir(&self.path) {
                for entry in entries.flatten() {
                    let _ = fs::set_permissions(entry.path(), fs::Permissions::from_mode(0o755));
                }
            }
            let _ = fs::set_permissions(&self.path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn storage_prefers_portable_when_adjacent_dir_is_writable() {
    let exe_dir = TestTempDir::new("portable_exe");
    let fallback_dir = TestTempDir::new("portable_fallback");

    let resolved = platform::resolve_storage_paths(Some(exe_dir.path()), Some(fallback_dir.path()))
        .expect("should resolve portable storage");

    assert_eq!(resolved.mode, StorageMode::Portable);
    assert_eq!(resolved.paths.data_dir, exe_dir.path().join("data"));
    assert!(resolved.fallback_reason.is_none());

    resolved
        .paths
        .ensure_directories()
        .expect("ensure_directories should succeed");
    assert!(resolved.paths.automatic_backups_dir.is_dir());
    assert!(resolved.paths.manual_backups_dir.is_dir());
    assert!(resolved.paths.exports_dir.is_dir());
    assert!(resolved.paths.logos_dir.is_dir());
    assert!(resolved.paths.logs_dir.is_dir());
    assert!(resolved.paths.updates_dir.is_dir());
}

#[test]
fn storage_falls_back_when_adjacent_data_path_is_blocked_by_file() {
    let exe_dir = TestTempDir::new("blocked_exe");
    let fallback_dir = TestTempDir::new("blocked_fallback");

    // Cross-platform unwritable adjacent simulation: a regular file occupies `<exe_dir>/data`.
    fs::write(exe_dir.path().join("data"), b"blocker").expect("write blocker file");

    let resolved = platform::resolve_storage_paths(Some(exe_dir.path()), Some(fallback_dir.path()))
        .expect("should fall back to per-user app data dir");

    assert_eq!(resolved.mode, StorageMode::Fallback);
    assert_eq!(resolved.paths.data_dir, fallback_dir.path().join("data"));
    assert!(resolved.fallback_reason.is_some());
}

#[cfg(unix)]
#[test]
fn storage_falls_back_when_adjacent_dir_is_readonly_unix() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TestTempDir::new("readonly_unix");
    let exe_dir = temp.path().join("readonly_exe");
    let fallback_dir = temp.path().join("fallback_dir");
    fs::create_dir_all(&exe_dir).expect("create exe_dir");
    fs::create_dir_all(&fallback_dir).expect("create fallback_dir");

    fs::set_permissions(&exe_dir, fs::Permissions::from_mode(0o555)).expect("set readonly mode");

    // Skip assertion if running as root (UID 0 ignores 0o555 write restriction).
    let probe_still_writable = fs::write(exe_dir.join(".root_check"), b"x").is_ok();
    if probe_still_writable {
        let _ = fs::remove_file(exe_dir.join(".root_check"));
        return;
    }

    let resolved = platform::resolve_storage_paths(Some(&exe_dir), Some(&fallback_dir))
        .expect("should fall back when exe_dir is read-only");

    assert_eq!(resolved.mode, StorageMode::Fallback);
    assert_eq!(resolved.paths.data_dir, fallback_dir.join("data"));
    assert!(resolved.fallback_reason.is_some());
}

#[test]
fn storage_errors_when_no_writable_directory_is_available() {
    let exe_dir = TestTempDir::new("no_fallback_exe");
    fs::write(exe_dir.path().join("data"), b"blocker").expect("write blocker file");

    let err = platform::resolve_storage_paths(Some(exe_dir.path()), None)
        .expect_err("should fail when adjacent is blocked and no fallback exists");

    assert!(matches!(err, AppError::StorageUnavailable(_)));
}

#[test]
fn safe_join_child_prevents_path_traversal() {
    let base = Path::new("/tmp/my_schedule_base");

    assert_eq!(
        platform::safe_join_child(base, "backup_2026.sqlite").expect("valid filename"),
        base.join("backup_2026.sqlite")
    );

    for invalid in [
        "",
        ".",
        "..",
        "../secret",
        "sub/file.txt",
        "sub\\file.txt",
        "/etc/passwd",
    ] {
        let err = platform::safe_join_child(base, invalid)
            .expect_err(&format!("expected traversal rejection for {invalid:?}"));
        assert!(matches!(err, AppError::InvalidPath(_)));
    }
}

#[test]
fn serialized_ipc_errors_do_not_leak_sensitive_paths() {
    let err = AppError::DatabaseInit("/home/secret-user/private/database.sqlite".into());
    let json = serde_json::to_value(&err).expect("serialize AppError");

    assert_eq!(json["code"], "DATABASE_INIT_FAILED");
    let message = json["message"].as_str().expect("message string");
    assert!(!message.contains("secret-user"));
    assert!(!message.contains("database.sqlite"));
}

#[tokio::test]
async fn database_creates_migrates_reopens_and_preserves_data() {
    let temp = TestTempDir::new("db_lifecycle");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    // First startup: creates DB, runs migrations, passes health check.
    let (conn1, health1) = database::initialize(&paths.database_path)
        .await
        .expect("first database init should succeed");
    assert!(health1.ok);
    assert!(health1.integrity_ok);
    assert!(health1.foreign_keys_enabled);
    assert_eq!(health1.journal_mode, "wal");
    assert!(health1.applied_migrations >= 1);
    assert!(health1.latest_migration_version.is_some());

    let pool1 = &conn1.require_pool().expect("pool1");
    let schema_ver: String = sqlx::query_scalar(
        "SELECT value FROM app_metadata WHERE key = 'schema_foundation_version'",
    )
    .fetch_one(pool1)
    .await
    .expect("seed metadata should exist");
    assert_eq!(schema_ver, "1");

    // Write a custom row to verify reopen does not overwrite the database.
    sqlx::query("INSERT INTO app_metadata (key, value, updated_at) VALUES (?1, ?2, ?3)")
        .bind("test_marker")
        .bind("persisted_ok")
        .bind("2026-11-24T00:00:00Z")
        .execute(pool1)
        .await
        .expect("insert marker");

    conn1.close().await;

    // Second startup (repeat init): idempotent migrations and preserved data.
    let (conn2, health2) = database::initialize(&paths.database_path)
        .await
        .expect("repeat database init should succeed");
    assert!(health2.ok);
    assert_eq!(health2.applied_migrations, health1.applied_migrations);

    let pool2 = &conn2.require_pool().expect("pool2");
    let marker: String =
        sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'test_marker'")
            .fetch_one(pool2)
            .await
            .expect("persisted marker should survive reopen");
    assert_eq!(marker, "persisted_ok");

    let live_health = health::check_health(pool2)
        .await
        .expect("health check should succeed");
    assert_eq!(live_health, health2);

    conn2.close().await;
}

#[tokio::test]
async fn database_enforces_foreign_keys_on_connections() {
    let temp = TestTempDir::new("db_fk");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    let (conn, _health) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool = &conn.require_pool().expect("pool");

    sqlx::query("CREATE TABLE fk_parent (id INTEGER PRIMARY KEY)")
        .execute(pool)
        .await
        .expect("create parent");
    sqlx::query(
        "CREATE TABLE fk_child (id INTEGER PRIMARY KEY, parent_id INTEGER NOT NULL REFERENCES fk_parent(id))",
    )
    .execute(pool)
    .await
    .expect("create child");

    // Inserting a child referencing a missing parent must fail due to PRAGMA foreign_keys = ON.
    let fk_err = sqlx::query("INSERT INTO fk_child (id, parent_id) VALUES (1, 999)")
        .execute(pool)
        .await
        .expect_err("foreign key violation must be rejected");

    let err_text = fk_err.to_string().to_ascii_lowercase();
    assert!(
        err_text.contains("foreign key"),
        "expected foreign key error, got: {err_text}"
    );

    conn.close().await;
}

#[tokio::test]
async fn corrupted_database_file_returns_typed_error_without_overwriting() {
    let temp = TestTempDir::new("db_corrupt");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    let corrupt_bytes = b"this is definitely not a valid sqlite database header!!";
    fs::write(&paths.database_path, corrupt_bytes).expect("write corrupt file");

    let err = database::initialize(&paths.database_path)
        .await
        .expect_err("corrupted database must fail initialization");

    assert!(
        matches!(err, AppError::DatabaseCorrupted(_)),
        "expected DatabaseCorrupted, got: {err:?}"
    );

    // Confirm existing file was NOT silently overwritten or truncated.
    let remaining_bytes = fs::read(&paths.database_path).expect("read corrupt file back");
    assert_eq!(remaining_bytes, corrupt_bytes);
}

#[tokio::test]
async fn failed_migration_returns_typed_error_and_preserves_existing_data() {
    let temp = TestTempDir::new("db_bad_migration");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    let (conn, _) = database::initialize(&paths.database_path)
        .await
        .expect("baseline init");
    let pool = &conn.require_pool().expect("pool");

    // Build a temporary migration directory with a syntactically invalid migration.
    let bad_migrations_dir = temp.path().join("bad_migrations");
    fs::create_dir_all(&bad_migrations_dir).expect("create bad migrations dir");
    fs::write(
        bad_migrations_dir.join("9999_broken.sql"),
        "THIS IS INVALID SQL SYNTAX;",
    )
    .expect("write broken migration");

    let mut bad_migrator = sqlx::migrate::Migrator::new(bad_migrations_dir)
        .await
        .expect("load bad migrator");
    bad_migrator.set_ignore_missing(true);

    let err = migrations::run_migrator(pool, &bad_migrator)
        .await
        .expect_err("broken migration should fail");
    assert!(matches!(err, AppError::MigrationFailed(_)));

    // Existing metadata must remain intact.
    let schema_ver: String = sqlx::query_scalar(
        "SELECT value FROM app_metadata WHERE key = 'schema_foundation_version'",
    )
    .fetch_one(pool)
    .await
    .expect("existing metadata must remain intact");
    assert_eq!(schema_ver, "1");

    conn.close().await;
}
