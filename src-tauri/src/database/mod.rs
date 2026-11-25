pub mod connection;
pub mod health;
pub mod migrations;
pub mod repositories;

use std::path::Path;

use connection::DatabaseConnection;
use health::DatabaseHealth;

use crate::errors::AppResult;

/// Connects to the local SQLite database, runs pending migrations, and verifies
/// baseline database health. Idempotent across repeated startups.
pub async fn initialize(db_path: &Path) -> AppResult<(DatabaseConnection, DatabaseHealth)> {
    let connection = DatabaseConnection::connect(db_path).await?;
    let pool = connection.require_pool()?;

    migrations::run_migrations(&pool).await?;

    let db_health = health::check_health(&pool).await?;
    tracing::info!(
        integrity_ok = db_health.integrity_ok,
        foreign_keys_enabled = db_health.foreign_keys_enabled,
        journal_mode = %db_health.journal_mode,
        applied_migrations = db_health.applied_migrations,
        latest_migration_version = ?db_health.latest_migration_version,
        "Local SQLite database initialized"
    );

    Ok((connection, db_health))
}
