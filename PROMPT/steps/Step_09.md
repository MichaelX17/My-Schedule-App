# Step 09 — Data exports and backup/restore

## Objective

Implement reliable user-controlled exports and safe automatic/manual SQLite backups.

## Tasks

1. Implement JSON export of business data with a versioned export format and documented schema.
2. Implement CSV exports for practical entities (clients, services, appointments, appointment services) with correct escaping, encoding, and headers.
3. Consider PDF reports only if a maintainable library and reliable layout are available; do not compromise core backup/export correctness.
4. Use save dialogs or a clear user-selected destination. Never silently write to an unexpected location.
5. Implement SQLite-safe backup using the SQLite online backup API or a proven equivalent. Do not copy only the main DB file while WAL is active.
6. Support manual backup and automatic scheduled/startup backup policy.
7. Default to retaining the latest 30 automatic backups; expose retention as a preference in Step 10.
8. Store metadata: app version, schema version, creation timestamp, backup type, and integrity result.
9. Validate backups using SQLite integrity checks and schema compatibility checks.
10. Implement restore with:
    - explicit user confirmation;
    - a safety backup of the current database;
    - validation before replacing active data;
    - safe close/replace/reopen sequence;
    - recoverable failure behavior.
11. Do not restore an incompatible/newer schema silently. Explain the issue and preserve the current database.
12. Add tests for CSV escaping, JSON versioning, backup integrity, retention, interrupted backup behavior where feasible, restore, and failed restore recovery.
13. Update `Progress.md`.

## Acceptance criteria

- Manual and automatic backups work.
- Restore works and preserves data.
- Backups are validated.
- Retention works.
- Exports are user-accessible and correctly formatted.
- No unsafe live-database file copying.
- Tests/build pass.

## Stop condition

Stop after Step 09.
