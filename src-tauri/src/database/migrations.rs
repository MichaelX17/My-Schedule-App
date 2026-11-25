use sqlx::migrate::Migrator;
use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};

/// Embedded source-controlled SQLite migrations from `src-tauri/migrations/`.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Executes all pending source-controlled SQLite migrations idempotently.
///
/// SQLx tracks applied migrations in `_sqlx_migrations` with checksums and
/// executes each migration transactionally where supported by SQLite.
pub async fn run_migrations(pool: &SqlitePool) -> AppResult<()> {
    run_migrator(pool, &MIGRATOR).await
}

/// Runs an explicit `Migrator` against `pool` (used by `run_migrations` and tests).
pub async fn run_migrator(pool: &SqlitePool, migrator: &Migrator) -> AppResult<()> {
    migrator.run(pool).await.map_err(|err| {
        tracing::error!(error = %err, "Database migration execution failed");
        AppError::MigrationFailed(err.to_string())
    })
}
