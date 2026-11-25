use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};

/// Default SQLite busy timeout to handle brief lock contention safely.
pub const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum number of pooled SQLite connections for this single-operator desktop app.
pub const SQLITE_MAX_CONNECTIONS: u32 = 5;

/// Managed handle for the local SQLite connection pool.
///
/// Backed by `Arc<RwLock<Option<SqlitePool>>>` so database restore can safely close,
/// replace, reopen, and hot-swap the active connection pool without restarting the process.
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pool: Arc<RwLock<Option<SqlitePool>>>,
}

impl DatabaseConnection {
    pub fn uninitialized() -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
        }
    }

    pub fn from_pool(pool: SqlitePool) -> Self {
        Self {
            pool: Arc::new(RwLock::new(Some(pool))),
        }
    }

    pub fn pool(&self) -> Option<SqlitePool> {
        self.pool.read().ok().and_then(|guard| guard.clone())
    }

    pub fn require_pool(&self) -> AppResult<SqlitePool> {
        self.pool()
            .ok_or_else(|| AppError::DatabaseInit("Database pool is not initialized".into()))
    }

    /// Replaces the active pool handle after a validated reopen (e.g. post-restore).
    pub fn replace_pool(&self, new_pool: SqlitePool) {
        if let Ok(mut guard) = self.pool.write() {
            *guard = Some(new_pool);
        }
    }

    /// Closes and detaches the active SQLite pool so all file locks and WAL handles are released.
    pub async fn close(&self) {
        let taken = self.pool.write().ok().and_then(|mut guard| guard.take());
        if let Some(pool) = taken {
            pool.close().await;
        }
    }

    /// Opens (or creates if missing) the SQLite database at `db_path` with:
    /// - `PRAGMA foreign_keys = ON` enforced on every connection;
    /// - `WAL` journal mode for crash resilience and safe concurrent reads;
    /// - `NORMAL` synchronous mode (standard safe pairing with WAL);
    /// - 5-second busy timeout.
    ///
    /// Never overwrites an existing file. If an existing file is corrupted or
    /// not a valid SQLite database, returns `AppError::DatabaseCorrupted`.
    pub async fn connect(db_path: &Path) -> AppResult<Self> {
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                return Err(AppError::DatabaseInit(format!(
                    "Database parent directory does not exist: {}",
                    parent.display()
                )));
            }
        }

        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(SQLITE_BUSY_TIMEOUT);

        let pool = SqlitePoolOptions::new()
            .max_connections(SQLITE_MAX_CONNECTIONS)
            .connect_with(options)
            .await
            .map_err(classify_open_error)?;

        // Verify the database header/pages before running migrations so corrupted
        // files are detected immediately without modifying them.
        let quick_check: String = sqlx::query_scalar("PRAGMA quick_check(1)")
            .fetch_one(&pool)
            .await
            .map_err(classify_open_error)?;

        if !quick_check.eq_ignore_ascii_case("ok") {
            pool.close().await;
            return Err(AppError::DatabaseCorrupted(format!(
                "PRAGMA quick_check reported: {quick_check}"
            )));
        }

        Ok(Self::from_pool(pool))
    }
}

/// Classifies SQLite open/check errors so corrupted or non-database files
/// produce `AppError::DatabaseCorrupted` instead of a generic init error.
pub(crate) fn classify_open_error(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(ref db_err) = err {
        let code = db_err.code();
        let msg = db_err.message().to_ascii_lowercase();
        if matches!(code.as_deref(), Some("11") | Some("26"))
            || msg.contains("not a database")
            || msg.contains("malformed")
            || msg.contains("corrupt")
        {
            return AppError::DatabaseCorrupted(err.to_string());
        }
    }
    AppError::DatabaseInit(err.to_string())
}
