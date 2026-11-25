use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::backup::{self, BackupMetadata, BackupType};
use crate::errors::{AppError, AppResult};
use crate::platform::{self, AppPaths};

/// Configured GitHub repository for My Schedule App releases.
pub const GITHUB_REPO: &str = "MichaelX17/My-Schedule-App";
pub const GITHUB_RELEASES_URL: &str = "https://github.com/MichaelX17/My-Schedule-App/releases";
pub const MANIFEST_URL: &str =
    "https://github.com/MichaelX17/My-Schedule-App/releases/latest/download/latest.json";
pub const SIGNATURE_URL: &str =
    "https://github.com/MichaelX17/My-Schedule-App/releases/latest/download/latest.json.sig";

/// Returns the platform key for the current build target (e.g., `windows-x86_64`, `linux-x86_64`).
pub fn current_platform_key() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Compile-time or runtime-injected base64-encoded 32-byte Ed25519 release public key.
/// Private keys must NEVER be committed to source control; inject only the public key
/// via `MY_SCHEDULE_UPDATE_PUBKEY` during release builds.
pub fn configured_public_key_b64() -> Option<String> {
    std::env::var("MY_SCHEDULE_UPDATE_PUBKEY")
        .ok()
        .or_else(|| option_env!("MY_SCHEDULE_UPDATE_PUBKEY").map(str::to_string))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Static updater metadata shown in Settings → Updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdaterInfo {
    pub current_version: String,
    pub platform_key: String,
    pub github_repo: String,
    pub releases_url: String,
    pub signing_key_configured: bool,
}

/// Per-platform downloadable release artifact inside `latest.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformArtifact {
    pub artifact_filename: String,
    pub download_url: String,
    pub sha256: String,
}

/// Signed multi-platform release manifest (`latest.json`) published on GitHub Releases.
/// Supports both Windows (`.exe` / `.zip`) and Linux (`.AppImage`) artifacts in one release.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateManifest {
    pub version: String,
    pub pub_date: String,
    pub notes: String,
    pub target_schema_version: Option<i64>,
    pub platforms: BTreeMap<String, PlatformArtifact>,
}

/// Outcome of comparing `current` against a candidate release version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    UpdateAvailable,
    UpToDate,
    DowngradeRejected,
}

/// Non-fatal status returned by `check_for_updates`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateCheckStatus {
    UpdateAvailable,
    UpToDate,
    PlatformUnsupported,
    Offline,
    RateLimited,
    NoReleasePublished,
    SigningKeyUnconfigured,
    VerificationFailed,
    Error,
}

/// Structured result of checking GitHub Releases for a newer signed version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateCheckResult {
    pub status: UpdateCheckStatus,
    pub current_version: String,
    pub platform_key: String,
    pub latest_version: Option<String>,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
    pub artifact_filename: Option<String>,
    pub message: String,
}

/// Result returned after verifying signature + SHA-256, creating a pre-update safety backup,
/// and staging the platform artifact + replacement/rollback helper script.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StagedUpdateResult {
    pub version: String,
    pub platform_key: String,
    pub staged_artifact_path: String,
    pub apply_script_path: String,
    pub sha256: String,
    pub safety_backup: BackupMetadata,
    pub message: String,
}

pub fn get_updater_info() -> UpdaterInfo {
    UpdaterInfo {
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        platform_key: current_platform_key(),
        github_repo: GITHUB_REPO.to_string(),
        releases_url: GITHUB_RELEASES_URL.to_string(),
        signing_key_configured: configured_public_key_b64().is_some(),
    }
}

/// Compares semantic versions strictly; never allows equal or older versions as updates.
pub fn compare_versions(current_raw: &str, candidate_raw: &str) -> AppResult<VersionComparison> {
    let current = Version::parse(current_raw.trim().trim_start_matches('v'))
        .map_err(|_| AppError::Validation("Invalid current application version.".into()))?;
    let candidate = Version::parse(candidate_raw.trim().trim_start_matches('v'))
        .map_err(|_| AppError::Validation("Invalid release manifest version.".into()))?;

    if candidate > current {
        Ok(VersionComparison::UpdateAvailable)
    } else if candidate == current {
        Ok(VersionComparison::UpToDate)
    } else {
        Ok(VersionComparison::DowngradeRejected)
    }
}

/// Computes lowercase hex SHA-256 digest of `bytes`.
pub fn compute_sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Verifies that `bytes` match the expected 64-char hex SHA-256 digest.
pub fn verify_artifact_sha256(bytes: &[u8], expected_hex: &str) -> AppResult<()> {
    let normalized_expected = expected_hex.trim().to_ascii_lowercase();
    if normalized_expected.len() != 64
        || !normalized_expected.chars().all(|c| c.is_ascii_hexdigit())
    {
        return Err(AppError::Validation(
            "Release manifest contains an invalid SHA-256 digest.".into(),
        ));
    }

    let actual = compute_sha256_hex(bytes);
    if actual != normalized_expected {
        return Err(AppError::Validation(
            "Downloaded update failed SHA-256 integrity verification and was discarded.".into(),
        ));
    }
    Ok(())
}

/// Verifies an Ed25519 signature (`signature_b64`, 64 bytes) over the exact `manifest_bytes`
/// using the trusted public key (`pubkey_b64`, 32 bytes).
pub fn verify_manifest_signature(
    manifest_bytes: &[u8],
    signature_b64: &str,
    pubkey_b64: &str,
) -> AppResult<()> {
    let pk_bytes = BASE64
        .decode(pubkey_b64.trim())
        .map_err(|_| AppError::Validation("Configured update public key is invalid.".into()))?;
    let pk_array: [u8; 32] = pk_bytes.try_into().map_err(|_| {
        AppError::Validation("Configured update public key has invalid length.".into())
    })?;
    let verifying_key = VerifyingKey::from_bytes(&pk_array)
        .map_err(|_| AppError::Validation("Configured update public key is malformed.".into()))?;

    let sig_bytes = BASE64
        .decode(signature_b64.trim())
        .map_err(|_| AppError::Validation("Update signature is not valid base64.".into()))?;
    let sig_array: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| AppError::Validation("Update signature has invalid length.".into()))?;
    let signature = Signature::from_bytes(&sig_array);

    verifying_key
        .verify(manifest_bytes, &signature)
        .map_err(|_| {
            AppError::Validation(
                "Update manifest signature verification failed. The update was rejected.".into(),
            )
        })
}

fn validate_platform_artifact(platform_key: &str, artifact: &PlatformArtifact) -> AppResult<()> {
    let url = artifact.download_url.trim();
    let allowed_prefix = url.starts_with(&format!("https://github.com/{GITHUB_REPO}/releases/"))
        || url.starts_with("https://objects.githubusercontent.com/");
    if !allowed_prefix {
        return Err(AppError::Validation(
            "Release manifest download URL must point to HTTPS GitHub Releases for this repository."
                .into(),
        ));
    }

    let name = artifact.artifact_filename.trim();
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(AppError::Validation(
            "Release manifest artifact filename is invalid.".into(),
        ));
    }

    let lower = name.to_ascii_lowercase();
    let valid_ext = if platform_key.starts_with("windows-") {
        lower.ends_with(".exe") || lower.ends_with(".zip")
    } else if platform_key.starts_with("linux-") {
        lower.ends_with(".appimage")
    } else {
        lower.ends_with(".exe") || lower.ends_with(".zip") || lower.ends_with(".appimage")
    };
    if !valid_ext {
        return Err(AppError::Validation(format!(
            "Unsupported artifact extension for platform {platform_key}: {name}"
        )));
    }

    let sha = artifact.sha256.trim();
    if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::Validation(
            "Release manifest SHA-256 digest is invalid.".into(),
        ));
    }

    Ok(())
}

/// Verifies the Ed25519 signature FIRST, then parses and validates all manifest fields
/// and every platform artifact entry.
pub fn parse_and_verify_manifest(
    manifest_bytes: &[u8],
    signature_b64: &str,
    pubkey_b64: &str,
) -> AppResult<UpdateManifest> {
    verify_manifest_signature(manifest_bytes, signature_b64, pubkey_b64)?;

    let manifest: UpdateManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|_| AppError::Validation("Release manifest JSON is malformed.".into()))?;

    Version::parse(manifest.version.trim().trim_start_matches('v')).map_err(|_| {
        AppError::Validation("Release manifest version is not valid semver.".into())
    })?;

    if manifest.platforms.is_empty() {
        return Err(AppError::Validation(
            "Release manifest contains no platform artifacts.".into(),
        ));
    }

    for (platform_key, artifact) in &manifest.platforms {
        validate_platform_artifact(platform_key, artifact)?;
    }

    Ok(manifest)
}

/// Selects the verified artifact for `platform_key` from a validated manifest.
pub fn select_artifact_for_platform<'a>(
    manifest: &'a UpdateManifest,
    platform_key: &str,
) -> Option<&'a PlatformArtifact> {
    manifest.platforms.get(platform_key)
}

fn build_http_client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent(format!("MyScheduleApp/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to initialize HTTPS client: {e}")))
}

/// Checks GitHub Releases for a signed multi-platform update manifest.
pub async fn check_for_updates(
    current_version: &str,
    platform_key: &str,
    pubkey_override: Option<&str>,
) -> UpdateCheckResult {
    let pubkey = pubkey_override
        .map(str::to_string)
        .or_else(configured_public_key_b64);

    let Some(pubkey_b64) = pubkey else {
        return UpdateCheckResult {
            status: UpdateCheckStatus::SigningKeyUnconfigured,
            current_version: current_version.to_string(),
            platform_key: platform_key.to_string(),
            latest_version: None,
            notes: None,
            pub_date: None,
            artifact_filename: None,
            message: format!(
                "Update verification key is not configured in this build. You can check releases manually at {GITHUB_RELEASES_URL}."
            ),
        };
    };

    let client = match build_http_client() {
        Ok(c) => c,
        Err(err) => {
            return UpdateCheckResult {
                status: UpdateCheckStatus::Error,
                current_version: current_version.to_string(),
                platform_key: platform_key.to_string(),
                latest_version: None,
                notes: None,
                pub_date: None,
                artifact_filename: None,
                message: err.to_string(),
            };
        }
    };

    let manifest_resp = match client.get(MANIFEST_URL).send().await {
        Ok(resp) => resp,
        Err(err) => {
            tracing::info!(error = %err, "Update check skipped (offline or network unreachable)");
            return UpdateCheckResult {
                status: UpdateCheckStatus::Offline,
                current_version: current_version.to_string(),
                platform_key: platform_key.to_string(),
                latest_version: None,
                notes: None,
                pub_date: None,
                artifact_filename: None,
                message: "Unable to reach GitHub Releases (you appear to be offline). Core features continue working normally.".into(),
            };
        }
    };

    let status_code = manifest_resp.status();
    if status_code.as_u16() == 404 {
        return UpdateCheckResult {
            status: UpdateCheckStatus::NoReleasePublished,
            current_version: current_version.to_string(),
            platform_key: platform_key.to_string(),
            latest_version: None,
            notes: None,
            pub_date: None,
            artifact_filename: None,
            message: "No signed release manifest has been published on GitHub Releases yet.".into(),
        };
    }
    if status_code.as_u16() == 403 || status_code.as_u16() == 429 {
        return UpdateCheckResult {
            status: UpdateCheckStatus::RateLimited,
            current_version: current_version.to_string(),
            platform_key: platform_key.to_string(),
            latest_version: None,
            notes: None,
            pub_date: None,
            artifact_filename: None,
            message: "GitHub rate limit reached. Please try checking for updates again later."
                .into(),
        };
    }
    if !status_code.is_success() {
        return UpdateCheckResult {
            status: UpdateCheckStatus::Error,
            current_version: current_version.to_string(),
            platform_key: platform_key.to_string(),
            latest_version: None,
            notes: None,
            pub_date: None,
            artifact_filename: None,
            message: format!("GitHub Releases returned HTTP {status_code}."),
        };
    }

    let manifest_bytes = match manifest_resp.bytes().await {
        Ok(b) => b,
        Err(_) => {
            return UpdateCheckResult {
                status: UpdateCheckStatus::Offline,
                current_version: current_version.to_string(),
                platform_key: platform_key.to_string(),
                latest_version: None,
                notes: None,
                pub_date: None,
                artifact_filename: None,
                message: "Download of release manifest was interrupted.".into(),
            };
        }
    };

    let sig_resp = match client.get(SIGNATURE_URL).send().await {
        Ok(resp) if resp.status().is_success() => resp,
        Ok(resp) => {
            return UpdateCheckResult {
                status: UpdateCheckStatus::VerificationFailed,
                current_version: current_version.to_string(),
                platform_key: platform_key.to_string(),
                latest_version: None,
                notes: None,
                pub_date: None,
                artifact_filename: None,
                message: format!(
                    "Release signature file (latest.json.sig) could not be fetched (HTTP {}). Unsigned updates are rejected.",
                    resp.status()
                ),
            };
        }
        Err(_) => {
            return UpdateCheckResult {
                status: UpdateCheckStatus::Offline,
                current_version: current_version.to_string(),
                platform_key: platform_key.to_string(),
                latest_version: None,
                notes: None,
                pub_date: None,
                artifact_filename: None,
                message: "Unable to fetch release signature (network interrupted).".into(),
            };
        }
    };

    let sig_text = match sig_resp.text().await {
        Ok(t) => t,
        Err(_) => {
            return UpdateCheckResult {
                status: UpdateCheckStatus::VerificationFailed,
                current_version: current_version.to_string(),
                platform_key: platform_key.to_string(),
                latest_version: None,
                notes: None,
                pub_date: None,
                artifact_filename: None,
                message: "Unable to read release signature.".into(),
            };
        }
    };

    let manifest = match parse_and_verify_manifest(&manifest_bytes, &sig_text, &pubkey_b64) {
        Ok(m) => m,
        Err(err) => {
            tracing::warn!(error = %err, "Update manifest verification failed");
            return UpdateCheckResult {
                status: UpdateCheckStatus::VerificationFailed,
                current_version: current_version.to_string(),
                platform_key: platform_key.to_string(),
                latest_version: None,
                notes: None,
                pub_date: None,
                artifact_filename: None,
                message: err.to_string(),
            };
        }
    };

    match compare_versions(current_version, &manifest.version) {
        Ok(VersionComparison::UpdateAvailable) => {
            let Some(artifact) = select_artifact_for_platform(&manifest, platform_key) else {
                return UpdateCheckResult {
                    status: UpdateCheckStatus::PlatformUnsupported,
                    current_version: current_version.to_string(),
                    platform_key: platform_key.to_string(),
                    latest_version: Some(manifest.version.clone()),
                    notes: Some(manifest.notes),
                    pub_date: Some(manifest.pub_date),
                    artifact_filename: None,
                    message: format!(
                        "Version {} was released, but no artifact for {platform_key} is listed yet.",
                        manifest.version
                    ),
                };
            };

            let artifact_filename = artifact.artifact_filename.clone();
            let message = format!(
                "Version {} is available for {platform_key} ({artifact_filename}).",
                manifest.version
            );

            UpdateCheckResult {
                status: UpdateCheckStatus::UpdateAvailable,
                current_version: current_version.to_string(),
                platform_key: platform_key.to_string(),
                latest_version: Some(manifest.version),
                notes: Some(manifest.notes),
                pub_date: Some(manifest.pub_date),
                artifact_filename: Some(artifact_filename),
                message,
            }
        }
        Ok(VersionComparison::UpToDate) | Ok(VersionComparison::DowngradeRejected) => {
            UpdateCheckResult {
                status: UpdateCheckStatus::UpToDate,
                current_version: current_version.to_string(),
                platform_key: platform_key.to_string(),
                latest_version: Some(manifest.version),
                notes: None,
                pub_date: Some(manifest.pub_date),
                artifact_filename: None,
                message: format!("You are running the latest version (v{current_version})."),
            }
        }
        Err(err) => UpdateCheckResult {
            status: UpdateCheckStatus::VerificationFailed,
            current_version: current_version.to_string(),
            platform_key: platform_key.to_string(),
            latest_version: None,
            notes: None,
            pub_date: None,
            artifact_filename: None,
            message: err.to_string(),
        },
    }
}

/// Generates the Windows portable apply/rollback PowerShell helper script (supports both `.exe` and `.zip`).
fn build_windows_apply_script(artifact_filename: &str) -> String {
    let is_zip = artifact_filename.to_ascii_lowercase().ends_with(".zip");
    let prepare_new_exe = if is_zip {
        format!(
            r#"$ExtractDir = Join-Path $PSScriptRoot "extracted"
if (Test-Path $ExtractDir) {{ Remove-Item -Recurse -Force $ExtractDir }}
Expand-Archive -Path (Join-Path $PSScriptRoot "{artifact_filename}") -DestinationPath $ExtractDir -Force
$NewExe = Join-Path $ExtractDir "MyScheduleApp.exe""#
        )
    } else {
        format!(r#"$NewExe = Join-Path $PSScriptRoot "{artifact_filename}""#)
    };

    format!(
        r#"# My Schedule App — Windows Portable Update Apply & Rollback Helper
# Close My Schedule App before running this script. Never touches data\.
param(
    [Parameter(Mandatory=$true)][string]$AppDir,
    [int]$WaitPid = 0
)
$ErrorActionPreference = "Stop"
if ($WaitPid -gt 0) {{
    try {{ Wait-Process -Id $WaitPid -Timeout 30 -ErrorAction SilentlyContinue }} catch {{}}
}}
{prepare_new_exe}
if (-not (Test-Path $NewExe)) {{ throw "Staged executable not found at $NewExe" }}
$TargetExe = Join-Path $AppDir "MyScheduleApp.exe"
$RollbackExe = Join-Path $AppDir "MyScheduleApp.exe.rollback_bak"
if (Test-Path $RollbackExe) {{ Remove-Item -Force $RollbackExe }}
if (Test-Path $TargetExe) {{ Move-Item -Path $TargetExe -Destination $RollbackExe -Force }}
try {{
    Copy-Item -Path $NewExe -Destination $TargetExe -Force
    Start-Process -FilePath $TargetExe -WorkingDirectory $AppDir
}} catch {{
    if (Test-Path $RollbackExe) {{ Move-Item -Path $RollbackExe -Destination $TargetExe -Force }}
    throw
}}
"#
    )
}

/// Generates the Linux AppImage apply/rollback shell helper script (`apply-update.sh`).
fn build_linux_apply_script(artifact_filename: &str) -> String {
    format!(
        r#"#!/usr/bin/env bash
# My Schedule App — Linux AppImage Update Apply & Rollback Helper
# Usage: ./apply-update.sh /path/to/MyScheduleApp.AppImage [wait_pid]
# Never touches the adjacent data/ directory.
set -euo pipefail

TARGET_APPIMAGE="${{1:-${{APPIMAGE:-}}}}"
WAIT_PID="${{2:-0}}"

if [[ -z "$TARGET_APPIMAGE" ]]; then
  echo "Usage: $0 /path/to/MyScheduleApp.AppImage [wait_pid]" >&2
  exit 1
fi

if [[ "$WAIT_PID" -gt 0 ]]; then
  for _ in $(seq 1 30); do
    kill -0 "$WAIT_PID" 2>/dev/null || break
    sleep 1
  done
fi

SCRIPT_DIR="$(cd "$(dirname "${{BASH_SOURCE[0]}}")" && pwd)"
NEW_APPIMAGE="$SCRIPT_DIR/{artifact_filename}"
ROLLBACK_APPIMAGE="${{TARGET_APPIMAGE}}.rollback_bak"

if [[ ! -f "$NEW_APPIMAGE" ]]; then
  echo "Staged AppImage not found at $NEW_APPIMAGE" >&2
  exit 1
fi

rm -f "$ROLLBACK_APPIMAGE"
if [[ -f "$TARGET_APPIMAGE" ]]; then
  mv -f "$TARGET_APPIMAGE" "$ROLLBACK_APPIMAGE"
fi

if cp -f "$NEW_APPIMAGE" "$TARGET_APPIMAGE" && chmod +x "$TARGET_APPIMAGE"; then
  nohup "$TARGET_APPIMAGE" >/dev/null 2>&1 &
  rm -f "$ROLLBACK_APPIMAGE"
else
  if [[ -f "$ROLLBACK_APPIMAGE" ]]; then
    mv -f "$ROLLBACK_APPIMAGE" "$TARGET_APPIMAGE"
  fi
  echo "Update failed; restored previous AppImage." >&2
  exit 1
fi
"#
    )
}

/// Verifies `artifact_bytes` against the signed platform artifact's SHA-256, creates a verified
/// pre-update SQLite safety backup FIRST, stages the artifact atomically via `.part` -> final
/// file, and writes the OS-appropriate apply/rollback helper (`apply-update.ps1` on Windows,
/// `apply-update.sh` on Linux).
pub async fn stage_verified_update_bytes(
    pool: &SqlitePool,
    paths: &AppPaths,
    manifest: &UpdateManifest,
    platform_key: &str,
    artifact_bytes: &[u8],
) -> AppResult<StagedUpdateResult> {
    let artifact = select_artifact_for_platform(manifest, platform_key).ok_or_else(|| {
        AppError::Validation(format!(
            "No update artifact available for platform {platform_key}."
        ))
    })?;

    // 1. Verify artifact SHA-256 against the Ed25519-signed manifest digest BEFORE touching anything.
    verify_artifact_sha256(artifact_bytes, &artifact.sha256)?;

    // 2. Create and validate a pre-update SQLite safety backup before any update/migration.
    let safety_backup = backup::create_backup(
        pool,
        paths,
        BackupType::Safety,
        Some(format!(
            "Pre-update safety backup before v{} ({platform_key})",
            manifest.version
        )),
        None,
    )
    .await?;

    // 3. Stage atomically inside `data/updates/staged/v<version>/`.
    let version_folder = format!("v{}", manifest.version.trim().trim_start_matches('v'));
    let staged_root = platform::safe_join_child(&paths.updates_dir, "staged")?;
    let version_dir = platform::safe_join_child(&staged_root, &version_folder)?;
    fs::create_dir_all(&version_dir)?;

    let filename = artifact.artifact_filename.trim();
    let final_artifact_path = platform::safe_join_child(&version_dir, filename)?;
    let part_name = format!(".{filename}.{}.part", Uuid::new_v4());
    let part_path = platform::safe_join_child(&version_dir, &part_name)?;

    let write_res = (|| -> std::io::Result<()> {
        let mut f = File::create(&part_path)?;
        f.write_all(artifact_bytes)?;
        f.sync_all()?;
        Ok(())
    })();

    if let Err(err) = write_res {
        let _ = fs::remove_file(&part_path);
        return Err(AppError::Io(err));
    }

    if let Err(err) = fs::rename(&part_path, &final_artifact_path) {
        let _ = fs::remove_file(&part_path);
        return Err(AppError::Io(err));
    }

    let (script_name, script_body) = if platform_key.starts_with("windows-") {
        ("apply-update.ps1", build_windows_apply_script(filename))
    } else {
        ("apply-update.sh", build_linux_apply_script(filename))
    };

    let apply_script_path = platform::safe_join_child(&version_dir, script_name)?;
    fs::write(&apply_script_path, script_body)?;

    #[cfg(unix)]
    if !platform_key.starts_with("windows-") {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&apply_script_path, fs::Permissions::from_mode(0o755));
        let _ = fs::set_permissions(&final_artifact_path, fs::Permissions::from_mode(0o755));
    }

    tracing::info!(
        version = %manifest.version,
        platform = %platform_key,
        safety_backup = %safety_backup.filename,
        "Verified update staged with pre-update safety backup"
    );

    Ok(StagedUpdateResult {
        version: manifest.version.clone(),
        platform_key: platform_key.to_string(),
        staged_artifact_path: final_artifact_path.to_string_lossy().into_owned(),
        apply_script_path: apply_script_path.to_string_lossy().into_owned(),
        sha256: artifact.sha256.to_ascii_lowercase(),
        safety_backup,
        message: format!(
            "Version {} ({filename}) verified and staged. Pre-update safety backup saved. Close the app and run {script_name} to complete replacement safely.",
            manifest.version
        ),
    })
}

/// Full user-consented download, signature verification, SHA-256 verification, safety backup,
/// and staging flow against GitHub Releases for the current platform.
pub async fn download_and_stage_update(
    pool: &SqlitePool,
    paths: &AppPaths,
    current_version: &str,
    platform_key: &str,
    pubkey_override: Option<&str>,
) -> AppResult<StagedUpdateResult> {
    let pubkey_b64 = pubkey_override
        .map(str::to_string)
        .or_else(configured_public_key_b64)
        .ok_or_else(|| {
            AppError::Validation(
                "Update verification public key is not configured in this build.".into(),
            )
        })?;

    let client = build_http_client()?;
    let manifest_bytes = client
        .get(MANIFEST_URL)
        .send()
        .await
        .map_err(|_| AppError::Validation("Unable to reach GitHub Releases (offline).".into()))?
        .error_for_status()
        .map_err(|e| AppError::Validation(format!("Failed to fetch release manifest: {e}")))?
        .bytes()
        .await
        .map_err(|_| AppError::Validation("Release manifest download was interrupted.".into()))?;

    let sig_text = client
        .get(SIGNATURE_URL)
        .send()
        .await
        .map_err(|_| AppError::Validation("Unable to fetch release signature (offline).".into()))?
        .error_for_status()
        .map_err(|e| AppError::Validation(format!("Failed to fetch release signature: {e}")))?
        .text()
        .await
        .map_err(|_| AppError::Validation("Release signature download was interrupted.".into()))?;

    let manifest = parse_and_verify_manifest(&manifest_bytes, &sig_text, &pubkey_b64)?;
    if compare_versions(current_version, &manifest.version)? != VersionComparison::UpdateAvailable {
        return Err(AppError::Validation(
            "No newer version is available to download.".into(),
        ));
    }

    let artifact = select_artifact_for_platform(&manifest, platform_key).ok_or_else(|| {
        AppError::Validation(format!(
            "No update artifact is published for platform {platform_key}."
        ))
    })?;

    let artifact_bytes = client
        .get(&artifact.download_url)
        .send()
        .await
        .map_err(|_| AppError::Validation("Artifact download failed (network error).".into()))?
        .error_for_status()
        .map_err(|e| AppError::Validation(format!("Failed to download update artifact: {e}")))?
        .bytes()
        .await
        .map_err(|_| AppError::Validation("Artifact download was interrupted.".into()))?;

    stage_verified_update_bytes(pool, paths, &manifest, platform_key, &artifact_bytes).await
}

/// Testable atomic binary swap with automatic rollback on post-swap verification failure.
/// Works identically for Windows `.exe` and Linux `.AppImage`; never touches sibling `data/`.
pub fn replace_executable_with_rollback<F>(
    target_exe: &Path,
    new_exe_bytes: &[u8],
    verify_new_exe: F,
) -> AppResult<()>
where
    F: FnOnce(&Path) -> AppResult<()>,
{
    let parent = target_exe
        .parent()
        .ok_or_else(|| AppError::InvalidPath("Invalid executable path.".into()))?;
    let exe_name = target_exe
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::InvalidPath("Invalid executable filename.".into()))?;

    let staging_exe = platform::safe_join_child(parent, &format!(".{exe_name}.new"))?;
    let rollback_exe = platform::safe_join_child(parent, &format!("{exe_name}.rollback_bak"))?;

    fs::write(&staging_exe, new_exe_bytes)?;

    if target_exe.exists() {
        fs::rename(target_exe, &rollback_exe)?;
    }

    if let Err(err) = fs::rename(&staging_exe, target_exe) {
        let _ = fs::remove_file(&staging_exe);
        if rollback_exe.exists() {
            let _ = fs::rename(&rollback_exe, target_exe);
        }
        return Err(AppError::Io(err));
    }

    match verify_new_exe(target_exe) {
        Ok(()) => {
            let _ = fs::remove_file(&rollback_exe);
            Ok(())
        }
        Err(verify_err) => {
            let _ = fs::remove_file(target_exe);
            if rollback_exe.exists() {
                let _ = fs::rename(&rollback_exe, target_exe);
            }
            Err(verify_err)
        }
    }
}
