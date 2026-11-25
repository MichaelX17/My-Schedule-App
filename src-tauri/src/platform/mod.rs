use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::errors::{AppError, AppResult};

static PROBE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Indicates whether the application is storing data alongside the executable
/// (portable mode) or in the OS per-user application data directory (fallback).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageMode {
    Portable,
    Fallback,
}

/// Resolved filesystem layout for application data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub root_dir: PathBuf,
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub backups_dir: PathBuf,
    pub automatic_backups_dir: PathBuf,
    pub manual_backups_dir: PathBuf,
    pub exports_dir: PathBuf,
    pub logos_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub updates_dir: PathBuf,
}

impl AppPaths {
    /// Builds the predictable directory layout rooted at `base_dir`:
    ///
    /// ```text
    /// <base_dir>/
    ///   data/
    ///     database.sqlite
    ///     backups/
    ///       automatic/
    ///       manual/
    ///     exports/
    ///     logos/
    ///     logs/
    ///     updates/
    /// ```
    pub fn from_root(base_dir: &Path) -> Self {
        let data_dir = base_dir.join("data");
        let backups_dir = data_dir.join("backups");
        Self {
            root_dir: base_dir.to_path_buf(),
            database_path: data_dir.join("database.sqlite"),
            automatic_backups_dir: backups_dir.join("automatic"),
            manual_backups_dir: backups_dir.join("manual"),
            backups_dir,
            exports_dir: data_dir.join("exports"),
            logos_dir: data_dir.join("logos"),
            logs_dir: data_dir.join("logs"),
            updates_dir: data_dir.join("updates"),
            data_dir,
        }
    }

    /// Creates all required application data subdirectories idempotently.
    pub fn ensure_directories(&self) -> AppResult<()> {
        let dirs = [
            &self.data_dir,
            &self.backups_dir,
            &self.automatic_backups_dir,
            &self.manual_backups_dir,
            &self.exports_dir,
            &self.logos_dir,
            &self.logs_dir,
            &self.updates_dir,
        ];

        for dir in dirs {
            fs::create_dir_all(dir).map_err(|err| {
                AppError::StorageUnavailable(format!(
                    "failed to create directory {}: {err}",
                    dir.display()
                ))
            })?;
        }

        Ok(())
    }
}

/// Result of storage path resolution, including active mode and optional fallback reason.
#[derive(Debug, Clone)]
pub struct ResolvedStorage {
    pub mode: StorageMode,
    pub paths: AppPaths,
    pub fallback_reason: Option<String>,
}

/// Safely joins a single normal filename/segment onto `base_dir`, rejecting
/// path traversal (`..`, `.`), separators, drive prefixes, and empty names.
pub fn safe_join_child(base_dir: &Path, child_name: &str) -> AppResult<PathBuf> {
    if child_name.is_empty() || child_name.contains('/') || child_name.contains('\\') {
        return Err(AppError::InvalidPath(
            "child name must be a single non-empty path segment".into(),
        ));
    }

    let candidate = Path::new(child_name);
    let mut components = candidate.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(segment)), None) => Ok(base_dir.join(segment)),
        _ => Err(AppError::InvalidPath(
            "child name must not contain traversal or root components".into(),
        )),
    }
}

/// Resolves application storage paths using the current executable directory
/// (portable mode) with fallback to the OS per-user app data directory.
///
/// Refuses portable mode if the executable is running inside the OS temporary
/// directory (for example, launched directly from an extracted ZIP preview) so
/// user data is never placed in a volatile temp folder.
pub fn resolve_runtime_storage(
    fallback_app_data_dir: Option<PathBuf>,
) -> AppResult<ResolvedStorage> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    let temp_dir = std::env::temp_dir();

    resolve_storage_paths_with_temp_guard(
        exe_dir.as_deref(),
        fallback_app_data_dir.as_deref(),
        Some(temp_dir.as_path()),
    )
}

/// Checks whether `candidate` is located inside `temp_root` (comparing canonicalized
/// paths when available, falling back to normalized prefix comparison).
fn is_inside_temp_dir(candidate: &Path, temp_root: &Path) -> bool {
    let norm_candidate = fs::canonicalize(candidate).unwrap_or_else(|_| candidate.to_path_buf());
    let norm_temp = fs::canonicalize(temp_root).unwrap_or_else(|_| temp_root.to_path_buf());
    norm_candidate.starts_with(&norm_temp)
}

/// Pure, testable storage-path resolver without a temporary-directory guard
/// (suitable for tests that run inside temporary test fixtures).
pub fn resolve_storage_paths(
    exe_dir: Option<&Path>,
    fallback_app_data_dir: Option<&Path>,
) -> AppResult<ResolvedStorage> {
    resolve_storage_paths_with_temp_guard(exe_dir, fallback_app_data_dir, None)
}

/// Pure, testable storage-path resolver with optional temporary-directory protection.
///
/// 1. If `temp_dir_guard` is provided and `exe_dir` resides inside it, refuses portable
///    mode and falls back to `fallback_app_data_dir` so user data is never lost when
///    temp folders are cleaned.
/// 2. Otherwise attempts to use `<exe_dir>/data` if `exe_dir` is genuinely writable
///    (verified via create + write + remove probe).
/// 3. Falls back to `<fallback_app_data_dir>/data`, verifying that it is writable.
pub fn resolve_storage_paths_with_temp_guard(
    exe_dir: Option<&Path>,
    fallback_app_data_dir: Option<&Path>,
    temp_dir_guard: Option<&Path>,
) -> AppResult<ResolvedStorage> {
    let portable_failure_reason = if let Some(dir) = exe_dir {
        if let Some(temp_root) = temp_dir_guard {
            if is_inside_temp_dir(dir, temp_root) {
                tracing::warn!(
                    "Executable appears to be inside a temporary directory; refusing portable mode to protect user data"
                );
                Some(
                    "Application is running from a temporary folder; using per-user application data directory to keep your data safe."
                        .to_string(),
                )
            } else {
                let candidate_paths = AppPaths::from_root(dir);
                match probe_directory_writable(&candidate_paths.data_dir) {
                    Ok(()) => {
                        return Ok(ResolvedStorage {
                            mode: StorageMode::Portable,
                            paths: candidate_paths,
                            fallback_reason: None,
                        });
                    }
                    Err(reason) => {
                        tracing::warn!(
                            reason = %reason,
                            "Portable adjacent data directory is not writable; attempting per-user fallback"
                        );
                        Some(
                            "Adjacent application directory is not writable; using per-user application data directory."
                                .to_string(),
                        )
                    }
                }
            }
        } else {
            let candidate_paths = AppPaths::from_root(dir);
            match probe_directory_writable(&candidate_paths.data_dir) {
                Ok(()) => {
                    return Ok(ResolvedStorage {
                        mode: StorageMode::Portable,
                        paths: candidate_paths,
                        fallback_reason: None,
                    });
                }
                Err(reason) => {
                    tracing::warn!(
                        reason = %reason,
                        "Portable adjacent data directory is not writable; attempting per-user fallback"
                    );
                    Some(
                        "Adjacent application directory is not writable; using per-user application data directory."
                            .to_string(),
                    )
                }
            }
        }
    } else {
        Some(
            "Executable directory could not be determined; using per-user application data directory."
                .to_string(),
        )
    };

    let fallback_root = fallback_app_data_dir.ok_or_else(|| {
        AppError::StorageUnavailable(
            "Neither adjacent portable directory nor per-user fallback directory is available"
                .into(),
        )
    })?;

    let fallback_paths = AppPaths::from_root(fallback_root);
    probe_directory_writable(&fallback_paths.data_dir).map_err(|reason| {
        AppError::StorageUnavailable(format!(
            "Fallback application data directory is not writable: {reason}"
        ))
    })?;

    Ok(ResolvedStorage {
        mode: StorageMode::Fallback,
        paths: fallback_paths,
        fallback_reason: portable_failure_reason,
    })
}

/// Tests genuine create/write/delete capability inside `target_dir`.
///
/// If `target_dir` does not exist yet, attempts to create it. If creating a probe
/// file fails after creating a brand-new empty directory, cleans up the empty directory.
fn probe_directory_writable(target_dir: &Path) -> Result<(), String> {
    let existed_before = target_dir.exists();
    if existed_before && !target_dir.is_dir() {
        return Err("target path exists and is not a directory".into());
    }

    if !existed_before {
        fs::create_dir_all(target_dir)
            .map_err(|err| format!("unable to create data directory: {err}"))?;
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seq = PROBE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let probe_name = format!(".write_probe_{}_{}_{}", std::process::id(), nanos, seq);
    let probe_path = target_dir.join(probe_name);

    let write_result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe_path)
        .and_then(|mut file| {
            file.write_all(b"probe")?;
            file.flush()
        });

    let _ = fs::remove_file(&probe_path);

    if let Err(err) = write_result {
        if !existed_before {
            let _ = fs::remove_dir(target_dir);
        }
        return Err(format!("write probe failed: {err}"));
    }

    Ok(())
}
