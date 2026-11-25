use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use my_schedule_app_lib::backup::BackupType;
use my_schedule_app_lib::database;
use my_schedule_app_lib::errors::AppError;
use my_schedule_app_lib::platform;
use my_schedule_app_lib::updater::{
    self, PlatformArtifact, UpdateCheckStatus, UpdateManifest, VersionComparison, GITHUB_REPO,
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
            "my_schedule_app_upd_{label}_{}_{}_{}",
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

fn make_test_keypair(seed_byte: u8) -> (SigningKey, String) {
    let signing_key = SigningKey::from_bytes(&[seed_byte; 32]);
    let pubkey_b64 = BASE64.encode(signing_key.verifying_key().to_bytes());
    (signing_key, pubkey_b64)
}

fn make_multi_platform_manifest(win_sha: &str, linux_sha: &str) -> UpdateManifest {
    let mut platforms = BTreeMap::new();
    platforms.insert(
        "windows-x86_64".to_string(),
        PlatformArtifact {
            artifact_filename: "MyScheduleApp-v0.2.0-windows-x64.exe".into(),
            download_url: format!(
                "https://github.com/{GITHUB_REPO}/releases/download/v0.2.0/MyScheduleApp-v0.2.0-windows-x64.exe"
            ),
            sha256: win_sha.to_string(),
        },
    );
    platforms.insert(
        "linux-x86_64".to_string(),
        PlatformArtifact {
            artifact_filename: "MyScheduleApp-v0.2.0-linux-x86_64.AppImage".into(),
            download_url: format!(
                "https://github.com/{GITHUB_REPO}/releases/download/v0.2.0/MyScheduleApp-v0.2.0-linux-x86_64.AppImage"
            ),
            sha256: linux_sha.to_string(),
        },
    );

    UpdateManifest {
        version: "0.2.0".into(),
        pub_date: "2026-12-01T12:00:00Z".into(),
        notes: "Cross-platform Windows (.exe) and Linux (.AppImage) release.".into(),
        target_schema_version: Some(7),
        platforms,
    }
}

#[test]
fn semver_version_comparison_and_downgrade_rejection() {
    assert_eq!(
        updater::compare_versions("0.1.0", "0.2.0").unwrap(),
        VersionComparison::UpdateAvailable
    );
    assert_eq!(
        updater::compare_versions("0.1.0", "v0.1.1").unwrap(),
        VersionComparison::UpdateAvailable
    );
    assert_eq!(
        updater::compare_versions("0.2.0", "0.2.0").unwrap(),
        VersionComparison::UpToDate
    );
    assert_eq!(
        updater::compare_versions("0.2.0", "0.1.9").unwrap(),
        VersionComparison::DowngradeRejected
    );
    assert!(matches!(
        updater::compare_versions("0.1.0", "not-a-version"),
        Err(AppError::Validation(_))
    ));
}

#[test]
fn ed25519_manifest_signature_verification_and_multi_platform_validation() {
    let (signing_key, pubkey_b64) = make_test_keypair(42);
    let (_other_key, wrong_pubkey_b64) = make_test_keypair(99);

    let win_sha = updater::compute_sha256_hex(b"windows-exe-bytes");
    let linux_sha = updater::compute_sha256_hex(b"linux-appimage-bytes");
    let valid_manifest = make_multi_platform_manifest(&win_sha, &linux_sha);

    let manifest_bytes = serde_json::to_vec(&valid_manifest).unwrap();
    let sig_b64 = BASE64.encode(signing_key.sign(&manifest_bytes).to_bytes());

    // 1. Valid signature + valid Windows & Linux entries succeeds
    let parsed = updater::parse_and_verify_manifest(&manifest_bytes, &sig_b64, &pubkey_b64)
        .expect("valid signed manifest");
    assert_eq!(
        updater::select_artifact_for_platform(&parsed, "windows-x86_64")
            .unwrap()
            .artifact_filename,
        "MyScheduleApp-v0.2.0-windows-x64.exe"
    );
    assert_eq!(
        updater::select_artifact_for_platform(&parsed, "linux-x86_64")
            .unwrap()
            .artifact_filename,
        "MyScheduleApp-v0.2.0-linux-x86_64.AppImage"
    );

    // 2. Tampered manifest fails signature check
    let mut tampered = manifest_bytes.clone();
    tampered.push(b' ');
    assert!(matches!(
        updater::parse_and_verify_manifest(&tampered, &sig_b64, &pubkey_b64),
        Err(AppError::Validation(_))
    ));

    // 3. Wrong public key fails signature check
    assert!(matches!(
        updater::parse_and_verify_manifest(&manifest_bytes, &sig_b64, &wrong_pubkey_b64),
        Err(AppError::Validation(_))
    ));

    // 4. Untrusted download URL is rejected
    let mut bad_url = valid_manifest.clone();
    bad_url
        .platforms
        .get_mut("windows-x86_64")
        .unwrap()
        .download_url = "https://evil.example.com/MyScheduleApp.exe".into();
    let bad_url_bytes = serde_json::to_vec(&bad_url).unwrap();
    let bad_url_sig = BASE64.encode(signing_key.sign(&bad_url_bytes).to_bytes());
    assert!(matches!(
        updater::parse_and_verify_manifest(&bad_url_bytes, &bad_url_sig, &pubkey_b64),
        Err(AppError::Validation(_))
    ));

    // 5. Path traversal in Linux filename is rejected
    let mut bad_name = valid_manifest;
    bad_name
        .platforms
        .get_mut("linux-x86_64")
        .unwrap()
        .artifact_filename = "../escape.AppImage".into();
    let bad_name_bytes = serde_json::to_vec(&bad_name).unwrap();
    let bad_name_sig = BASE64.encode(signing_key.sign(&bad_name_bytes).to_bytes());
    assert!(matches!(
        updater::parse_and_verify_manifest(&bad_name_bytes, &bad_name_sig, &pubkey_b64),
        Err(AppError::Validation(_))
    ));
}

#[tokio::test]
async fn sha256_verification_pre_update_safety_backup_and_windows_linux_staging() {
    let temp = TestDir::new("staging");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("dirs");
    let (conn, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool = conn.require_pool().expect("pool");

    let win_bytes = b"valid-windows-exe-payload";
    let linux_bytes = b"valid-linux-appimage-payload";
    let win_sha = updater::compute_sha256_hex(win_bytes);
    let linux_sha = updater::compute_sha256_hex(linux_bytes);
    let manifest = make_multi_platform_manifest(&win_sha, &linux_sha);

    // 1. Corrupted download fails SHA-256 verification before staging
    assert!(matches!(
        updater::stage_verified_update_bytes(
            &pool,
            &paths,
            &manifest,
            "windows-x86_64",
            b"truncated"
        )
        .await,
        Err(AppError::Validation(_))
    ));

    // 2. Stage Windows .exe -> creates safety backup + apply-update.ps1
    let staged_win =
        updater::stage_verified_update_bytes(&pool, &paths, &manifest, "windows-x86_64", win_bytes)
            .await
            .expect("stage windows update");
    assert_eq!(staged_win.safety_backup.backup_type, BackupType::Safety);
    assert!(staged_win.staged_artifact_path.ends_with(".exe"));
    assert!(staged_win.apply_script_path.ends_with("apply-update.ps1"));
    assert!(Path::new(&staged_win.apply_script_path).is_file());

    // 3. Stage Linux .AppImage -> creates safety backup + executable apply-update.sh
    let staged_linux =
        updater::stage_verified_update_bytes(&pool, &paths, &manifest, "linux-x86_64", linux_bytes)
            .await
            .expect("stage linux update");
    assert_eq!(staged_linux.safety_backup.backup_type, BackupType::Safety);
    assert!(staged_linux.staged_artifact_path.ends_with(".AppImage"));
    assert!(staged_linux.apply_script_path.ends_with("apply-update.sh"));
    assert!(Path::new(&staged_linux.apply_script_path).is_file());

    // 4. Unconfigured signing key reports clean non-fatal status on update check
    let check_unconfigured = updater::check_for_updates("0.1.0", "linux-x86_64", None).await;
    if updater::configured_public_key_b64().is_none() {
        assert_eq!(
            check_unconfigured.status,
            UpdateCheckStatus::SigningKeyUnconfigured
        );
    }
}

#[test]
fn portable_executable_replacement_and_rollback_preserves_user_data() {
    let temp = TestDir::new("rollback");
    let app_dir = temp.path().join("MyScheduleApp");
    let data_dir = app_dir.join("data");
    fs::create_dir_all(&data_dir).expect("create dirs");

    let appimage_path = app_dir.join("MyScheduleApp.AppImage");
    let user_db = data_dir.join("database.sqlite");
    fs::write(&appimage_path, b"OLD_APPIMAGE_V1").expect("write v1");
    fs::write(&user_db, b"USER_DATA_MUST_SURVIVE").expect("write user data");

    // 1. Failed post-swap check rolls back to OLD_APPIMAGE_V1 automatically
    let failed = updater::replace_executable_with_rollback(&appimage_path, b"BROKEN_V2", |_path| {
        Err(AppError::Internal(
            "Simulated startup/migration crash".into(),
        ))
    });
    assert!(failed.is_err());
    assert_eq!(fs::read(&appimage_path).unwrap(), b"OLD_APPIMAGE_V1");
    assert_eq!(fs::read(&user_db).unwrap(), b"USER_DATA_MUST_SURVIVE");

    // 2. Successful post-swap check promotes GOOD_V2 and cleans up rollback backup while preserving data/
    updater::replace_executable_with_rollback(&appimage_path, b"GOOD_V2", |path| {
        assert_eq!(fs::read(path).unwrap(), b"GOOD_V2");
        Ok(())
    })
    .expect("swap succeeds");
    assert_eq!(fs::read(&appimage_path).unwrap(), b"GOOD_V2");
    assert_eq!(fs::read(&user_db).unwrap(), b"USER_DATA_MUST_SURVIVE");
    assert!(!app_dir.join("MyScheduleApp.AppImage.rollback_bak").exists());
}
