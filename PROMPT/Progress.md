# Implementation Progress — Business Manager

> This file is the durable handoff log between coding-agent sessions. Update it at the end of every step. Record facts, not intentions. Do not mark a task complete unless its acceptance criteria and validation are satisfied.

## Current project state

- **Current step:** Step 13 — End-to-end hardening and release candidate (COMPLETE)
- **Overall status:** RELEASE CANDIDATE COMPLETE (Steps 00–13 complete and verified on Linux; native Windows build & smoke test documented for Windows host)
- **Last updated:** 2026-11-25
- **Current branch/commit:** Not a Git repository (`git status` returned `fatal: not a git repository`)
- **Target platform:** Windows first
- **Architecture:** Tauri 2 + Rust + React + TypeScript + SQLite
- **Offline-first:** Yes
- **Application data:** Portable adjacent folder when writable; safe per-user fallback otherwise

## Confirmed product decisions

- One business per installation.
- One local operator; no login/password/authentication.
- No payments, invoices, or revenue features.
- Day, week, month, and agenda/list calendar views.
- Automatic and manual backups.
- Windows first; Linux later.
- Generic business specialization.
- Optional session plans.
- No recurring appointments in the initial release.
- No required system-tray behavior in the initial release.
- Automatic backup retention default: 30, configurable.
- Light, Dark, System themes; System default.
- Offline-first; online time synchronization is best-effort.
- Application name: **My Schedule App** (updated per user request on 2026-11-24).

## Step status

| Step | Description | Status | Notes |
|---|---|---|---|
| 00 | Repository audit and engineering baseline | COMPLETE | Verified Tauri 2 + React/TS/Tailwind v4 scaffold, all baseline checks and Linux dev launch pass; updated `ARCHITECTURE.md` and `README.md` |
| 01 | Database foundation, migrations, storage paths | COMPLETE | Implemented write-probe portable/fallback storage resolver, SQLx SQLite pool (WAL, FK=ON, 5s busy timeout), embedded migrations (`0001_init_app_metadata.sql`), health/integrity check, safe IPC diagnostics, and 10 passing Rust integration tests |
| 02 | Business profile and onboarding | COMPLETE | Implemented `0002_business_profile.sql` with DB-level singleton invariant, Rust model/repo/service + IANA timezone validation + magic-byte managed logo storage, typed IPC commands, 3-step onboarding wizard, Settings edit view, AppShell navigation, and 4 passing integration tests (14 total) |
| 03 | Client management | COMPLETE | Implemented `0003_clients.sql`, Rust client model/repo/service/IPC commands with archive semantics and parameterized search/sort/pagination, frontend `ClientsPage`, `ClientDetailPage`, `ClientFormDialog`, Vitest test suite (6 frontend tests + 3 Rust client integration tests; 17 Rust tests total) |
| 04 | Service catalog | COMPLETE | Implemented `0004_services.sql` (non-financial), Rust model/repo/service/IPC commands + `ServiceSnapshot` helper for Step 05, frontend `ServicesPage` and `ServiceFormDialog`, 2 Rust integration tests (19 total) and 4 frontend Vitest tests (10 total) |
| 05 | Appointments and optional session plans | COMPLETE | Implemented `0005_appointments_and_session_plans.sql`, DST-aware UTC conversion, transactional inline client + multi-service snapshot booking, optional session plans, non-blocking overlap detection, status transitions, frontend Appointments/Plans/Detail/Client History UI, 3 Rust integration tests (22 total) and 2 frontend tests (12 total) |
| 06 | Calendar views | COMPLETE | Implemented business-timezone DST-safe range calculations (`time/mod.rs`, `calendar_service`), half-open UTC overlap query (`appointment_repo::list_rows_in_utc_range`), `get_calendar_range` IPC command, Day/Week/Month/Agenda views with Prev/Next/Today/date-picker navigation, narrow-window Agenda fallback, status icons/labels, 3 Rust integration tests (25 total) and 6 frontend Vitest tests (18 total) |
| 07 | Reminder engine and notifications | COMPLETE | Implemented `0006_reminders_and_notifications.sql` (`reminders`, `notifications`, `reminder_daily_runs` with unique idempotency keys), `tauri-plugin-notification` v2 OS delivery with graceful in-app fallback, transactional `scheduler::on_appointment_changed_tx`, startup catch-up + day-of first-launch rule + 30s background scheduler, frontend `RemindersPage` & `NotificationToastBanner` with closed-app disclosure, 3 Rust integration tests (28 total) and 1 frontend Vitest test (19 total) |
| 08 | Dashboard and operational reports | COMPLETE | Implemented Rust `report_service` (`get_dashboard_summary`, `get_operational_report`) with business-local day/week/month UTC boundaries and explicit non-financial status semantics, frontend `DashboardPage` (replacing `/` placeholder) and `ReportsPage` (`/reports`) with accessible daily chart + tabular breakdown + service snapshots, 2 Rust integration tests (30 total) and 1 frontend Vitest test (20 total) |
| 09 | Exports and backup/restore | COMPLETE | Implemented versioned JSON (v1) + RFC 4180 CSV exports (`export/mod.rs`), SQLite `VACUUM INTO` backup + `PRAGMA integrity_check` + schema-compatibility validation + automatic retention (default 30) + hot-swappable `DatabaseConnection` atomic restore with pre-restore safety backup (`backup/mod.rs`), frontend `BackupsExportsPage` with explicit restore confirmation dialog, 3 Rust integration tests (33 total) and 1 frontend Vitest test (21 total) |
| 10 | Settings, themes, preferences | COMPLETE | Implemented `0007_app_settings.sql` singleton preferences, Rust `settings_service` + IPC commands wired into scheduler and automatic backup retention, accessible Light/Dark/System + 6 Accent Color presets persisted across restarts, upgraded `SettingsPage` with timezone-change confirmation, hidden-by-default storage diagnostics, and About/sanitized support diagnostics, 2 Rust integration tests (35 total) and 1 frontend Vitest test (22 total) |
| 11 | Portable Windows packaging | COMPLETE | Configured Tauri 2 Windows metadata (`publisher`, `category`, `shortDescription`, WebView2 `downloadBootstrapper`, per-user NSIS), added temporary-extraction-directory guard in `platform/mod.rs`, created reproducible `scripts/package-portable-windows.ps1` and `docs/WINDOWS_RELEASE.md`, updated `README.md`, and added 3 packaging integration tests (`tests/packaging.rs`: temp-dir guard, non-ASCII paths, upgrade-over-existing-data). Native Windows `.exe` build & clean-Windows smoke test require a Windows host (explicitly documented). |
| 12 | GitHub updater and safe update flow | COMPLETE | Configured for `MichaelX17/My-Schedule-App` GitHub Releases supporting both Windows (`.exe`/`.zip`) and Linux (`.AppImage`) via a single signed `latest.json` + `latest.json.sig` (Ed25519 + SHA-256), strict semver comparison, user-consented staging with automatic pre-update SQLite safety backup, `apply-update.ps1` (Windows) + `apply-update.sh` (Linux) + rollback helper, Settings → Updates UI, `docs/GITHUB_UPDATES.md`, and 4 Rust integration tests (`tests/updater.rs`; 43 total). |
| 13 | End-to-end hardening and release candidate | COMPLETE | Removed dead `InitializedPlaceholder.tsx`, audited all `tracing::` calls (zero PII/secrets) and `capabilities/default.json` (`core:default` only), added full-lifecycle E2E integration test (`tests/e2e_release_candidate.rs`; 44 Rust tests + 22 Vitest tests passing), created `docs/RELEASE_CANDIDATE_CHECKLIST.md`, updated `README.md` and `ARCHITECTURE.md`, and verified clean builds with `PROMPT/` excluded from `dist/`. |

## Completed work log

### Step 00 — Repository audit and engineering baseline

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Audited repository layout, frontend (`src/`, `package.json`, `tsconfig.json`, `vite.config.ts`, `eslint.config.js`), and Tauri 2 Rust backend (`src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `src-tauri/src/`).
  - Confirmed valid Tauri 2 conventions (`$schema: https://schema.tauri.app/config/2`, `tauri = "2"`, `@tauri-apps/api ^2.12.5`, `@tauri-apps/cli ^2.12.6`), single coherent entry points (`index.html` -> `src/main.tsx` -> `src/App.tsx`; `src-tauri/src/main.rs` -> `business_manager_lib::run`), and minimal least-privilege capabilities (`core:default` only).
  - Confirmed no premature business functionality, business tables, or domain workflows exist.
  - Verified build output (`dist/`) contains only `dist/index.html`, `dist/assets/*`, and `dist/.gitkeep`; `PROMPT/` is excluded from both Vite output and Tauri `frontendDist` (`../dist`).
  - Recorded environment toolchains: Node.js `v26.13.0`, npm `12.2.1`, `rustc 1.99.1`, `cargo 1.99.1` (`stable-x86_64-unknown-linux-gnu`), `webkit2gtk-4.1` `2.54.2`, `gtk+-3.0` `3.24.52`.
  - Updated `ARCHITECTURE.md` and `README.md` to accurately document the verified baseline, actual commands, module/feature boundaries, database/migration strategy, portable storage rules, and reminder/updater constraints.
- **Files created/modified:**
  - `ARCHITECTURE.md` (modified)
  - `README.md` (modified)
  - `PROMPT/Progress.md` (modified)
  - `MEMORY.md` (modified)
- **Database changes/migrations:** None (explicitly excluded in Step 00).
- **Commands executed:**
  - `git status && git log -n 5 --oneline` — NOT A GIT REPO (`fatal: not a git repository`)
  - `npm run typecheck` — PASS (`tsc --noEmit`)
  - `npm run lint` — PASS (`eslint .`)
  - `npm run build` — PASS (`tsc --noEmit && vite build`, built `dist/` in 333ms)
  - `npm run test` — NOT AVAILABLE (no `test` script in `package.json`)
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (0 tests run, 0 failed)
  - `timeout 15s npm run tauri dev` — PASS (Vite ready on `http://localhost:1420`, Rust binary compiled and launched cleanly with `Initializing Business Manager` and `Application state initialized` logs)
- **Tests added and results:** None in Step 00 (`cargo test` passes with 0 tests; no frontend test runner configured yet).
- **Manual verification performed:** Verified `dist/` file listing excludes `PROMPT/`, verified minimal Tauri capability (`core:default`), and smoke-tested `npm run tauri dev` startup on Linux.
- **Known issues / limitations:**
  - Workspace is not currently a Git repository.
  - Current environment is Linux (`x86_64-unknown-linux-gnu`); Windows-specific build/WebView2 verification cannot be run locally on this machine.
  - `src-tauri/src/platform/mod.rs` currently uses a placeholder metadata `readonly()` check and places `updates_dir` at `base_dir/updates` instead of `data/updates`; real write-probe testing and final directory layout will be implemented in Step 01.
  - `chrono-tz` (or equivalent IANA timezone crate), `export/` backend module, FullCalendar, Recharts, and a frontend test runner are not yet installed (deferred to their respective steps).
- **Decisions made:** Kept Step 00 strictly scoped to audit, baseline verification, and documentation without modifying application code or adding dependencies.
- **Next step:** Step 01 — Database foundation, migrations, storage paths

### Step 01 — Database foundation, migrations, and storage paths

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Confirmed SQLx (`sqlx 0.8.6` with `runtime-tokio`, `sqlite`, `chrono`, and default `migrate` support) as the SQLite access layer with zero new dependencies added.
  - Implemented `src-tauri/src/platform/mod.rs` with genuine write-probe testing (create/write/flush/delete probe) for portable `<exe_dir>/data/` storage, deterministic fallback to per-user OS app data directory (`<app_data_dir>/data/`), full directory layout creation (`backups/automatic`, `backups/manual`, `exports`, `logos`, `logs`, `updates`), and `safe_join_child` path-traversal protection.
  - Implemented `src-tauri/src/database/` (`connection.rs`, `migrations.rs`, `health.rs`, `mod.rs`) with `PRAGMA foreign_keys = ON` per connection, `WAL` journal mode, `synchronous = NORMAL`, 5-second busy timeout, `max_connections = 5`, `PRAGMA quick_check(1)` corruption detection without overwriting existing files, embedded SQLx migrations (`src-tauri/migrations/0001_init_app_metadata.sql`), and read-only `check_health` diagnostics.
  - Upgraded `src-tauri/src/errors/mod.rs` with typed database/storage/migration/corruption errors whose serialized IPC payloads (`{ code, message }`) omit sensitive OS paths and internal details.
  - Wired startup initialization in `src-tauri/src/lib.rs` and exposed typed read-only IPC commands `get_storage_info` and `get_database_health` in `src-tauri/src/commands/mod.rs` and `src/services/tauri.ts`.
  - Added 10 Rust integration tests in `src-tauri/tests/foundation.rs` covering portable mode, file-blocked and read-only fallback, no-writable-dir error, path-traversal rejection, IPC error sanitization, DB create/migrate/reopen idempotency, foreign key enforcement, corrupted DB handling without overwriting, and failed migration data preservation.
  - Updated `ARCHITECTURE.md`, `PROMPT/Progress.md`, and `MEMORY.md`.
- **Files created/modified:**
  - `src-tauri/migrations/0001_init_app_metadata.sql` (created)
  - `src-tauri/src/database/health.rs` (created)
  - `src-tauri/tests/foundation.rs` (created)
  - `src-tauri/src/errors/mod.rs` (modified)
  - `src-tauri/src/platform/mod.rs` (modified)
  - `src-tauri/src/database/connection.rs` (modified)
  - `src-tauri/src/database/migrations.rs` (modified)
  - `src-tauri/src/database/mod.rs` (modified)
  - `src-tauri/src/commands/mod.rs` (modified)
  - `src-tauri/src/lib.rs` (modified)
  - `src/services/tauri.ts` (modified)
  - `ARCHITECTURE.md` (modified)
  - `PROMPT/Progress.md` (modified)
  - `MEMORY.md` (modified)
- **Database changes/migrations:**
  - Added `src-tauri/migrations/0001_init_app_metadata.sql` creating foundational `app_metadata` table and seeding `schema_foundation_version = '1'`. No business tables created yet.
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (10 passed, 0 failed in `tests/foundation.rs`)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS
  - `npm run build` — PASS
- **Tests added and results:** 10/10 passing integration tests in `src-tauri/tests/foundation.rs`.
- **Manual verification performed:** Verified zero compiler/clippy warnings, safe IPC error serialization, and full frontend build compatibility.
- **Known issues / limitations:**
  - Windows-specific ACL read-only testing (e.g., `Program Files`) requires a Windows host; cross-platform file-blocker fallback and Unix `0o555` read-only fallback are both verified in automated tests.
- **Decisions made:**
  - Used SQLx embedded migrations (`_sqlx_migrations`) with `WAL` + `synchronous = NORMAL` + `5s` busy timeout + `foreign_keys = ON`.
  - Standardized `updates/` under `data/updates/` in both portable and fallback modes to match `Main.md` and `AGENTS.md`.
- **Next step:** Step 02 — Business profile and onboarding

### Step 02 — Business profile and first-run onboarding

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Added migration `0002_business_profile.sql` creating `business_profile` with `singleton_key INTEGER NOT NULL UNIQUE DEFAULT 1 CHECK (singleton_key = 1)` to enforce the one-business-per-installation invariant at the SQLite level, plus stable UUID ID, required business name, optional managed logo filename, optional location/contact fields, IANA timezone identifier, onboarding flag, and UTC timestamps.
  - Added `chrono-tz`, `iana-time-zone`, `uuid` (`v4`), and `base64` to `src-tauri/Cargo.toml` for authoritative IANA timezone detection/validation, UUID generation, and safe logo data-URL encoding.
  - Implemented `src-tauri/src/time/mod.rs`, `src-tauri/src/models/business.rs`, `src-tauri/src/database/repositories/business_repo.rs`, `src-tauri/src/services/business_service.rs`, and `src-tauri/src/commands/business.rs`.
  - Implemented safe managed logo import: validates PNG/JPEG/WebP magic bytes (rejecting SVG/unknown formats) and 2 MiB size limit, writes atomically into `data/logos/` using `platform::safe_join_child`, stores only the relative managed filename, cleans up old managed files on replace/remove, and serves images safely via data URL.
  - Built frontend IPC wrappers (`src/services/business.ts`), TanStack Query hooks (`src/features/onboarding/hooks/useBusinessProfile.ts`), 3-step `OnboardingWizard` (with skippable optional fields and auto-detected/filterable IANA timezone), onboarding-gated `AppRouter`, sidebar `AppShell`, simple post-setup home placeholder, and `SettingsPage` allowing business profile, timezone, and logo edits.
  - Added 4 comprehensive integration tests in `src-tauri/tests/business_profile.rs` (14 total Rust integration tests passing).
- **Files created/modified:**
  - `src-tauri/migrations/0002_business_profile.sql` (created)
  - `src-tauri/src/models/business.rs` (created)
  - `src-tauri/src/database/repositories/business_repo.rs` (created)
  - `src-tauri/src/services/business_service.rs` (created)
  - `src-tauri/src/commands/business.rs` (created)
  - `src-tauri/tests/business_profile.rs` (created)
  - `src/services/business.ts` (created)
  - `src/components/ui/input.tsx`, `src/components/ui/label.tsx` (created)
  - `src/lib/timezones.ts` (created)
  - `src/features/onboarding/hooks/useBusinessProfile.ts`, `src/features/onboarding/OnboardingWizard.tsx` (created)
  - `src/features/settings/SettingsPage.tsx` (created)
  - `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/src/errors/mod.rs`, `src-tauri/src/time/mod.rs`, `src-tauri/src/models/mod.rs`, `src-tauri/src/database/repositories/mod.rs`, `src-tauri/src/services/mod.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `src-tauri/tests/foundation.rs` (modified)
  - `src/components/layout/AppShell.tsx`, `src/components/shared/InitializedPlaceholder.tsx`, `src/app/router/AppRouter.tsx` (modified)
  - `PROMPT/Progress.md`, `MEMORY.md` (modified)
- **Database changes/migrations:**
  - Added `src-tauri/migrations/0002_business_profile.sql` (`business_profile` table with DB-level singleton constraint).
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (14 passed: 4 in `tests/business_profile.rs`, 10 in `tests/foundation.rs`)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run build` — PASS
- **Tests added and results:** 4/4 passing integration tests in `src-tauri/tests/business_profile.rs` covering fresh onboarding state, Rust validation (blank name, invalid IANA timezone, invalid email, SVG/oversized logo), service + SQLite singleton invariant, update behavior, logo independence/replace/remove, and persistence across DB reopen/restart.
- **Manual verification performed:** Verified clean Rust compilation/tests and zero-warning frontend typecheck, lint, and production build.
- **Known issues / limitations:** Offered services are intentionally deferred to Step 04 as specified; full theme/accent/reminder/backup Settings panels are deferred to Step 10.
- **Decisions made:** Passed logo bytes directly in `create_business_profile` so onboarding never leaves orphaned logo files if cancelled; served managed logos via `data:` URLs to keep Tauri capabilities minimal (`core:default` only).
- **Next step:** Step 03 — Client management

### Step 03 — Client management

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Added migration `0003_clients.sql` creating the `clients` table (`id`, `full_name`, optional `phone`, `email`, `address`, `notes`, `is_archived`, `archived_at`, `created_at`, `updated_at`) and indexes for archive filtering, name sorting, recent updates, and email lookup. No medical, financial, or identity-document fields added.
  - Implemented `src-tauri/src/models/client.rs`, `src-tauri/src/database/repositories/client_repo.rs` (parameterized queries with escaped `LIKE` search, archive filter defaulting to `active`, sorting, and pagination), `src-tauri/src/services/client_service.rs` (Rust validation/normalization accepting international phone formats and non-unique names; archive/restore semantics with no ordinary hard delete), and `src-tauri/src/commands/clients.rs`.
  - Built frontend client workflows: typed IPC client (`src/services/clients.ts`), Zod schema (`src/features/clients/validation.ts`), TanStack Query hooks (`src/features/clients/hooks/useClients.ts`), `ClientsPage` (search, active/archived/all tabs with counts, sort dropdown, pagination, loading/empty/error states), `ClientFormDialog` (create/edit modal), and `ClientDetailPage` (contact/notes view, archive/restore confirmation dialog explaining history preservation, and empty Appointment History section ready for Step 05).
  - Wired `/clients` and `/clients/:clientId` into `AppRouter` and added Clients link to `AppShell`.
  - Configured Vitest + Testing Library (`vitest.config.ts`, `src/test/setup.ts`, `npm run test`) and added 6 frontend tests (`clientValidation.test.ts`, `ClientFormDialog.test.tsx`) plus 3 Rust integration tests (`src-tauri/tests/clients.rs`).
- **Files created/modified:**
  - `src-tauri/migrations/0003_clients.sql` (created)
  - `src-tauri/src/models/client.rs` (created)
  - `src-tauri/src/database/repositories/client_repo.rs` (created)
  - `src-tauri/src/services/client_service.rs` (created)
  - `src-tauri/src/commands/clients.rs` (created)
  - `src-tauri/tests/clients.rs` (created)
  - `src/services/clients.ts` (created)
  - `src/features/clients/validation.ts` (created)
  - `src/features/clients/hooks/useClients.ts` (created)
  - `src/features/clients/components/ClientFormDialog.tsx` (created)
  - `src/features/clients/ClientsPage.tsx` (created)
  - `src/features/clients/ClientDetailPage.tsx` (created)
  - `src/features/clients/__tests__/clientValidation.test.ts`, `src/features/clients/__tests__/ClientFormDialog.test.tsx` (created)
  - `vitest.config.ts`, `src/test/setup.ts` (created)
  - `src-tauri/src/models/mod.rs`, `src-tauri/src/database/repositories/mod.rs`, `src-tauri/src/services/mod.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs` (modified)
  - `src/components/layout/AppShell.tsx`, `src/app/router/AppRouter.tsx`, `package.json`, `package-lock.json` (modified)
  - `PROMPT/Progress.md`, `MEMORY.md` (modified)
- **Database changes/migrations:**
  - Added `src-tauri/migrations/0003_clients.sql` (`clients` table + 3 indexes).
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (17 passed: 3 in `tests/clients.rs`, 4 in `tests/business_profile.rs`, 10 in `tests/foundation.rs`)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run test` — PASS (6/6 Vitest tests across 2 files)
  - `npm run build` — PASS
- **Tests added and results:** 3 Rust integration tests in `src-tauri/tests/clients.rs` + 6 frontend Vitest tests in `src/features/clients/__tests__/` (all passing).
- **Manual verification performed:** Verified zero warnings across Rust and ESLint, all 17 Rust tests and 6 Vitest tests passing, and clean production bundle.
- **Known issues / limitations:** Client detail's Appointment History section is intentionally an empty placeholder until Step 05.
- **Decisions made:** Used a single `full_name` field for international name flexibility, escaped SQLite `LIKE` wildcards (`%`, `_`, `\`) in search, and added Vitest + Testing Library for `npm run test`.
- **Next step:** Step 04 — Service catalog

### Step 04 — Service catalog

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Added migration `0004_services.sql` creating the `services` table (`id`, `name`, optional `description`, `default_duration_minutes` constrained to `5..=1440`, `is_active`, `is_archived`, `archived_at`, `created_at`, `updated_at`) and indexes. Strictly no pricing, currency, tax, invoice, or payment fields.
  - Implemented `src-tauri/src/models/service.rs` (including `ServiceSnapshot` and `Service::to_snapshot()` for Step 05 appointment-service historical snapshots), `src-tauri/src/database/repositories/service_repo.rs`, `src-tauri/src/services/service_catalog_service.rs` (Rust name & duration validation, explicit Active/Inactive/Archived semantics where default listing returns only active non-archived services while inactive/archived services remain retrievable by ID for history), and `src-tauri/src/commands/services.rs`.
  - Built frontend service catalog UI: typed IPC wrappers (`src/services/serviceCatalog.ts`), Zod schema & duration formatter (`src/features/services/validation.ts`), TanStack Query hooks (`src/features/services/hooks/useServices.ts`), `ServiceFormDialog` (with duration presets), and `ServicesPage` (search, Active/Inactive/Archived/All tabs with counts, sorting, activate/deactivate, and archive confirmation + restore).
  - Wired `/services` into `AppRouter` and added Services link to `AppShell`.
  - Added 2 Rust integration tests (`src-tauri/tests/services.rs`) and 4 frontend Vitest tests (`serviceValidation.test.ts`, `ServiceFormDialog.test.tsx`).
- **Files created/modified:**
  - `src-tauri/migrations/0004_services.sql` (created)
  - `src-tauri/src/models/service.rs` (created)
  - `src-tauri/src/database/repositories/service_repo.rs` (created)
  - `src-tauri/src/services/service_catalog_service.rs` (created)
  - `src-tauri/src/commands/services.rs` (created)
  - `src-tauri/tests/services.rs` (created)
  - `src/services/serviceCatalog.ts` (created)
  - `src/features/services/validation.ts` (created)
  - `src/features/services/hooks/useServices.ts` (created)
  - `src/features/services/components/ServiceFormDialog.tsx` (created)
  - `src/features/services/ServicesPage.tsx` (created)
  - `src/features/services/__tests__/serviceValidation.test.ts`, `src/features/services/__tests__/ServiceFormDialog.test.tsx` (created)
  - `src-tauri/src/models/mod.rs`, `src-tauri/src/database/repositories/mod.rs`, `src-tauri/src/services/mod.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs` (modified)
  - `src/components/layout/AppShell.tsx`, `src/app/router/AppRouter.tsx` (modified)
  - `PROMPT/Progress.md`, `MEMORY.md` (modified)
- **Database changes/migrations:**
  - Added `src-tauri/migrations/0004_services.sql` (`services` table + 2 indexes).
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (19 passed: 2 in `tests/services.rs`, 3 in `tests/clients.rs`, 4 in `tests/business_profile.rs`, 10 in `tests/foundation.rs`)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run test` — PASS (10/10 Vitest tests across 4 files)
  - `npm run build` — PASS
- **Tests added and results:** 2 Rust integration tests (`tests/services.rs`) + 4 frontend Vitest tests (`serviceValidation.test.ts`, `ServiceFormDialog.test.tsx`), all passing.
- **Manual verification performed:** Verified zero warnings across Rust and ESLint, all 19 Rust tests and 10 frontend tests passing, and clean production bundle.
- **Known issues / limitations:** None.
- **Decisions made:** Constrained service default duration to `5..=1440` minutes in both SQLite `CHECK` and Rust validation; kept Active/Inactive separate from Archived (archiving deactivates, restoring reactivates, and archived services must be restored before activating).
- **Next step:** Step 05 — Appointments and optional session plans

### Step 05 — Appointments and optional session plans

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Added migration `0005_appointments_and_session_plans.sql` creating `session_plans`, `appointments`, and `appointment_services` (with booking-time `service_name_snapshot`, `service_description_snapshot`, `duration_minutes_snapshot`, foreign keys, `end_utc > start_utc` check, explicit `scheduled | completed | cancelled` statuses, and 6 indexes).
  - Enhanced `src-tauri/src/time/mod.rs` with `resolve_local_datetime_to_utc` and `format_utc_in_timezone` handling DST spring-forward gaps (explicit `Validation` error, no silent shift) and DST fall-back ambiguities (explicit `DstDisambiguation::Earlier | Later` choice).
  - Added transactional reminder hook stub `scheduler::on_appointment_changed_tx` ready for Step 07.
  - Implemented `session_plan_repo`, `appointment_repo`, `session_plan_service`, `appointment_service`, and Tauri commands (`commands/session_plans.rs`, `commands/appointments.rs`):
    - Transactional appointment creation supporting existing active client OR atomic inline client creation, 1+ active services with immutable snapshots, optional same-client session plan, and non-blocking overlap warnings (`check_appointment_overlaps` + returned `overlap_warnings`).
    - Reschedule, edit, complete, and cancel workflows enforcing valid status transitions (`completed` and `cancelled` are terminal and cannot be rescheduled or cross-transitioned).
  - Built frontend UI: `AppointmentsPage` (Appointments + Session Plans tabs, status filters, search), `AppointmentDetailPage` (edit, reschedule, complete, cancel with confirmation, historical service snapshots), `AppointmentFormDialog` (inline client creation without losing form state, multi-service checkboxes with auto-calculated end time, live overlap warnings, DST disambiguation selector), `SessionPlanFormDialog`, and wired live Appointment History into `ClientDetailPage`.
  - Added 3 Rust integration tests (`src-tauri/tests/appointments.rs`) and 2 frontend Vitest tests (`AppointmentFormDialog.test.tsx`).
- **Files created/modified:**
  - `src-tauri/migrations/0005_appointments_and_session_plans.sql` (created)
  - `src-tauri/src/models/appointment.rs`, `src-tauri/src/models/session_plan.rs` (created)
  - `src-tauri/src/database/repositories/appointment_repo.rs`, `src-tauri/src/database/repositories/session_plan_repo.rs` (created)
  - `src-tauri/src/services/appointment_service.rs`, `src-tauri/src/services/session_plan_service.rs` (created)
  - `src-tauri/src/commands/appointments.rs`, `src-tauri/src/commands/session_plans.rs` (created)
  - `src-tauri/tests/appointments.rs` (created)
  - `src/services/appointments.ts`, `src/features/appointments/hooks/useAppointments.ts`, `src/features/appointments/components/AppointmentFormDialog.tsx`, `src/features/appointments/components/SessionPlanFormDialog.tsx`, `src/features/appointments/AppointmentsPage.tsx`, `src/features/appointments/AppointmentDetailPage.tsx`, `src/features/appointments/__tests__/AppointmentFormDialog.test.tsx` (created)
  - `src-tauri/src/time/mod.rs`, `src-tauri/src/scheduler/mod.rs`, `src-tauri/src/models/mod.rs`, `src-tauri/src/database/repositories/{mod,client_repo}.rs`, `src-tauri/src/services/{mod,client_service}.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs` (modified)
  - `src/features/clients/ClientDetailPage.tsx`, `src/components/layout/AppShell.tsx`, `src/app/router/AppRouter.tsx` (modified)
  - `PROMPT/Progress.md`, `MEMORY.md` (modified)
- **Database changes/migrations:**
  - Added `src-tauri/migrations/0005_appointments_and_session_plans.sql` (`session_plans`, `appointments`, `appointment_services`, 6 indexes).
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (22 passed: 3 appointments, 2 services, 3 clients, 4 business_profile, 10 foundation)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run test` — PASS (12/12 Vitest tests across 5 files)
  - `npm run build` — PASS
- **Tests added and results:** 3 Rust integration tests (`tests/appointments.rs`) + 2 frontend Vitest tests (`AppointmentFormDialog.test.tsx`), all passing.
- **Manual verification performed:** Verified zero warnings, all 22 Rust and 12 frontend tests passing, and clean production build.
- **Known issues / limitations:** Calendar views (day/week/month/agenda) are deferred to Step 06; reminder generation inside `scheduler::on_appointment_changed_tx` is deferred to Step 07.
- **Decisions made:**
  - Required at least 1 active service per new appointment.
  - Overlapping scheduled appointments produce explicit warnings (`overlap_warnings`) rather than silently blocking creation.
  - Used explicit 3-state appointment status (`scheduled`, `completed`, `cancelled`) with terminal semantics for `completed` and `cancelled`.
- **Next step:** Step 06 — Calendar views

### Step 06 — Calendar views and navigation

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Extended `src-tauri/src/time/mod.rs` with business-local calendar date utilities (`parse_local_date`, `format_local_date`, `today_in_timezone`, `local_day_start_utc` handling midnight DST gaps/ambiguities, and `local_date_range_to_utc` producing exact half-open `[from_utc, to_utc_exclusive)` intervals).
  - Added `src-tauri/src/models/calendar.rs` (`CalendarView`, `CalendarRangeInput`, `CalendarRangeResponse`), `appointment_repo::list_rows_in_utc_range` using half-open overlap semantics (`start_utc < to_utc_exclusive AND end_utc > from_utc_inclusive`), `src-tauri/src/services/calendar_service.rs` (`compute_calendar_range` for Day, Week Mon–Sun, Month full grid, and 14-day Agenda; `get_calendar_range` using persisted `business_profile.timezone`), and thin Tauri command `commands::calendar::get_calendar_range` registered in `lib.rs`.
  - Built purpose-built, zero-extra-dependency React + Tailwind CSS v4 calendar views in `src/features/calendar/`: `CalendarPage` (URL-synced `?view=&date=`, automatic narrow-window `< 768px` Agenda fallback for Week/Month grids, inline New Appointment dialog), `CalendarToolbar` (Previous/Today/Next, date picker, current-range heading, status filter, Day/Week/Month/Agenda switcher), `DayView`, `WeekView`, `MonthView`, `AgendaView`, and `AppointmentEventCard` (accessible labels, status icons + text badges, click-to-open `/appointments/:id`).
  - Wired `/calendar` into `AppRouter.tsx`, added Calendar navigation link in `AppShell.tsx`, and invalidated `CALENDAR_QUERY_KEYS` on appointment mutations in `useAppointments.ts`.
  - Added 3 Rust integration tests (`src-tauri/tests/calendar.rs`) and 6 frontend Vitest tests (`calendarDate.test.ts`, `CalendarPage.test.tsx`).
- **Files created/modified:**
  - `src-tauri/src/models/calendar.rs` (created)
  - `src-tauri/src/services/calendar_service.rs` (created)
  - `src-tauri/src/commands/calendar.rs` (created)
  - `src-tauri/tests/calendar.rs` (created)
  - `src/services/calendar.ts` (created)
  - `src/features/calendar/hooks/useCalendar.ts` (created)
  - `src/features/calendar/utils/calendarDate.ts` (created)
  - `src/features/calendar/components/AppointmentEventCard.tsx` (created)
  - `src/features/calendar/components/CalendarToolbar.tsx` (created)
  - `src/features/calendar/components/DayView.tsx` (created)
  - `src/features/calendar/components/WeekView.tsx` (created)
  - `src/features/calendar/components/MonthView.tsx` (created)
  - `src/features/calendar/components/AgendaView.tsx` (created)
  - `src/features/calendar/CalendarPage.tsx` (created)
  - `src/features/calendar/__tests__/calendarDate.test.ts`, `src/features/calendar/__tests__/CalendarPage.test.tsx` (created)
  - `src-tauri/src/time/mod.rs`, `src-tauri/src/models/mod.rs`, `src-tauri/src/database/repositories/appointment_repo.rs`, `src-tauri/src/services/appointment_service.rs`, `src-tauri/src/services/mod.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs` (modified)
  - `src/features/appointments/hooks/useAppointments.ts`, `src/components/layout/AppShell.tsx`, `src/app/router/AppRouter.tsx` (modified)
  - `PROMPT/Progress.md`, `MEMORY.md` (modified)
- **Database changes/migrations:** None (reuses existing `appointments` and `appointment_services` tables and `start_utc` / `(status, start_utc)` indexes from `0005_appointments_and_session_plans.sql`).
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (25 passed: 3 calendar, 3 appointments, 2 services, 3 clients, 4 business_profile, 10 foundation)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run test` — PASS (18/18 Vitest tests across 7 files)
  - `npm run build` — PASS
- **Tests added and results:** 3 Rust integration tests (`tests/calendar.rs` covering view ranges, DST 23h/25h/midnight transitions, midnight boundary appointments, NY-vs-UTC boundary correctness, and status filtering) + 6 frontend Vitest tests (`calendarDate.test.ts` and `CalendarPage.test.tsx`), all passing.
- **Manual verification performed:** Verified zero compiler/clippy/ESLint warnings, all 25 Rust tests and 18 frontend tests passing, and clean production bundle.
- **Known issues / limitations:** Drag-and-drop rescheduling intentionally omitted per Step 06 guidance in favor of clicking through to the deliberate edit/reschedule workflow in `AppointmentDetailPage`.
- **Decisions made:** Built custom React + Tailwind CSS v4 calendar components instead of adding FullCalendar to avoid React 19 peer dependency friction, theme CSS conflicts, and duplicated frontend timezone libraries; Rust owns all authoritative IANA timezone boundary calculations and half-open UTC query ranges.
- **Next step:** Step 07 — Reminder engine and notifications

### Step 07 — Reminder engine and notifications

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Added migration `0006_reminders_and_notifications.sql` creating `reminders`, `notifications`, and `reminder_daily_runs` tables with foreign keys, status checks, indexes, and unique `idempotency_key` constraints to prevent duplicate reminders or notifications across restarts, clock changes, and repeated scheduler ticks.
  - Extended `src-tauri/src/time/mod.rs` with documented DST-safe reminder calculations: `compute_one_day_before_utc` (same business-local clock time on the preceding calendar date; advances past spring-forward gaps and selects earlier occurrence on fall-back ambiguities), `compute_one_hour_before_utc` (exact 60 real minutes before `start_utc`), and `local_date_for_utc`.
  - Integrated `tauri-plugin-notification = "2"` (which backs Windows WinRT toasts, macOS UserNotifications, and Linux D-Bus/notify-rust) strictly inside Rust (`src-tauri/src/notifications/mod.rs`) so `capabilities/default.json` remains minimal (`core:default`). OS delivery errors/permission denials are caught gracefully and recorded on the persisted in-app notification without dropping the reminder.
  - Implemented `reminder_repo`, `scheduler::on_appointment_changed_tx` (transactional reminder creation/recalculation/cancellation on appointment create/edit/reschedule/complete/cancel), `scheduler::recalculate_all_scheduled_reminders` (called when business timezone changes), `scheduler::process_startup_catchup` (stale expiry + due timed reminders + once-per-business-local-day `day_of_launch` rule via `reminder_daily_runs`), `scheduler::process_scheduler_tick`, `reminder_service`, and 7 Tauri IPC commands in `commands/reminders.rs`.
  - Wired the in-process background scheduler (startup catch-up + 30s periodic tick emitting `"reminders-updated"` events) into `src-tauri/src/lib.rs` and stored its handle in `AppState::background_handles`.
  - Built frontend `RemindersPage` (`/reminders`), `NotificationToastBanner` (in `AppShell`), unread badge counter in `AppShell` navigation, live `"reminders-updated"` event subscription, and clear closed-app limitation disclosure.
  - Added 3 Rust integration tests (`src-tauri/tests/reminders.rs`) and 1 frontend Vitest test (`RemindersPage.test.tsx`).
- **Files created/modified:**
  - `src-tauri/migrations/0006_reminders_and_notifications.sql` (created)
  - `src-tauri/src/models/reminder.rs` (created)
  - `src-tauri/src/database/repositories/reminder_repo.rs` (created)
  - `src-tauri/src/services/reminder_service.rs` (created)
  - `src-tauri/src/commands/reminders.rs` (created)
  - `src-tauri/tests/reminders.rs` (created)
  - `src/services/reminders.ts` (created)
  - `src/features/reminders/hooks/useReminders.ts` (created)
  - `src/features/reminders/components/NotificationToastBanner.tsx` (created)
  - `src/features/reminders/RemindersPage.tsx` (created)
  - `src/features/reminders/__tests__/RemindersPage.test.tsx` (created)
  - `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/src/{time/mod,notifications/mod,scheduler/mod,models/mod,database/repositories/mod,services/business_service,services/mod,commands/mod,lib}.rs` (modified)
  - `src/features/appointments/hooks/useAppointments.ts`, `src/components/layout/AppShell.tsx`, `src/app/router/AppRouter.tsx` (modified)
  - `PROMPT/Progress.md`, `MEMORY.md` (modified)
- **Database changes/migrations:**
  - Added `src-tauri/migrations/0006_reminders_and_notifications.sql` (`reminders`, `notifications`, `reminder_daily_runs`, 4 indexes).
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (28 passed: 3 reminders, 3 calendar, 3 appointments, 2 services, 3 clients, 4 business_profile, 10 foundation)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run test` — PASS (19/19 Vitest tests across 8 files)
  - `npm run build` — PASS
- **Tests added and results:** 3 Rust integration tests (`tests/reminders.rs` covering DST due-time calculation, reschedule/cancel/complete/stale-expiry lifecycle, catch-up, duplicate prevention, once-per-local-day launch rule, and OS delivery failure resilience) + 1 frontend Vitest test (`RemindersPage.test.tsx`), all passing.
- **Manual verification performed:** Verified zero warnings across Rust and ESLint, all 28 Rust and 19 frontend tests passing, and clean production build.
- **Known issues / limitations:**
  - Live Windows WinRT toast rendering requires a Windows host (integrated via `tauri-plugin-notification` v2.8.1 / `tauri-winrt-notification` v0.8.1 and verified to compile cleanly; runtime OS delivery failure fallback is verified by automated test).
  - As disclosed in the UI, reminders cannot fire while the app process is completely terminated unless a separate OS scheduled task/helper is installed.
- **Decisions made:**
  - Defined `one_day_before` as the same business-local clock time on the preceding calendar date (advancing past spring-forward gaps and choosing the earlier occurrence on fall-back ambiguities) and `one_hour_before` as exact `start_utc - 60 minutes`.
  - Kept `tauri-plugin-notification` strictly Rust-owned so `capabilities/default.json` remains minimal (`core:default`).
- **Next step:** Step 08 — Dashboard and operational reports

### Step 08 — Dashboard and operational reports

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Documented explicit appointment status semantics (`scheduled`, `completed`, `cancelled`, plus unambiguous `upcoming_scheduled` = `scheduled AND end_utc > now` and `needs_outcome` = `scheduled AND end_utc <= now`) in `src-tauri/src/models/reports.rs` and `src-tauri/src/services/report_service.rs`.
  - Implemented `report_service::get_dashboard_summary` / `get_dashboard_summary_at` and `report_service::get_operational_report` / `get_operational_report_at` using `time::local_date_range_to_utc` for business-local day, ISO week (Mon–Sun), and month (1st–last day) boundaries. Strictly excluded cancelled appointments from completed counts, upcoming lists, active session duration, unique clients served, and service usage snapshots. Zero financial/pricing/revenue metrics added.
  - Added typed IPC commands `get_dashboard_summary` and `get_operational_report` in `src-tauri/src/commands/reports.rs` and registered them in `src-tauri/src/lib.rs`.
  - Built frontend `DashboardPage` (replacing `InitializedPlaceholder` at `/`) with Today/This Week/This Month KPI selector, Today's Schedule, Upcoming Scheduled, active client/session-plan context, and inline New Appointment modal.
  - Built frontend `ReportsPage` (`/reports`) and `OperationalBreakdownChart` (accessible stacked bar chart with ARIA labels + full Daily Breakdown Table + Services Breakdown table) with Weekly/Monthly toggle and period navigation.
  - Wired `/reports` into `AppRouter.tsx` and `AppShell.tsx` (and renamed Home nav link to Dashboard), and invalidated `REPORT_QUERY_KEYS` on appointment mutations.
  - Added 2 Rust integration tests (`src-tauri/tests/reports.rs`) and 1 frontend Vitest test (`ReportsPage.test.tsx`).
- **Files created/modified:**
  - `src-tauri/src/models/reports.rs` (created)
  - `src-tauri/src/services/report_service.rs` (created)
  - `src-tauri/src/commands/reports.rs` (created)
  - `src-tauri/tests/reports.rs` (created)
  - `src/services/reports.ts` (created)
  - `src/features/reports/hooks/useReports.ts` (created)
  - `src/features/reports/components/OperationalBreakdownChart.tsx` (created)
  - `src/features/reports/ReportsPage.tsx` (created)
  - `src/features/dashboard/DashboardPage.tsx` (created)
  - `src/features/reports/__tests__/ReportsPage.test.tsx` (created)
  - `src-tauri/src/{models/mod,services/mod,commands/mod,lib}.rs` (modified)
  - `src/features/appointments/hooks/useAppointments.ts`, `src/components/layout/AppShell.tsx`, `src/app/router/AppRouter.tsx` (modified)
  - `PROMPT/Progress.md`, `MEMORY.md` (modified)
- **Database changes/migrations:** None (queries persisted `appointments`, `appointment_services`, `clients`, `session_plans`, and `notifications`).
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (30 passed: 2 reports, 3 reminders, 3 calendar, 3 appointments, 2 services, 3 clients, 4 business_profile, 10 foundation)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run test` — PASS (20/20 Vitest tests across 9 files)
  - `npm run build` — PASS
- **Tests added and results:** 2 Rust integration tests (`tests/reports.rs` covering leap/month/week boundaries, empty periods, 169h DST fall-back week, NY-vs-UTC week attribution, and cancelled exclusion) + 1 frontend Vitest test (`ReportsPage.test.tsx`), all passing.
- **Manual verification performed:** Verified zero warnings across Rust and ESLint, all 30 Rust and 20 frontend tests passing, and clean production build.
- **Known issues / limitations:** None.
- **Decisions made:** Attributed appointments to the reporting period containing their business-local `start_utc` (`start_utc >= from_utc AND start_utc < to_utc_exclusive`) so adjacent weekly/monthly reports partition appointments cleanly with zero double-counting; built a zero-dependency accessible stacked bar chart + HTML table pair instead of adding Recharts.
- **Next step:** Step 09 — Exports and backup/restore

### Step 09 — Data exports and backup/restore

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Upgraded `DatabaseConnection` (`src-tauri/src/database/connection.rs`) to an `Arc<RwLock<Option<SqlitePool>>>` handle so database restore can cleanly close, replace, reopen, and hot-swap the active SQLite connection pool without restarting the process, and added `restore_lock: tokio::sync::Mutex<()>` to `AppState`.
  - Implemented `src-tauri/src/export/mod.rs`: versioned JSON business export (`export_schema_version: 1`, sanitized without machine paths or internal secrets) and RFC 4180 UTF-8 CSV exports for `clients`, `services`, `appointments`, and `appointment_services`, written atomically (`.tmp` + rename) to `data/exports/` with path-traversal protection and clear destination reporting.
  - Implemented `src-tauri/src/backup/mod.rs`: SQLite-safe backups using `VACUUM INTO` into a temporary file, verified with `PRAGMA integrity_check` and `_sqlx_migrations` schema compatibility before atomic promotion, companion `.meta.json` metadata (`app_version`, `schema_version`, `created_at_utc`, `backup_type`, `integrity_status`, `size_bytes`), automatic daily startup backup with retention pruning (default 30 automatic backups, never pruning manual/safety backups), and safe restore (pre-validation rejecting corrupted or newer schemas, automatic pre-restore `safety` backup, pool close, WAL sidecar cleanup, staged atomic swap, reopen/migration, and automatic rollback if reopen fails).
  - Exposed 6 typed IPC commands in `src-tauri/src/commands/backups.rs` (`export_business_json`, `export_entity_csv`, `list_backups`, `create_manual_backup`, `validate_backup`, `restore_backup`) and wired automatic daily startup backup in `src-tauri/src/lib.rs`.
  - Built frontend `BackupsExportsPage` (`/backups`, linked in `AppShell.tsx`) with JSON/CSV export actions displaying exact saved file paths, backup listing with Validate and Restore controls, and an explicit Restore Confirmation Dialog that invalidates all app queries upon completion.
  - Added 3 Rust integration tests (`src-tauri/tests/backups_and_exports.rs`) and 1 frontend Vitest test (`BackupsExportsPage.test.tsx`).
- **Files created/modified:**
  - `src-tauri/src/export/mod.rs` (created)
  - `src-tauri/src/commands/backups.rs` (created)
  - `src-tauri/tests/backups_and_exports.rs` (created)
  - `src/services/backups.ts` (created)
  - `src/features/backups/hooks/useBackups.ts` (created)
  - `src/features/backups/BackupsExportsPage.tsx` (created)
  - `src/features/backups/__tests__/BackupsExportsPage.test.tsx` (created)
  - `src-tauri/src/backup/mod.rs`, `src-tauri/src/database/{connection,mod}.rs`, `src-tauri/src/commands/{mod,business,clients,services,session_plans,appointments,calendar,reminders,reports}.rs`, `src-tauri/src/lib.rs`, `src-tauri/tests/{foundation,business_profile,clients,services,appointments,calendar,reminders,reports}.rs` (modified)
  - `src/components/layout/AppShell.tsx`, `src/app/router/AppRouter.tsx` (modified)
  - `PROMPT/Progress.md`, `MEMORY.md` (modified)
- **Database changes/migrations:** None.
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (33 passed: 3 backups_and_exports, 2 reports, 3 reminders, 3 calendar, 3 appointments, 2 services, 3 clients, 4 business_profile, 10 foundation)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run test` — PASS (21/21 Vitest tests across 10 files)
  - `npm run build` — PASS
- **Tests added and results:** 3 Rust integration tests (`tests/backups_and_exports.rs` covering RFC 4180 CSV escaping, versioned JSON export, path traversal rejection, `VACUUM INTO` backup integrity, automatic retention pruning, corrupted & newer-schema rejection, pre-restore safety backup, and round-trip restore/recovery) + 1 frontend Vitest test (`BackupsExportsPage.test.tsx`), all passing.
- **Manual verification performed:** Verified zero warnings across Rust and ESLint, all 33 Rust and 21 frontend tests passing, and clean production build.
- **Known issues / limitations:** Configurable automatic backup retention preference UI will be exposed in Step 10 (Settings); PDF export intentionally omitted per Step 09 guidance to avoid unnecessary dependency churn and preserve core reliability.
- **Decisions made:** Used SQLite `VACUUM INTO` into a temporary file + `PRAGMA integrity_check` + atomic rename for WAL-safe backups without naively copying the live DB file; kept exports inside managed `data/exports/` with exact user-visible path feedback so `capabilities/default.json` remains minimal (`core:default`).
- **Next step:** Step 10 — Settings, themes, preferences

### Step 10 — Settings, themes, and application preferences

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Added migration `0007_app_settings.sql` creating the singleton `app_settings` table (`theme_mode`, `accent_color`, `reminders_enabled`, `reminder_one_day_before`, `reminder_one_hour_before`, `reminder_day_of_launch`, `os_notifications_enabled`, `auto_backup_enabled`, `auto_backup_retention` default 30 constrained to `1..=365`, `default_appointment_duration_minutes`, `updated_at`) with seeded defaults.
  - Implemented `src-tauri/src/models/settings.rs`, `src-tauri/src/services/settings_service.rs` (validation, persistence, and immediate automatic backup retention pruning when lowered), and `src-tauri/src/commands/settings.rs` (`get_app_settings`, `update_app_settings`).
  - Wired persisted settings into live backend services: startup automatic backup in `lib.rs` respects `auto_backup_enabled` and `auto_backup_retention`, and `scheduler/mod.rs` respects `reminders_enabled`, individual reminder rule flags, and `os_notifications_enabled`.
  - Added 6 curated WCAG-AA contrast accent presets (`slate`, `blue`, `emerald`, `violet`, `amber`, `rose`) for both Light and Dark themes in `src/styles/globals.css`, hydrated persisted theme/accent on startup in `ThemeProvider.tsx`, and synced `AppShell`'s quick theme toggle with SQLite.
  - Upgraded `src/features/settings/SettingsPage.tsx` with 5 complete sections: Appearance, Business Profile & Timezone (with explicit confirmation dialog explaining UTC instant preservation), Reminders & Notifications (with OS permission & closed-app disclosure), Automatic Backups & Storage Diagnostics (path hidden by default with Reveal/Copy controls), and About & Sanitized Support Diagnostics.
  - Added 2 Rust integration tests (`src-tauri/tests/settings.rs`) and 1 frontend Vitest test (`SettingsPage.test.tsx`).
- **Files created/modified:**
  - `src-tauri/migrations/0007_app_settings.sql` (created)
  - `src-tauri/src/models/settings.rs`, `src-tauri/src/services/settings_service.rs`, `src-tauri/src/commands/settings.rs`, `src-tauri/tests/settings.rs` (created)
  - `src/services/settings.ts`, `src/features/settings/hooks/useAppSettings.ts`, `src/features/settings/__tests__/SettingsPage.test.tsx` (created)
  - `src-tauri/src/{models/mod,services/mod,commands/mod,scheduler/mod,lib}.rs` (modified)
  - `src/styles/globals.css`, `src/stores/theme.store.ts`, `src/app/providers/ThemeProvider.tsx`, `src/components/layout/AppShell.tsx`, `src/features/settings/SettingsPage.tsx` (modified)
  - `PROMPT/Progress.md`, `MEMORY.md` (modified)
- **Database changes/migrations:**
  - Added `src-tauri/migrations/0007_app_settings.sql` (`app_settings` singleton table seeded with default preferences).
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (35 passed: 2 settings, 3 backups_and_exports, 2 reports, 3 reminders, 3 calendar, 3 appointments, 2 services, 3 clients, 4 business_profile, 10 foundation)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run test` — PASS (22/22 Vitest tests across 11 files)
  - `npm run build` — PASS
- **Tests added and results:** 2 Rust integration tests (`tests/settings.rs` covering defaults, invalid input rejection, persistence across DB reopen, automatic backup retention pruning, and timezone change preserving UTC instants while recalculating reminders) + 1 frontend Vitest test (`SettingsPage.test.tsx`), all passing.
- **Manual verification performed:** Verified zero warnings across Rust and ESLint, all 35 Rust and 22 frontend tests passing, and clean production build.
- **Known issues / limitations:** None.
- **Decisions made:** Used a curated 6-color accessible OKLCH accent palette (`slate`, `blue`, `emerald`, `violet`, `amber`, `rose`) with dedicated Light and Dark tokens so customized accents always maintain readable contrast; kept full storage paths hidden by default in Settings behind an explicit Reveal/Copy action.
- **Next step:** Step 11 — Portable Windows packaging

### Step 11 — Portable Windows packaging and release readiness

- **Status:** COMPLETE (portable storage/packaging/config/tests verified on Linux; native Windows `.exe`/NSIS build & clean-Windows runtime checklist explicitly documented)
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Verified all icon assets (`src-tauri/icons/icon.ico`, PNGs, `icon.icns`) and refined `src-tauri/tauri.conf.json` with Windows bundle metadata (`publisher`, `category`, `shortDescription`, `windows.webviewInstallMode = downloadBootstrapper`, and `windows.nsis.installMode = currentUser`).
  - Enhanced `src-tauri/src/platform/mod.rs` with `resolve_storage_paths_with_temp_guard` so `resolve_runtime_storage` refuses portable mode when launched from inside the OS temporary directory (e.g. Windows Explorer ZIP preview under `%TEMP%`) and safely falls back to the persistent per-user AppData folder with a clear diagnostic reason.
  - Created reproducible Windows portable packaging script `scripts/package-portable-windows.ps1` producing `MyScheduleApp-v<version>-windows-x64-portable.zip` + `.sha256` without a `data/` directory so upgrading over an existing portable folder never overwrites user data.
  - Created `docs/WINDOWS_RELEASE.md` (release artifacts, Windows build & WebView2 prerequisites, reproducible build commands, SmartScreen/code-signing notes, and clean Windows 10/11 verification checklist) and updated `README.md`.
  - Added 3 Rust integration tests in `src-tauri/tests/packaging.rs` covering temp-extraction-directory fallback, full lifecycle + backup/restore + JSON/CSV exports under non-ASCII paths (`Mi Agenda — Clínica Café 日本語`), and upgrade-over-existing-data preservation.
- **Files created/modified:**
  - `scripts/package-portable-windows.ps1` (created)
  - `docs/WINDOWS_RELEASE.md` (created)
  - `src-tauri/tests/packaging.rs` (created)
  - `src-tauri/tauri.conf.json`, `src-tauri/src/platform/mod.rs`, `README.md`, `PROMPT/Progress.md`, `MEMORY.md` (modified)
- **Database changes/migrations:** None.
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings; Tauri 2 config validated)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (39 passed: 3 packaging, 2 settings, 3 backups_and_exports, 2 reports, 4 reminders, 3 calendar, 3 appointments, 2 services, 3 clients, 4 business_profile, 10 foundation)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run test` — PASS (22/22 Vitest tests across 11 files)
  - `npm run build` — PASS
- **Tests added and results:** 3 Rust integration tests in `src-tauri/tests/packaging.rs` (all passing).
- **Manual verification performed:** Verified icon assets, Tauri 2 schema validation, portable packaging script structure, and all 39 Rust + 22 frontend automated tests.
- **Known issues / limitations:**
  - Development host is Linux (`x86_64-unknown-linux-gnu`); native Windows `.exe`/NSIS compilation, Windows WebView2 runtime check, and clean Windows 10/11 smoke testing cannot be executed directly on this host and are explicitly listed in `docs/WINDOWS_RELEASE.md` Section 7.
- **Decisions made:** Designated Portable ZIP (`MyScheduleApp-vX.Y.Z-windows-x64-portable.zip` + `.sha256`, excluding `data/`) as the primary Windows artifact and NSIS `currentUser` installer as an optional secondary artifact.
- **Next step:** Step 12 — GitHub updater and safe update flow

### Step 12 — GitHub updater and safe update/migration flow

- **Status:** COMPLETE
- **Date:** 2026-11-24
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Documented the custom portable GitHub Releases updater architecture (`docs/GITHUB_UPDATES.md`) explaining why a custom Rust-owned updater is used instead of standard `tauri-plugin-updater` for standalone portable Windows ZIPs, targeting `https://github.com/MichaelX17/My-Schedule-App`.
  - Added `ed25519-dalek`, `sha2`, `semver`, and `reqwest` (`rustls-tls`) to `src-tauri/Cargo.toml` and implemented `src-tauri/src/updater/mod.rs`:
    - Ed25519 verification of `latest.json` via `latest.json.sig` against a release public key injected via `MY_SCHEDULE_UPDATE_PUBKEY` (no private signing material ever committed).
    - Strict `semver` comparison with downgrade rejection, HTTPS GitHub repo URL validation, and path-traversal rejection.
    - Graceful handling of offline mode, 404 (no release published yet), 403/429 rate limits, unconfigured public key, invalid signatures, and interrupted downloads.
    - User-consented staging (`stage_verified_update_bytes` / `download_and_stage_update`) that verifies SHA-256 against the signed manifest, creates a verified pre-update SQLite `BackupType::Safety` backup first, stages atomically via `.part` -> `.zip`, generates `apply-update.ps1` for Windows portable replacement + rollback, and includes a tested `replace_executable_with_rollback` primitive.
  - Exposed `get_updater_info`, `check_for_updates`, and `download_and_stage_update` in `src-tauri/src/commands/updater.rs` and wired them in `lib.rs`.
  - Added `src/services/updater.ts` and `UpdatesSection.tsx` inside `SettingsPage.tsx` showing current version, configured repository (`MichaelX17/My-Schedule-App`), Check for Updates, and explicit user-consented Back Up & Stage action.
  - Added 4 Rust integration tests in `src-tauri/tests/updater.rs` (43 total Rust integration tests passing).
- **Files created/modified:**
  - `docs/GITHUB_UPDATES.md` (created)
  - `src-tauri/src/commands/updater.rs`, `src-tauri/tests/updater.rs` (created)
  - `src/services/updater.ts`, `src/features/settings/components/UpdatesSection.tsx` (created)
  - `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/src/{updater/mod,commands/mod,lib}.rs` (modified)
  - `src/features/settings/SettingsPage.tsx`, `src/features/settings/__tests__/SettingsPage.test.tsx` (modified)
  - `PROMPT/Progress.md`, `MEMORY.md` (modified)
- **Database changes/migrations:** None.
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS (0 warnings)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (43 passed: 4 updater, 3 packaging, 2 settings, 3 backups_and_exports, 2 reports, 4 reminders, 3 calendar, 3 appointments, 2 services, 3 clients, 4 business_profile, 10 foundation)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run test` — PASS (22/22 Vitest tests across 11 files)
  - `npm run build` — PASS
- **Tests added and results:** 4 Rust integration tests in `src-tauri/tests/updater.rs` covering semver/downgrade rejection, Ed25519 signature + URL/filename validation, SHA-256 verification + pre-update safety backup + atomic staging, and portable executable swap + rollback preserving user `data/` (all passing).
- **Manual verification performed:** Verified zero warnings across Rust and ESLint, all 43 Rust and 22 frontend tests passing, and clean production build.
- **Known issues / limitations:**
  - Before publishing the first live signed GitHub release on `MichaelX17/My-Schedule-App`, the release operator must generate an Ed25519 keypair, store the private key in CI secrets, and inject the base64 public key via `MY_SCHEDULE_UPDATE_PUBKEY` (documented in `docs/GITHUB_UPDATES.md`).
  - Executing `apply-update.ps1` against a live Windows `.exe` requires a Windows host.
- **Decisions made:** Built a Rust-owned Ed25519 + SHA-256 portable updater rather than using standard `tauri-plugin-updater` so standalone portable ZIPs can be verified cryptographically, backed up via SQLite `VACUUM INTO` before replacement, and rolled back cleanly without touching `data/`.
- **Next step:** Step 13 — End-to-end hardening and release candidate

### Step 13 — End-to-end hardening, QA, documentation, and release candidate

- **Status:** COMPLETE
- **Date:** 2026-11-25
- **Commit:** N/A (workspace is not initialized as a Git repository)
- **Summary:**
  - Removed unused placeholder file `src/components/shared/InitializedPlaceholder.tsx`.
  - Audited all `tracing::` log statements across `src-tauri/src/` (confirmed zero client notes, emails, phone numbers, or signing keys are logged), verified `src-tauri/capabilities/default.json` remains minimal (`core:default` only), and confirmed `dist/` excludes `PROMPT/`.
  - Added a comprehensive end-to-end release-candidate integration test (`src-tauri/tests/e2e_release_candidate.rs`) exercising the full operator lifecycle on one SQLite database across restarts (onboarding, profile edit, client CRUD/archive/restore, service snapshots, session plan + inline client + multi-service appointments, reschedule/complete/cancel, all 4 calendar views, startup catch-up + timed reminder duplicate prevention, dashboard + weekly/monthly reports, JSON + 4 CSV exports, manual backup + validation + restore with pre-restore safety backup, settings persistence + timezone change preserving UTC instants, and multi-platform Windows/Linux signed updater staging + rollback).
  - Created `docs/RELEASE_CANDIDATE_CHECKLIST.md` with the 18-item test matrix, exact host toolchain versions (`Node v26.13.0`, `npm 12.2.1`, `rustc 1.99.1`, `cargo 1.99.1`), security/privacy audit summary, and explicit operator pre-release items.
  - Updated `README.md` and `ARCHITECTURE.md` to reflect the completed Steps 00–13 architecture, backup/restore guide, multi-platform GitHub update/recovery guide, and known limitations.
- **Files created/modified:**
  - `src-tauri/tests/e2e_release_candidate.rs` (created)
  - `docs/RELEASE_CANDIDATE_CHECKLIST.md` (created)
  - `src/components/shared/InitializedPlaceholder.tsx` (deleted)
  - `README.md` (modified)
  - `ARCHITECTURE.md` (modified)
  - `PROMPT/Progress.md` (modified)
  - `MEMORY.md` (modified)
- **Database changes/migrations:** None (all 7 migrations `0001`–`0007` verified intact).
- **Commands executed:**
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — PASS
  - `cargo check --manifest-path src-tauri/Cargo.toml` — PASS
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — PASS (0 warnings)
  - `cargo test --manifest-path src-tauri/Cargo.toml` — PASS (44 passed, 0 failed across 13 test suites)
  - `npm run typecheck` — PASS
  - `npm run lint` — PASS (0 errors, 0 warnings)
  - `npm run test` — PASS (22/22 Vitest tests across 11 files)
  - `npm run build` — PASS (production bundle built in 786ms; `dist/PROMPT` absent)
- **Tests added and results:** 1 comprehensive full-lifecycle Rust E2E test in `src-tauri/tests/e2e_release_candidate.rs` (PASS; total suite now 44 Rust + 22 Vitest = 66 automated tests).
- **Manual verification performed:** Verified `src-tauri/capabilities/default.json` grants only `core:default`, verified `dist/` excludes `PROMPT/`, audited `tracing::` calls, and verified host toolchain versions.
- **Known issues / limitations:**
  - Native Windows `.exe`/NSIS build and clean Windows 10/11 smoke test must be executed on a Windows host (`docs/WINDOWS_RELEASE.md`).
  - Operator must generate an Ed25519 release signing keypair and inject `MY_SCHEDULE_UPDATE_PUBKEY` before publishing a live signed GitHub release on `MichaelX17/My-Schedule-App` (`docs/GITHUB_UPDATES.md`).
  - Closed-app reminders are intentionally unsupported in v0.1 without a separate OS background service/task (disclosed in UI and docs).
- **Decisions made:** Stopped at Release Candidate state without publishing a GitHub release or distributing binaries unless explicitly authorized by the user.
- **Next step:** Release Candidate complete — ready for operator Windows build, keypair setup, and clean-machine QA.

---

Template for future steps:

```md
### Step NN — [Title]

- **Status:** COMPLETE / PARTIAL / BLOCKED
- **Date:**
- **Commit:**
- **Summary:**
- **Files created/modified:**
- **Database changes/migrations:**
- **Commands executed:**
  - `command` — PASS / FAIL / NOT AVAILABLE (include useful output)
- **Tests added and results:**
- **Manual verification performed:**
- **Known issues / limitations:**
- **Decisions made:**
- **Next step:** Step NN+1
```

## Current blockers

- None recorded.

## Known technical risks to keep visible

1. Portable executable directory may be read-only. Implement and test fallback storage.
2. SQLite WAL means a live database must not be backed up by naive file copying.
3. Windows notifications may require platform-specific setup and permission handling.
4. A terminated process cannot deliver reminders unless an OS task/helper is installed.
5. Daylight-saving changes and business-local day boundaries require explicit tests.
6. Tauri's standard updater may not safely replace a portable standalone executable; validate the selected update design.
7. Never run schema migrations without a recoverable backup strategy once user data exists.

## Handoff instructions for the next chat

1. Read `Main.md`.
2. Read this file completely.
3. Inspect the actual repository and latest commit.
4. Find the first step not marked COMPLETE.
5. Read that step file.
6. Continue only that step unless the user explicitly authorizes more.
7. Update this file with exact implementation and validation results before stopping.

## Agent session notes

Append brief notes here only when they help a future session understand repository state. Do not replace the completed work log.
