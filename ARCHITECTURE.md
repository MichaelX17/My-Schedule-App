# Architecture Overview

My Schedule App is a Windows-first, offline-first, single-operator desktop application structured as a **modular monolith** powered by **Tauri 2**, **Rust**, **React**, **TypeScript**, and **SQLite**.

## 1. Frontend / Backend Boundary & IPC-Only Data Access

The application enforces a strict separation of responsibilities:

- **Frontend (`src/`)**:
  - Owns presentation, layout, navigation, client-side form state, and client-side validation.
  - Uses React 19, TypeScript (strict mode), Vite, Tailwind CSS v4, shadcn/ui primitives, Lucide icons, React Router, TanStack Query (IPC server state), Zustand (local UI state such as theme/accent), React Hook Form, and Zod.
  - Communicates with the backend **exclusively through typed Tauri IPC commands** wrapped in `src/services/`.
  - **Never** accesses SQLite, the local filesystem, OS notifications, or update mechanisms directly.
- **Rust Backend (`src-tauri/src/`)**:
  - Owns all persistence, business rules, authoritative input validation, timezone/scheduling logic, filesystem operations, exports, backups, notifications, updates, and OS integration.
  - Exposes narrow, typed `snake_case` Tauri commands returning structured, serializable `AppResult<T>` errors without leaking raw internal stack traces.
  - Keeps Tauri 2 capabilities (`src-tauri/capabilities/default.json`) minimal and least-privilege (`core:default` only; OS notifications, backups, exports, and HTTPS update calls all execute in Rust).

## 2. Rust Module Boundaries (`src-tauri/src/`)

| Module | Responsibility |
|---|---|
| `main.rs`, `lib.rs` | Desktop entry point, `tracing` initialization, `AppState` wiring, startup backup & reminder catch-up, 30s background scheduler loop, and Tauri command registration. |
| `commands/` | Thin Tauri IPC handlers (`system`, `business`, `clients`, `services`, `appointments`, `session_plans`, `calendar`, `reminders`, `reports`, `backups`, `settings`, `updater`) delegating to services/engines. |
| `models/` | Typed domain entities, DTOs, and explicit status enums (`appointment`, `business`, `calendar`, `client`, `reminder`, `reports`, `service`, `session_plan`, `settings`). |
| `services/` | Core business rules, transactional orchestration, and authoritative validation (`business_service`, `client_service`, `service_catalog_service`, `appointment_service`, `session_plan_service`, `calendar_service`, `reminder_service`, `report_service`, `settings_service`). |
| `database/` | Hot-swappable SQLite `DatabaseConnection` (`connection.rs`), migration runner (`migrations.rs`), health diagnostics (`health.rs`), and parameterized SQL repositories (`repositories/`). |
| `platform/` | OS-isolated path resolution, write probing, temporary extraction directory refusal (`%TEMP%`), portable vs. fallback storage selection, and child-path traversal protection (`safe_join_child`). |
| `time/` | UTC instant storage, IANA timezone conversions (`chrono-tz`), DST gap/ambiguity handling, and best-effort online time synchronization with system-clock fallback. |
| `scheduler/` | Persistent reminder calculation (`one_day_before`, `one_hour_before`, `day_of_launch`), startup catch-up, and background tick orchestration. |
| `notifications/` | In-app notification queue persistence and Rust-owned OS notification dispatch (`tauri-plugin-notification`) with graceful fallback. |
| `backup/` | SQLite-safe backups via `VACUUM INTO` (automatic, manual, safety), `PRAGMA integrity_check` & schema-version validation, retention pruning, and atomic restore with WAL checkpointing and rollback. |
| `export/` | Versioned JSON export (schema v1) and RFC 4180 CSV exports (`clients`, `services`, `appointments`, `appointment_services`). |
| `updater/` | Multi-platform (`windows-x86_64` `.exe`/`.zip` + `linux-x86_64` `.AppImage`) GitHub Releases update discovery, Ed25519 manifest signature verification, SHA-256 artifact checks, pre-update SQLite safety backup, atomic `.part` staging, platform swap scripts, and executable rollback. |
| `errors/` | Centralized `AppError` (`thiserror`) and `AppResult<T>` serializable across IPC. |

## 3. Frontend Feature Boundaries (`src/`)

- `src/app/`: Top-level providers (`AppProviders`, `QueryProvider`, `ThemeProvider` with startup hydration) and route definitions (`AppRouter`).
- `src/components/`: Reusable UI primitives (`ui/`), application shell (`layout/`), and shared widgets (`shared/`, including `NotificationToastBanner`).
- `src/services/`: Typed Tauri IPC command wrappers (`tauri.ts`, `business.ts`, `clients.ts`, `services.ts`, `appointments.ts`, `sessionPlans.ts`, `calendar.ts`, `reminders.ts`, `reports.ts`, `backups.ts`, `settings.ts`, `updater.ts`).
- `src/stores/`: Lightweight client-only UI state (`theme.store.ts` for theme mode and OKLCH accent preset).
- `src/features/`: Domain feature modules:
  - `onboarding/` (first-run business profile setup)
  - `dashboard/` (operational overview and quick actions)
  - `clients/` (client directory, archive/restore, history)
  - `services/` (non-financial service catalog)
  - `appointments/` (individual scheduled sessions, inline client creation, and optional session plans)
  - `calendar/` (day, week, month, and agenda/list views with narrow-window Agenda fallback)
  - `reminders/` (in-app notification inbox, upcoming reminders, closed-app disclosure)
  - `reports/` (weekly/monthly non-financial operational reports with accessible charts and tabular equivalents)
  - `backups/` (JSON/CSV exports, manual/automatic backups, integrity-checked restore)
  - `settings/` (business profile, timezone change confirmation, theme/accent, reminder & backup retention preferences, hidden-by-default storage diagnostics, About, and GitHub Updates)
- `src/hooks/`, `src/lib/`, `src/types/`, `src/styles/`: Shared React Query hooks, utilities, types, and Tailwind CSS v4 design tokens (`globals.css`).

## 4. Database Ownership & Migration Strategy

- **Ownership**: Only the Rust backend opens or mutates the SQLite database via **SQLx** (`src-tauri/src/database/`). No arbitrary SQL command is exposed to the frontend.
- **Connection & Integrity Settings** (`src-tauri/src/database/connection.rs`):
  - **Foreign keys**: `PRAGMA foreign_keys = ON` is enforced on every pooled connection.
  - **Journal mode**: `WAL` paired with `synchronous = NORMAL` for crash resilience and safe concurrent reads.
  - **Busy timeout & pooling**: `busy_timeout = 5s`, `max_connections = 5`, wrapped in `Arc<RwLock<Option<SqlitePool>>>` for safe close-and-reopen during backup restore.
  - **Corruption detection**: Runs `PRAGMA quick_check(1)` before executing migrations. Corrupted or non-SQLite files return `AppError::DatabaseCorrupted` without overwriting or truncating the existing file.
- **Source-Controlled Migrations** (`src-tauri/migrations/`):
  1. `0001_init_app_metadata.sql` — foundational metadata (`app_metadata`)
  2. `0002_business_profile.sql` — single-row business profile and timezone
  3. `0003_clients.sql` — clients with archive support and search indexes
  4. `0004_services.sql` — non-financial service catalog
  5. `0005_appointments_and_session_plans.sql` — individual appointments, historical `appointment_services` snapshots, and optional `session_plans`
  6. `0006_reminders_and_notifications.sql` — persistent `reminders`, `notifications`, and `reminder_daily_runs`
  7. `0007_app_settings.sql` — singleton `app_settings` (theme, accent, reminders, backup retention)

## 5. Portable Storage Strategy

A single Rust-owned storage resolver (`src-tauri/src/platform/mod.rs`) determines where application data lives:

1. Inspect the executable's directory (`std::env::current_exe()`).
2. Reject volatile temporary extraction directories (such as `%TEMP%`, `AppData\Local\Temp`, or archive-viewer temp folders) so portable data is never accidentally written to a self-cleaning temp path.
3. Run a genuine **write probe** (`create_new(true)`, write, flush, remove) inside `<exe_dir>/data`.
4. If writable, run in **portable mode** (`StorageMode::Portable`) using the adjacent layout:

```text
MyScheduleApp/
├── MyScheduleApp.exe (or MyScheduleApp.AppImage on Linux)
└── data/
    ├── database.sqlite
    ├── backups/
    │   ├── automatic/
    │   ├── manual/
    │   └── safety/
    ├── exports/
    ├── logos/
    ├── logs/
    └── updates/
```

5. If protected/read-only (e.g., `Program Files`) or inside a temp directory, fall back cleanly to `<per-user-app-data>/data` (`StorageMode::Fallback`), probe writability, and surface the active mode and safe `fallback_reason` via `get_storage_info`.

## 6. Reminder & Updater Architecture

### Reminders
- Rules: one day before (in business-local time), one hour before, and once on the appointment's business-local calendar day upon first app launch that day.
- Delivery state is persisted in SQLite (`reminders`, `notifications`, `reminder_daily_runs`) to prevent duplicates across restarts. Rescheduling recalculates pending reminders; cancellation/completion invalidates them.
- **Closed-app limitation**: While the application process is completely terminated, it cannot deliver reminders unless a separate OS scheduled task or background helper is installed. On next launch, startup catch-up processes due reminders with expiry rules to avoid delivering stale alerts.

### Updates
- Targets GitHub Releases at `MichaelX17/My-Schedule-App` using a single signed `latest.json` + `latest.json.sig` manifest covering both `windows-x86_64` (`.exe`/`.zip`) and `linux-x86_64` (`.AppImage`).
- Enforces Ed25519 signature verification against the compile-time `MY_SCHEDULE_UPDATE_PUBKEY`, SemVer downgrade rejection, SHA-256 verification of the downloaded artifact, pre-update SQLite safety backup, atomic `.part` staging, and platform-specific swap scripts (`apply-update.ps1` / `apply-update.sh`) that preserve `data/` and back up the prior executable for rollback.
