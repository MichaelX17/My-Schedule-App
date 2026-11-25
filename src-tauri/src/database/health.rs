use serde::Serialize;
use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};

/// Read-only diagnostics snapshot for the local SQLite database.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DatabaseHealth {
    pub ok: bool,
    pub integrity_ok: bool,
    pub foreign_keys_enabled: bool,
    pub journal_mode: String,
    pub applied_migrations: i64,
    pub latest_migration_version: Option<i64>,
}

/// Runs a safe, read-only health and integrity check against the SQLite pool.
pub async fn check_health(pool: &SqlitePool) -> AppResult<DatabaseHealth> {
    let integrity_result: String = sqlx::query_scalar("PRAGMA integrity_check(1)")
        .fetch_one(pool)
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "PRAGMA integrity_check failed");
            AppError::DatabaseCorrupted(err.to_string())
        })?;
    let integrity_ok = integrity_result.eq_ignore_ascii_case("ok");

    let fk_flag: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(pool)
        .await?;
    let foreign_keys_enabled = fk_flag == 1;

    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(pool)
        .await?;

    let applied_migrations: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = 1")
            .fetch_one(pool)
            .await?;

    let latest_migration_version: Option<i64> =
        sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1")
            .fetch_one(pool)
            .await?;

    let ok = integrity_ok && foreign_keys_enabled;

    Ok(DatabaseHealth {
        ok,
        integrity_ok,
        foreign_keys_enabled,
        journal_mode: journal_mode.to_ascii_lowercase(),
        applied_migrations,
        latest_migration_version,
    })
}
