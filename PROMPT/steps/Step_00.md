# Step 00 — Repository audit and engineering baseline

## Objective

Inspect the existing project scaffold, verify that it is a valid Tauri 2 + Rust + React + TypeScript foundation, establish a reliable baseline, and document any gaps. Do not implement business functionality in this step.

## Read first

- `Main.md`
- `Progress.md`

## Tasks

1. Inspect the entire repository structure. Identify the frontend root, Tauri/Rust root, package manager, scripts, configuration, capabilities, and current Git state.
2. Read `package.json`, lockfiles, `Cargo.toml`, `tauri.conf.json`, capability files, TypeScript/Vite/Tailwind configuration, and README.
3. Confirm the project uses Tauri 2 conventions. Do not copy Tauri 1 APIs or obsolete configuration examples.
4. Check for duplicate or conflicting app entry points, stale templates, unused dependencies, and inconsistent naming.
5. Verify the Rust and Node toolchain versions available in the environment. Record versions; do not install system software without permission.
6. Run the existing validation commands that are applicable:
   - frontend install only if dependencies are missing and the package manager is clear;
   - typecheck;
   - lint;
   - frontend build;
   - `cargo fmt --check`;
   - `cargo check`;
   - `cargo clippy --all-targets --all-features` where supported.
7. Fix only baseline defects that prevent the scaffold from building or that are clearly within the empty-project foundation. Do not add product features.
8. Verify that the app can launch in development mode if the environment supports the required Windows/WebView2/Tauri prerequisites. If not, record the exact blocker.
9. Review Tauri capabilities and ensure permissions are minimal. Do not grant broad filesystem, shell, or HTTP access preemptively.
10. Create or improve `ARCHITECTURE.md` with:
    - frontend/backend boundary;
    - IPC-only data access principle;
    - planned Rust module boundaries;
    - planned frontend feature boundaries;
    - database ownership and migration strategy;
    - portable storage strategy;
    - reminder/updater constraints.
11. Improve README with accurate prerequisites and actual commands. Do not document commands that do not exist.
12. Confirm the prompt pack is not accidentally included in app build output. Keep prompt documents in a repository location excluded from packaging if appropriate.
13. Update `Progress.md` with actual findings and validation results.

## Explicit exclusions

Do not create:
- SQLite business tables;
- client/service/appointment models;
- business onboarding;
- dashboard;
- calendar;
- notifications/reminders;
- reports;
- backup/restore;
- updater functionality;
- authentication;
- payments.

## Acceptance criteria

- Repository structure is understood and documented.
- Tauri 2 configuration and entry points are coherent.
- Frontend and Rust baseline checks pass, or exact environment blockers are documented.
- README and `ARCHITECTURE.md` accurately describe the current scaffold and planned architecture.
- No business functionality has been implemented.
- `Progress.md` records files changed, tool versions, commands, exact results, and the next step.

## Stop condition

After updating `Progress.md`, stop. Do not begin Step 01 without explicit user authorization.
