# Windows Release & Portable Packaging Guide — My Schedule App

This guide documents how to build, package, distribute, and verify **My Schedule App** for Windows (`x86_64-pc-windows-msvc`).

## 1. Release Artifacts

1. **Primary artifact — Portable ZIP (`MyScheduleApp-vX.Y.Z-windows-x64-portable.zip` + `.sha256`)**
   - Built via `scripts/package-portable-windows.ps1`.
   - Contains `MyScheduleApp.exe` and `PORTABLE_README.txt`.
   - **Never** contains a `data/` directory, so extracting a newer version over an existing portable folder preserves `data/database.sqlite`, backups, exports, logos, and logs.
2. **Optional installer — NSIS Per-User Installer (`My Schedule App_X.Y.Z_x64-setup.exe`)**
   - Built via `npm run tauri build -- --bundles nsis`.
   - Configured in `src-tauri/tauri.conf.json` with `installMode: "currentUser"` (no administrator elevation required).

## 2. Windows Build Prerequisites

To build Windows release artifacts on a Windows 10/11 (x64) machine:

1. **Node.js** v20+ and **npm**
2. **Rust stable** (`x86_64-pc-windows-msvc` target via `rustup`)
3. **Microsoft C++ Build Tools** (Visual Studio 2022 Build Tools with "Desktop development with C++" workload)
4. **WebView2 Runtime** (pre-installed on Windows 11 and modern Windows 10)

## 3. Reproducible Build Commands

### Portable ZIP (primary)

```powershell
pwsh -File scripts/package-portable-windows.ps1
```

Outputs:
- `release-artifacts/MyScheduleApp-v<version>-windows-x64-portable.zip`
- `release-artifacts/MyScheduleApp-v<version>-windows-x64-portable.zip.sha256`

### NSIS Per-User Installer (optional)

```powershell
npm ci
npm run tauri build -- --bundles nsis
```

Outputs appear under `src-tauri/target/release/bundle/nsis/`.

## 4. Portable Storage, Fallback, and Temp-Directory Protection

At startup (`src-tauri/src/platform/mod.rs`), My Schedule App resolves storage in this order:

1. **Temporary extraction guard:** If the executable resides inside the OS temporary directory (e.g., launched directly from a Windows Explorer ZIP preview under `%TEMP%`), portable mode is refused so user data is never placed in a volatile folder.
2. **Adjacent portable mode (`<exe_dir>\data\`):** Verified with a real create/write/flush/delete probe. Creates:
   - `data\database.sqlite`
   - `data\backups\automatic\`
   - `data\backups\manual\`
   - `data\exports\`
   - `data\logos\`
   - `data\logs\`
   - `data\updates\`
3. **Per-user fallback (`%APPDATA%\com.myscheduleapp.app\data\`):** Used automatically if `<exe_dir>` is read-only (e.g., `C:\Program Files\...`) or inside `%TEMP%`. Settings → Storage Diagnostics clearly reports the active mode and fallback reason.

## 5. Runtime Prerequisites & WebView2

- `src-tauri/tauri.conf.json` sets `bundle.windows.webviewInstallMode.type = "downloadBootstrapper"`.
- On Windows 11 and updated Windows 10 systems, the Evergreen WebView2 Runtime is already present and the app runs fully offline.
- On older Windows 10 machines without WebView2, users can install the official Microsoft Edge WebView2 Evergreen Standalone Installer offline before first launch.

## 6. Code Signing & SmartScreen

Unsigned executables may trigger a Windows Defender SmartScreen prompt on first launch. For public production releases, sign `MyScheduleApp.exe` and the NSIS installer with an Authenticode code-signing certificate (e.g., EV or Trusted Signing) before generating `.sha256` checksums.

## 7. Clean Windows Verification Checklist

The automated test suite (`cargo test`, including `tests/packaging.rs` and `tests/foundation.rs`) verifies portable write-probing, read-only fallback, temp-directory protection, non-ASCII paths, upgrade-over-existing-data preservation, and backup/restore on Linux.

Before publishing a Windows release candidate, run the following manual verification on a clean Windows 10/11 machine (cannot be executed on a Linux development host):

- [ ] Build portable ZIP via `scripts/package-portable-windows.ps1` and verify SHA-256 checksum.
- [ ] Extract to a writable path containing non-ASCII characters (e.g., `C:\Users\<user>\Desktop\Clínica Café\MyScheduleApp\`) and verify Portable mode in Settings.
- [ ] Complete onboarding, create clients/services/appointments, restart app, and confirm persistence offline (airplane mode / network disconnected).
- [ ] Copy executable to a read-only directory (e.g., `C:\Program Files\MyScheduleApp\`) and confirm automatic per-user fallback + clear reason in Settings.
- [ ] Verify Windows toast notification delivery and in-app fallback when Windows notifications are disabled.
- [ ] Replace `MyScheduleApp.exe` with a rebuilt binary over an existing `data/` folder and confirm zero data loss.
