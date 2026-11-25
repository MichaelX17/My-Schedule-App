# Release Candidate Checklist & End-to-End Test Matrix

This document records the release-candidate readiness verification for **My Schedule App** (Steps 00–13), distinguishing automated verification executed on this development host from manual verification that requires a native Windows 10/11 target machine or release signing keys.

## 1. Tested Host & Toolchain Versions

- **Host OS:** Linux (`x86_64-unknown-linux-gnu`)
- **Node.js:** `v26.13.0`
- **npm:** `12.2.1`
- **Rust (`rustc`):** `1.99.1 (58cf8f9c3 2026-10-13)`
- **Cargo (`cargo`):** `1.99.1 (5f94df478 2026-08-27)`
- **Target GitHub Releases Repository:** `https://github.com/MichaelX17/My-Schedule-App`

---

## 2. End-to-End Verification Matrix

| # | Scenario | Automated Coverage | Status on Linux Host | Manual Windows Check |
|---|---|---|---|---|
| 1 | Fresh launch and onboarding | `src-tauri/tests/business_profile.rs`, `src-tauri/tests/e2e_release_candidate.rs` | Verified | Required on clean Windows 10/11 |
| 2 | Business profile edit (including logo) | `src-tauri/tests/business_profile.rs`, `src-tauri/tests/e2e_release_candidate.rs` | Verified | Recommended |
| 3 | Client create, edit, archive, restore | `src-tauri/tests/clients.rs`, `src-tauri/tests/e2e_release_candidate.rs`, `src/features/clients/__tests__/ClientFormDialog.test.tsx`, `src/features/clients/__tests__/clientValidation.test.ts` | Verified | — |
| 4 | Service create, edit, deactivate, snapshot preservation | `src-tauri/tests/services.rs`, `src-tauri/tests/appointments.rs`, `src-tauri/tests/e2e_release_candidate.rs`, `src/features/services/__tests__/ServiceFormDialog.test.tsx`, `src/features/services/__tests__/serviceValidation.test.ts` | Verified | — |
| 5 | Appointment create (existing client, inline client, multi-service, session plan) | `src-tauri/tests/appointments.rs`, `src-tauri/tests/e2e_release_candidate.rs`, `src/features/appointments/__tests__/AppointmentFormDialog.test.tsx` | Verified | — |
| 6 | Appointment reschedule, complete, cancel, terminal transition guards | `src-tauri/tests/appointments.rs`, `src-tauri/tests/e2e_release_candidate.rs` | Verified | — |
| 7 | Calendar Day, Week, Month, Agenda views & DST boundaries | `src-tauri/tests/calendar.rs`, `src-tauri/tests/e2e_release_candidate.rs`, `src/features/calendar/__tests__/CalendarPage.test.tsx`, `src/features/calendar/__tests__/calendarDate.test.ts` | Verified | Visual smoke check |
| 8 | Reminder creation, startup catch-up (`day_of_launch` once/day), reschedule, cancel, duplicate prevention | `src-tauri/tests/reminders.rs`, `src-tauri/tests/e2e_release_candidate.rs`, `src/features/reminders/__tests__/RemindersPage.test.tsx` | Verified (in-app + `NoopNotificationSender`) | Live Windows toast delivery check |
| 9 | Dashboard summary and weekly/monthly operational reports (non-financial) | `src-tauri/tests/reports.rs`, `src-tauri/tests/e2e_release_candidate.rs`, `src/features/reports/__tests__/ReportsPage.test.tsx` | Verified | — |
| 10 | JSON versioned export and CSV exports (`clients`, `services`, `appointments`, `appointment_services`) | `src-tauri/tests/backups_and_exports.rs`, `src-tauri/tests/e2e_release_candidate.rs`, `src/features/backups/__tests__/BackupsExportsPage.test.tsx` | Verified | — |
| 11 | Manual & automatic backup, integrity validation, restore with pre-restore safety backup | `src-tauri/tests/backups_and_exports.rs`, `src-tauri/tests/e2e_release_candidate.rs` | Verified | — |
| 12 | Theme (System/Light/Dark), accent color presets, retention settings across restart | `src-tauri/tests/settings.rs`, `src-tauri/tests/e2e_release_candidate.rs`, `src/features/settings/__tests__/SettingsPage.test.tsx` | Verified | Visual check in Windows light/dark |
| 13 | Business timezone change preserving UTC instants & recalculating reminders | `src-tauri/tests/settings.rs`, `src-tauri/tests/e2e_release_candidate.rs` | Verified | — |
| 14 | Offline operation (app core workflows, time fallback, update check offline handling) | `src-tauri/tests/foundation.rs`, `src-tauri/tests/updater.rs`, all offline DB tests | Verified | Airplane-mode smoke check on Windows |
| 15 | Portable mode (writable adjacent `data/`), temp-dir refusal, read-only fallback | `src-tauri/tests/foundation.rs`, `src-tauri/tests/packaging.rs` | Verified | Verify in writable folder + `Program Files` |
| 16 | Corrupted database / invalid backup / tampered update rejection | `src-tauri/tests/foundation.rs`, `src-tauri/tests/backups_and_exports.rs`, `src-tauri/tests/updater.rs` | Verified | — |
| 17 | Multi-platform signed update staging (Windows `.exe` + Linux `.AppImage`) & rollback | `src-tauri/tests/updater.rs`, `src-tauri/tests/e2e_release_candidate.rs` | Verified | Live GitHub release test after publishing |
| 18 | Build isolation & least-privilege security hygiene | `src-tauri/capabilities/default.json` (`core:default` only), `dist/` excludes `PROMPT/`, zero PII/secrets in `tracing` logs | Verified | — |

---

## 3. Security, Privacy, and Quality Audit Summary

- **Tauri Capabilities:** `src-tauri/capabilities/default.json` grants only `core:default`. All filesystem, SQLite, OS notification, backup, export, and HTTPS update operations execute in Rust.
- **No Unwanted Scope:** Zero login/password/auth code, zero payments/prices/invoices/revenue, zero medical/regulated fields, and zero hardcoded production sample data.
- **Logging Hygiene:** Audited all `tracing::` statements across `src-tauri/src/` — no client notes, emails, phone numbers, or signing keys are logged.
- **Build Isolation:** `npm run build` emits only compiled `src/` assets into `dist/`; `PROMPT/` planning files are excluded from `dist/` and `src-tauri/tauri.conf.json`.
- **Database Safety:** All 7 migrations (`0001`–`0007`) are additive, non-destructive, enforce `PRAGMA foreign_keys = ON`, and use WAL mode with `VACUUM INTO` backups and `PRAGMA wal_checkpoint(TRUNCATE)` before restore swaps.

---

## 4. Remaining Pre-Release Operator Actions (Blocked on Windows Host / Release Keys)

These items cannot be completed from this Linux development environment and must be executed before publishing a public `v0.1.0` release:

1. **Generate an Ed25519 Release Signing Keypair:**
   - Keep the private key offline or in CI secrets (`MY_SCHEDULE_UPDATE_PRIVKEY`).
   - Compile release builds with `MY_SCHEDULE_UPDATE_PUBKEY=<base64-public-key>` (see [docs/GITHUB_UPDATES.md](./GITHUB_UPDATES.md)).
2. **Build Native Windows Artifacts on Windows 10/11:**
   - Run `scripts/package-portable-windows.ps1` on a Windows build machine with MSVC Build Tools and WebView2.
   - Optionally sign `MyScheduleApp.exe` / NSIS installer with an Authenticode certificate to minimize Windows SmartScreen warnings.
3. **Run Clean-Machine Windows 10/11 Smoke Test:**
   - Follow [docs/WINDOWS_RELEASE.md](./WINDOWS_RELEASE.md) to verify portable extraction, adjacent `data/` creation, read-only fallback, live OS notification toasts, and offline launch.
4. **Build Linux `.AppImage` & Publish Signed Release:**
   - Build the `.AppImage` on Linux (`npm run tauri build -- --bundles appimage`), generate `latest.json` + `latest.json.sig` with both `windows-x86_64` and `linux-x86_64` entries, and upload to `https://github.com/MichaelX17/My-Schedule-App/releases`.
