# Step 13 — End-to-end hardening, QA, documentation, and release candidate

## Objective

Perform a systematic final review across all implemented features and prepare a release candidate.

## Tasks

1. Read all of `Main.md`, `Progress.md`, and prior step notes. Inspect the actual repository and Git history.
2. Create an end-to-end test matrix covering:
   - first launch/onboarding;
   - business profile edit;
   - client create/edit/archive/restore;
   - service create/edit/deactivate;
   - independent appointment;
   - plan-linked appointment;
   - inline client creation;
   - multi-service appointment;
   - reschedule/cancel/complete;
   - all calendar views;
   - all reminder types;
   - startup catch-up and duplicate prevention;
   - weekly/monthly reports;
   - exports;
   - manual/automatic backup and restore;
   - theme/timezone/preferences;
   - portable path and fallback;
   - update check, verification, install, rollback.
3. Run the full Rust and frontend test suites.
4. Run formatters, lint, type checks, clippy, and production builds.
5. Fix defects within scope. Do not hide flaky tests or weaken assertions merely to obtain a green run.
6. Audit database migrations for data loss, missing indexes, foreign-key issues, and rollback/recovery limitations.
7. Audit timezone and daylight-saving behavior.
8. Audit reminder idempotency and stale notification handling.
9. Audit error handling, logs, privacy, file permissions, and Tauri capabilities.
10. Audit UI responsiveness, keyboard navigation, accessible labels, contrast, loading/empty/error states, and destructive confirmations.
11. Remove unused dependencies, dead code, sample data, debug-only controls, and misleading placeholders.
12. Verify all user-facing claims against actual implementation. Document limitations honestly.
13. Complete:
    - README;
    - architecture documentation;
    - build/release instructions;
    - backup/restore guide;
    - update/recovery guide;
    - known limitations and troubleshooting.
14. Produce a release-candidate checklist and record actual tested OS/toolchain versions.
15. Update `Progress.md` with the final status, test matrix, build artifacts, and unresolved issues.

## Acceptance criteria

- All core end-to-end workflows pass.
- No known critical data-loss, duplicate-reminder, or update-integrity defects remain.
- Full test/build checks pass, or failures and reasons are documented.
- User documentation matches the real product.
- Release limitations are explicit.
- Progress log gives a complete handoff/release record.

## Stop condition

Stop after Step 13 and provide a concise release-candidate report. Do not publish a GitHub release or distribute binaries unless the user explicitly authorizes it.
