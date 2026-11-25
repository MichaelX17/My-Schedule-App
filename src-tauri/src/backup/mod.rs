use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::connection::{classify_open_error, DatabaseConnection, SQLITE_BUSY_TIMEOUT};
use crate::database::{self, migrations};
use crate::errors::{AppError, AppResult};
use crate::platform::{self, AppPaths};
use crate::time;

/// Default maximum number of automatic backups retained (configurable in Step 10).
pub const DEFAULT_AUTO_BACKUP_RETENTION: usize = 30;

/// Backup category determining storage folder and retention policy.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackupType {
    Manual,
    Automatic,
    Safety,
}

impl BackupType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Automatic => "automatic",
            Self::Safety => "safety",
        }
    }
}

/// Companion metadata persisted alongside every SQLite backup file (`<stem>.meta.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupMetadata {
    pub id: String,
    pub filename: String,
    pub backup_type: BackupType,
    pub app_version: String,
    pub schema_version: i64,
    pub created_at_utc: String,
    pub size_bytes: u64,
    pub integrity_status: String,
    pub notes: Option<String>,
}

/// Validation report returned when inspecting a backup file before restore.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupValidationResult {
    pub valid: bool,
    pub integrity_ok: bool,
    pub schema_compatible: bool,
    pub backup_schema_version: i64,
    pub current_schema_version: i64,
    pub message: String,
}

/// Summary returned after a successful database restore.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestoreResult {
    pub restored_filename: String,
    pub restored_schema_version: i64,
    pub safety_backup: BackupMetadata,
}

/// Returns the highest migration version embedded in this build of My Schedule App.
pub fn current_supported_schema_version() -> i64 {
    migrations::MIGRATOR
        .iter()
        .map(|m| m.version)
        .max()
        .unwrap_or(0)
}

fn backup_dir_for_type(paths: &AppPaths, backup_type: BackupType) -> &Path {
    match backup_type {
        BackupType::Automatic => &paths.automatic_backups_dir,
        BackupType::Manual | BackupType::Safety => &paths.manual_backups_dir,
    }
}

fn metadata_path_for_sqlite(sqlite_path: &Path) -> PathBuf {
    sqlite_path.with_extension("meta.json")
}

/// Validates a SQLite backup file read-only using `PRAGMA integrity_check` and
/// checks schema version compatibility against `current_supported_schema_version()`.
pub async fn validate_backup_file(backup_path: &Path) -> AppResult<BackupValidationResult> {
    if !backup_path.is_file() {
        return Err(AppError::NotFound("Backup file was not found.".into()));
    }

    let current_version = current_supported_schema_version();
    let options = SqliteConnectOptions::new()
        .filename(backup_path)
        .read_only(true)
        .busy_timeout(SQLITE_BUSY_TIMEOUT);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(classify_open_error)?;

    let integrity_res: Result<String, sqlx::Error> = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await;

    let integrity_ok = match integrity_res {
        Ok(val) => val.eq_ignore_ascii_case("ok"),
        Err(err) => {
            pool.close().await;
            return Err(classify_open_error(err));
        }
    };

    if !integrity_ok {
        pool.close().await;
        return Ok(BackupValidationResult {
            valid: false,
            integrity_ok: false,
            schema_compatible: false,
            backup_schema_version: 0,
            current_schema_version: current_version,
            message: "Backup failed SQLite integrity verification.".into(),
        });
    }

    // Verify _sqlx_migrations table exists and read the backup's applied schema version.
    let has_migrations_table: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    if has_migrations_table == 0 {
        pool.close().await;
        return Ok(BackupValidationResult {
            valid: false,
            integrity_ok: true,
            schema_compatible: false,
            backup_schema_version: 0,
            current_schema_version: current_version,
            message:
                "Backup file is not a valid My Schedule App database (missing schema metadata)."
                    .into(),
        });
    }

    let backup_schema_version: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations WHERE success = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    pool.close().await;

    if backup_schema_version > current_version {
        return Ok(BackupValidationResult {
            valid: false,
            integrity_ok: true,
            schema_compatible: false,
            backup_schema_version,
            current_schema_version: current_version,
            message: format!(
                "Backup schema version ({backup_schema_version}) is newer than this app supports ({current_version}). Update My Schedule App before restoring."
            ),
        });
    }

    Ok(BackupValidationResult {
        valid: true,
        integrity_ok: true,
        schema_compatible: true,
        backup_schema_version,
        current_schema_version: current_version,
        message: format!(
            "Backup verified (integrity OK, schema v{backup_schema_version} compatible with v{current_version})."
        ),
    })
}

/// Creates a SQLite-safe, WAL-consistent backup using `VACUUM INTO` into a temporary file,
/// verifies integrity before committing, writes companion metadata, and enforces automatic
/// backup retention when `backup_type == BackupType::Automatic`.
pub async fn create_backup(
    pool: &SqlitePool,
    paths: &AppPaths,
    backup_type: BackupType,
    notes: Option<String>,
    auto_retention_limit: Option<usize>,
) -> AppResult<BackupMetadata> {
    let target_dir = backup_dir_for_type(paths, backup_type);
    fs::create_dir_all(target_dir)?;

    let now = Utc::now();
    let stamp = now.format("%Y%m%d_%H%M%S_%3f");
    let short_id = &Uuid::new_v4().simple().to_string()[..8];
    let stem = format!("{}_{}_{}", backup_type.as_str(), stamp, short_id);
    let final_filename = format!("{stem}.sqlite");
    let tmp_filename = format!(".{stem}.sqlite.tmp");

    let final_path = platform::safe_join_child(target_dir, &final_filename)?;
    let tmp_path = platform::safe_join_child(target_dir, &tmp_filename)?;

    // SQLite's VACUUM INTO creates a clean, transactional snapshot of the live WAL database
    // without naively copying the active main file.
    let tmp_path_str = tmp_path.to_string_lossy().into_owned();
    if let Err(err) = sqlx::query("VACUUM INTO ?1")
        .bind(&tmp_path_str)
        .execute(pool)
        .await
    {
        let _ = fs::remove_file(&tmp_path);
        return Err(AppError::Database(err));
    }

    // Validate the temporary snapshot before promoting it to a real backup.
    let validation = match validate_backup_file(&tmp_path).await {
        Ok(v) if v.valid => v,
        Ok(v) => {
            let _ = fs::remove_file(&tmp_path);
            return Err(AppError::DatabaseCorrupted(v.message));
        }
        Err(err) => {
            let _ = fs::remove_file(&tmp_path);
            return Err(err);
        }
    };

    // Atomically promote the verified snapshot.
    if let Err(err) = fs::rename(&tmp_path, &final_path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(AppError::Io(err));
    }

    let size_bytes = fs::metadata(&final_path)?.len();
    let metadata = BackupMetadata {
        id: stem,
        filename: final_filename,
        backup_type,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version: validation.backup_schema_version,
        created_at_utc: time::format_utc_iso(now),
        size_bytes,
        integrity_status: "ok".to_string(),
        notes,
    };

    let meta_path = metadata_path_for_sqlite(&final_path);
    let meta_bytes = serde_json::to_vec_pretty(&metadata)
        .map_err(|e| AppError::Internal(format!("Failed to serialize backup metadata: {e}")))?;
    fs::write(&meta_path, meta_bytes)?;

    if backup_type == BackupType::Automatic {
        let retention = auto_retention_limit
            .unwrap_or(DEFAULT_AUTO_BACKUP_RETENTION)
            .max(1);
        prune_automatic_backups(&paths.automatic_backups_dir, retention)?;
    }

    tracing::info!(
        backup_type = backup_type.as_str(),
        schema_version = metadata.schema_version,
        size_bytes = metadata.size_bytes,
        "SQLite backup created and verified"
    );

    Ok(metadata)
}

/// Retains only the newest `retention_limit` automatic backups, deleting older `.sqlite`
/// and `.meta.json` pairs. Never touches manual or safety backups.
pub fn prune_automatic_backups(auto_dir: &Path, retention_limit: usize) -> AppResult<usize> {
    if !auto_dir.is_dir() {
        return Ok(0);
    }

    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    for entry in fs::read_dir(auto_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || !name.ends_with(".sqlite") {
            continue;
        }
        entries.push((name.to_string(), path));
    }

    // Filenames embed sortable UTC timestamp `automatic_YYYYMMDD_HHMMSS_mmm_xxxxxxxx.sqlite`.
    entries.sort_by(|a, b| b.0.cmp(&a.0));

    let mut pruned = 0usize;
    if entries.len() > retention_limit {
        for (_, old_path) in entries.into_iter().skip(retention_limit) {
            let _ = fs::remove_file(&old_path);
            let _ = fs::remove_file(metadata_path_for_sqlite(&old_path));
            pruned += 1;
        }
    }
    Ok(pruned)
}

/// Creates a daily automatic backup on startup if no automatic backup has been created yet
/// on the current UTC date. Returns `Some(BackupMetadata)` when a backup was created.
pub async fn run_startup_automatic_backup(
    pool: &SqlitePool,
    paths: &AppPaths,
    retention_limit: Option<usize>,
) -> AppResult<Option<BackupMetadata>> {
    let today_prefix = format!("automatic_{}", Utc::now().format("%Y%m%d"));
    if paths.automatic_backups_dir.is_dir() {
        for entry in fs::read_dir(&paths.automatic_backups_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(&today_prefix) && name.ends_with(".sqlite") {
                    return Ok(None);
                }
            }
        }
    }

    let created = create_backup(
        pool,
        paths,
        BackupType::Automatic,
        Some("Automatic daily startup backup".into()),
        retention_limit,
    )
    .await?;
    Ok(Some(created))
}

fn read_backups_in_dir(dir: &Path, default_type: BackupType) -> AppResult<Vec<BackupMetadata>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut list = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || !name.ends_with(".sqlite") {
            continue;
        }

        let meta_path = metadata_path_for_sqlite(&path);
        if let Ok(raw) = fs::read_to_string(&meta_path) {
            if let Ok(meta) = serde_json::from_str::<BackupMetadata>(&raw) {
                list.push(meta);
                continue;
            }
        }

        // Fallback metadata if companion JSON was missing.
        let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let stem = name.trim_end_matches(".sqlite").to_string();
        let inferred_type = if stem.starts_with("safety_") {
            BackupType::Safety
        } else {
            default_type
        };
        list.push(BackupMetadata {
            id: stem,
            filename: name.to_string(),
            backup_type: inferred_type,
            app_version: "unknown".to_string(),
            schema_version: 0,
            created_at_utc: String::new(),
            size_bytes,
            integrity_status: "unverified".to_string(),
            notes: None,
        });
    }

    Ok(list)
}

/// Lists all available manual, safety, and automatic backups newest first.
pub fn list_backups(paths: &AppPaths) -> AppResult<Vec<BackupMetadata>> {
    let mut all = read_backups_in_dir(&paths.manual_backups_dir, BackupType::Manual)?;
    all.extend(read_backups_in_dir(
        &paths.automatic_backups_dir,
        BackupType::Automatic,
    )?);
    all.sort_by(|a, b| {
        b.created_at_utc
            .cmp(&a.created_at_utc)
            .then_with(|| b.filename.cmp(&a.filename))
    });
    Ok(all)
}

/// Resolves a backup file path safely inside the managed backup directories.
pub fn resolve_managed_backup_path(
    paths: &AppPaths,
    backup_type: BackupType,
    filename: &str,
) -> AppResult<PathBuf> {
    let trimmed = filename.trim();
    if trimmed.is_empty() || !trimmed.ends_with(".sqlite") {
        return Err(AppError::Validation("Invalid backup filename.".into()));
    }
    let dir = backup_dir_for_type(paths, backup_type);
    platform::safe_join_child(dir, trimmed)
}

fn remove_wal_sidecars(db_path: &Path) {
    let base = db_path.to_string_lossy();
    let _ = fs::remove_file(format!("{base}-wal"));
    let _ = fs::remove_file(format!("{base}-shm"));
}

/// Restores a validated backup with:
/// 1. Pre-restore validation (integrity + schema compatibility — rejects newer schemas).
/// 2. Automatic safety backup of the current live database before any file replacement.
/// 3. Safe pool close, WAL sidecar cleanup, staged atomic swap, and reopen/migration.
/// 4. Automatic rollback to the original database if replace or reopen fails.
pub async fn restore_backup(
    db: &DatabaseConnection,
    paths: &AppPaths,
    backup_type: BackupType,
    filename: &str,
) -> AppResult<RestoreResult> {
    let backup_path = resolve_managed_backup_path(paths, backup_type, filename)?;

    // Step 1: Validate candidate backup before touching active data.
    let validation = validate_backup_file(&backup_path).await?;
    if !validation.valid {
        return Err(AppError::Validation(validation.message));
    }

    // Step 2: Create a verified safety backup of the current live database.
    let live_pool = db.require_pool()?;
    let safety_backup = create_backup(
        &live_pool,
        paths,
        BackupType::Safety,
        Some(format!(
            "Pre-restore safety backup before restoring {filename}"
        )),
        None,
    )
    .await?;

    // Flush committed WAL frames into the main SQLite file before closing and
    // detaching so a rollback to `rollback_path` can never lose recent WAL writes.
    let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&live_pool)
        .await;

    // Drop local clone before closing so no references linger.
    drop(live_pool);

    // Step 3: Close active pool cleanly to release SQLite locks and WAL handles.
    db.close().await;

    let db_path = &paths.database_path;
    let staging_path = db_path.with_extension("sqlite.restore_staging");
    let rollback_path = db_path.with_extension("sqlite.rollback_bak");

    let _ = fs::remove_file(&staging_path);
    let _ = fs::remove_file(&rollback_path);

    // Copy validated backup into staging path first.
    if let Err(err) = fs::copy(&backup_path, &staging_path) {
        // Reopen original database so app stays usable.
        if let Ok((reopened, _)) = database::initialize(db_path).await {
            if let Some(p) = reopened.pool() {
                db.replace_pool(p);
            }
        }
        return Err(AppError::Io(err));
    }

    // Move original DB aside and promote staging DB.
    remove_wal_sidecars(db_path);
    if db_path.exists() {
        if let Err(err) = fs::rename(db_path, &rollback_path) {
            let _ = fs::remove_file(&staging_path);
            if let Ok((reopened, _)) = database::initialize(db_path).await {
                if let Some(p) = reopened.pool() {
                    db.replace_pool(p);
                }
            }
            return Err(AppError::Io(err));
        }
    }

    if let Err(err) = fs::rename(&staging_path, db_path) {
        // Rollback original DB immediately.
        let _ = fs::rename(&rollback_path, db_path);
        let _ = fs::remove_file(&staging_path);
        if let Ok((reopened, _)) = database::initialize(db_path).await {
            if let Some(p) = reopened.pool() {
                db.replace_pool(p);
            }
        }
        return Err(AppError::Io(err));
    }

    // Step 4: Reopen and run any forward migrations on the restored database.
    match database::initialize(db_path).await {
        Ok((reopened, _health)) => {
            let new_pool = reopened.require_pool()?;
            db.replace_pool(new_pool);
            let _ = fs::remove_file(&rollback_path);

            tracing::info!(
                restored_filename = %filename,
                safety_backup = %safety_backup.filename,
                "Database restored successfully"
            );

            Ok(RestoreResult {
                restored_filename: filename.to_string(),
                restored_schema_version: validation.backup_schema_version,
                safety_backup,
            })
        }
        Err(reopen_err) => {
            // Automatic rollback to preserve the pre-restore database!
            tracing::error!(
                error = %reopen_err,
                "Restored database failed initialization; rolling back to previous database"
            );
            remove_wal_sidecars(db_path);
            let _ = fs::remove_file(db_path);
            if rollback_path.exists() {
                let _ = fs::rename(&rollback_path, db_path);
            }
            if let Ok((restored_orig, _)) = database::initialize(db_path).await {
                if let Some(p) = restored_orig.pool() {
                    db.replace_pool(p);
                }
            }
            Err(reopen_err)
        }
    }
}
