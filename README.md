# My Schedule App

A Windows-first, offline-first, single-operator desktop application for managing a business's clients, services, appointments, optional session plans, reminders, operational reports, exports, backups, and settings. Built with **Tauri 2**, **Rust**, **React**, **TypeScript**, and **SQLite**.

> **Status:** Release Candidate (Steps 00–13 complete and verified on Linux; native Windows `.exe` packaging and clean-machine Windows 10/11 smoke test documented in [docs/WINDOWS_RELEASE.md](./docs/WINDOWS_RELEASE.md)).

## Key Capabilities

- **Single-Business, Offline-First:** No logins, no cloud accounts, no internet required for core scheduling workflows.
- **Non-Financial Operational Scope:** Manages business profile, clients (with archive/restore), services (non-priced, with historical appointment snapshots), individual appointments, and optional session plans.
- **Calendar & Timezone Safety:** Day, Week, Month, and Agenda views in the business's IANA timezone, with UTC storage and explicit DST gap/ambiguity handling.
- **Persistent Reminders:** One day before, one hour before, and once-per-day on-launch reminders persisted in SQLite to prevent duplicates across restarts.
- **Operational Reports & Exports:** Weekly/monthly non-financial reports, versioned JSON export, and RFC 4180 CSV exports (`clients`, `services`, `appointments`, `appointment_services`).
- **SQLite-Safe Backups & Restore:** Automatic backups (default 30 retained, configurable) and manual backups via `VACUUM INTO`, integrity/schema validation, and atomic restore with pre-restore safety backups.
- **Cross-Platform Signed GitHub Updates:** Supports both Windows (`.exe` / `.zip`) and Linux (`.AppImage`) releases from [`MichaelX17/My-Schedule-App`](https://github.com/MichaelX17/My-Schedule-App) with Ed25519 manifest signatures, SHA-256 verification, pre-update backups, and rollback.

## Technology Stack

### Backend (`src-tauri/`)
- **Rust** (edition 2021) & **Tauri 2** — Desktop shell, lifecycle, OS notifications, and typed IPC
- **Tokio** — Asynchronous runtime
- **SQLx (SQLite)** — Local persistence, WAL mode, foreign-key enforcement, and 7 source-controlled migrations
- **Chrono & chrono-tz** — UTC instant storage, IANA timezone conversions, and DST handling
- **ed25519-dalek, sha2, semver, reqwest (`rustls-tls`)** — Signed GitHub Releases updater
- **Tracing & Thiserror** — Structured logging and typed error boundaries

### Frontend (`src/`)
- **React 19**, **TypeScript** (strict mode), **Vite**, **Tailwind CSS v4**, **shadcn/ui**, **Lucide React**
- **React Router**, **TanStack Query**, **Zustand**, **React Hook Form**, **Zod**, **Vitest**

## Prerequisites

- **Node.js** (v20+ recommended) and **npm**
- **Rust** stable toolchain (`rustup`, `rustc`, `cargo`, `rustfmt`, `clippy`)
- **Tauri 2 system prerequisites**:
  - **Windows** (primary target): Microsoft C++ Build Tools and WebView2 Runtime
  - **Linux** (development & `.AppImage` target): `webkit2gtk-4.1`, `gtk+-3.0`, `pkg-config`, and C/C++ build tools (`build-essential`)

## Setup & Development

```bash
# Install frontend dependencies
npm install

# Run frontend dev server only (http://localhost:1420)
npm run dev

# Run full Tauri desktop app in development mode
npm run tauri dev
```

## Verification & Build Commands

```bash
# Frontend
npm run typecheck
npm run lint
npm run test
npm run build

# Rust backend
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## Storage, Backup, and Restore Guide

- **Portable Storage:** My Schedule App checks whether `<executable-dir>/data/` is writable (and not a temporary extraction directory like `%TEMP%`). If writable, it stores all data in that adjacent `data/` folder (`database.sqlite`, `backups/`, `exports/`, `logos/`, `logs/`, `updates/`). If the executable directory is read-only (e.g., `Program Files`), it falls back to the OS per-user application data directory and displays the active storage mode in **Settings**.
- **Automatic & Manual Backups:** Open **Backups & Exports** (`/backups`) to create a manual backup, inspect automatic backups, or adjust retention (default 30) in **Settings**.
- **Safe Restore:** Restoring a backup validates SQLite integrity and schema compatibility, automatically creates a pre-restore safety backup of the current database, checkpoints WAL sidecars, atomically replaces `database.sqlite`, and rolls back if reopening fails.
- **Important:** Never manually delete `-wal` or `-shm` files while the application is running, and never back up a live database by copying only `database.sqlite` from File Explorer.

## Updates & Recovery Guide

- Updates are checked on demand in **Settings → Application Updates** against `https://github.com/MichaelX17/My-Schedule-App`.
- Both Windows (`.exe` / `.zip`) and Linux (`.AppImage`) artifacts are supported in a single signed `latest.json` + `latest.json.sig` release manifest.
- Before applying an update, the app verifies the Ed25519 signature and SHA-256 digest, creates a pre-update SQLite safety backup in `data/backups/safety/`, stages the new binary in `data/updates/`, and generates a platform-specific swap script (`apply-update.ps1` on Windows, `apply-update.sh` on Linux) that backs up the previous executable and preserves `data/`.
- See [docs/GITHUB_UPDATES.md](./docs/GITHUB_UPDATES.md) for the exact manifest schema, signing workflow, and manual recovery steps.

## Known Limitations & Troubleshooting

- **Closed-App Reminders:** When the application process is completely terminated, it cannot deliver desktop reminders (no background OS daemon or system tray service is installed in v0.1). On the next launch, startup catch-up evaluates due reminders and avoids firing stale past alerts.
- **Recurring Appointments:** Not included in v0.1; use optional **Session Plans** to group individual scheduled appointments for a client.
- **Unsigned Update Builds:** Development builds compiled without `MY_SCHEDULE_UPDATE_PUBKEY` intentionally report `SigningKeyUnconfigured` when checking for updates rather than accepting unverified binaries.
- **Offline Time Sync:** Online time checks are best-effort; when offline, the app operates normally using the local system clock.

## Documentation Index

- [ARCHITECTURE.md](./ARCHITECTURE.md) — Module boundaries, SQLite strategy, storage rules, and security boundaries
- [docs/WINDOWS_RELEASE.md](./docs/WINDOWS_RELEASE.md) — Windows portable ZIP & NSIS packaging, WebView2, and clean-machine checklist
- [docs/GITHUB_UPDATES.md](./docs/GITHUB_UPDATES.md) — Multi-platform (`Windows .exe` + `Linux .AppImage`) signed GitHub Releases specification
- [docs/RELEASE_CANDIDATE_CHECKLIST.md](./docs/RELEASE_CANDIDATE_CHECKLIST.md) — Full 18-item end-to-end test matrix, tested toolchains, and pre-release operator checklist
