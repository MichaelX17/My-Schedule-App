# Step 01 — Database foundation, migrations, and storage paths

## Objective

Implement the durable local persistence foundation and safe application data-directory resolution. Do not build business-facing screens yet.

## Prerequisites

Step 00 complete. Read `Main.md` and `Progress.md`, inspect the actual code, and verify that Step 00's results still hold.

## Tasks

1. Select or confirm the SQLite access approach already compatible with the scaffold (prefer SQLx if already chosen). Explain any deviation.
2. Implement a single Rust-owned application storage-path resolver:
   - prefer a `data/` directory beside the executable when writable;
   - test actual create/write capability rather than relying only on path existence;
   - fall back to the platform's per-user application data directory if adjacent storage is unavailable;
   - expose the selected storage mode/path through a safe typed command for Settings/About later;
   - avoid path traversal and unsafe path composition.
3. Create the SQLite connection/pool lifecycle and ensure foreign-key enforcement is enabled on every connection.
4. Choose and document journal mode and busy timeout. WAL is acceptable, but account for its sidecar files and backup implications.
5. Add migration infrastructure with source-controlled migration files and a migration tracking mechanism.
6. Add only foundational metadata at this stage if needed (for example, app metadata/schema metadata). Do not create all product tables prematurely; those belong in the feature steps.
7. Implement typed database initialization errors and structured logging. Do not leak sensitive paths or data in user-facing errors.
8. Add a safe database health/integrity check suitable for diagnostics.
9. Ensure database initialization is idempotent and handles first launch, existing database, and failed migration safely.
10. Add tests for:
    - path selection and fallback behavior;
    - unwritable adjacent path simulation where feasible;
    - database creation and reopen;
    - foreign key enforcement;
    - migration execution and repeat startup;
    - corrupted/invalid database error handling.
11. Document the data layout, storage fallback, SQLite mode, migration strategy, and backup caveats in `ARCHITECTURE.md`.
12. Update `Progress.md`.

## Constraints

- No frontend direct database access.
- No arbitrary SQL command exposed to the frontend.
- No copying of a live SQLite database as a backup.
- Do not add client, service, appointment, reminder, or report screens in this step.
- Do not silently overwrite an existing database.

## Acceptance criteria

- Database initializes reliably and repeatedly.
- Path resolver chooses adjacent storage only when it is genuinely writable.
- Fallback path is deterministic and documented.
- Migrations are tracked and testable.
- Rust tests pass for the new foundation.
- App still builds.
- Progress log is complete and truthful.

## Stop condition

Stop after Step 01. Do not start Step 02 without authorization.
