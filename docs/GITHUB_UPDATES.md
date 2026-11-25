# GitHub Release Updater Architecture & Publishing Guide — My Schedule App

**Configured Release Repository:** `https://github.com/MichaelX17/My-Schedule-App`

## 1. Cross-Platform Portable Update Design (Windows `.exe` / `.zip` + Linux `.AppImage`)

My Schedule App supports **both Windows and Linux** in a single GitHub Release using a Rust-owned, cryptographically verified portable updater (`src-tauri/src/updater/mod.rs`).

Why a custom portable updater instead of standard `tauri-plugin-updater`?
- Standard `tauri-plugin-updater` targets OS installers (MSI/NSIS) and does not safely swap standalone portable `.exe` or `.AppImage` binaries living beside a `data/` directory, nor does it automatically create a SQLite `VACUUM INTO` safety backup before replacing the binary.

## 2. How Updates Work Step-by-Step

1. **Check for Updates (Read-Only):**
   - The app detects its current platform key (`windows-x86_64` on Windows, `linux-x86_64` on Linux).
   - It fetches `latest.json` and `latest.json.sig` over HTTPS from:
     - `https://github.com/MichaelX17/My-Schedule-App/releases/latest/download/latest.json`
     - `https://github.com/MichaelX17/My-Schedule-App/releases/latest/download/latest.json.sig`
2. **Ed25519 Signature Verification (First!):**
   - Verifies `latest.json.sig` against the trusted Ed25519 public key embedded via `MY_SCHEDULE_UPDATE_PUBKEY` **before** trusting any version, URL, or SHA-256 hash inside `latest.json`.
   - Rejects tampered manifests, missing signatures, non-HTTPS URLs, or non-repo URLs.
3. **Version & Platform Selection:**
   - Compares `manifest.version` against the running version using strict `semver` (rejects downgrades).
   - Selects the matching entry from `manifest.platforms["windows-x86_64"]` (`.exe` or `.zip`) or `manifest.platforms["linux-x86_64"]` (`.AppImage`).
4. **User-Consented Download, SHA-256 Check & Safety Backup:**
   - Nothing is downloaded or replaced without explicit user action in **Settings → GitHub Releases & Updates**.
   - On click of **Back Up & Stage**:
     1. Downloads the platform artifact over HTTPS.
     2. Verifies its **SHA-256** digest against the Ed25519-signed `sha256` field.
     3. Creates and validates a pre-update SQLite `VACUUM INTO` **safety backup** in `data/backups/manual/safety_*.sqlite`.
     4. Atomically stages the verified binary inside `data/updates/staged/v<version>/` and writes the OS-specific apply/rollback helper:
        - **Windows:** `apply-update.ps1` (replaces `MyScheduleApp.exe`, preserves `data\`, rolls back automatically on failure)
        - **Linux:** `apply-update.sh` (replaces `MyScheduleApp.AppImage`, `chmod +x`, preserves `data/`, rolls back automatically on failure)
5. **Migration Safety:**
   - Database migrations run only when the new binary launches; if a migration fails, SQLx rolls back the transaction and the pre-update safety backup remains intact in `data/backups/manual/`.

## 3. Multi-Platform Signed Manifest Schema (`latest.json`)

```json
{
  "version": "0.2.0",
  "pub_date": "2026-12-01T12:00:00Z",
  "notes": "Bug fixes and improvements.",
  "target_schema_version": 7,
  "platforms": {
    "windows-x86_64": {
      "artifact_filename": "MyScheduleApp-v0.2.0-windows-x64.exe",
      "download_url": "https://github.com/MichaelX17/My-Schedule-App/releases/download/v0.2.0/MyScheduleApp-v0.2.0-windows-x64.exe",
      "sha256": "<64-char-lowercase-hex-sha256>"
    },
    "linux-x86_64": {
      "artifact_filename": "MyScheduleApp-v0.2.0-linux-x86_64.AppImage",
      "download_url": "https://github.com/MichaelX17/My-Schedule-App/releases/download/v0.2.0/MyScheduleApp-v0.2.0-linux-x86_64.AppImage",
      "sha256": "<64-char-lowercase-hex-sha256>"
    }
  }
}
```

## 4. Publishing a Signed Windows + Linux Release

> **Security Rule:** Never commit the Ed25519 private signing key to Git or log it in CI output. Store it only in GitHub Actions Secrets (`UPDATE_SIGNING_PRIVATE_KEY`) and inject the base64 public key into builds via `MY_SCHEDULE_UPDATE_PUBKEY`.

1. Build Windows `.exe` (or portable `.zip`) and Linux `.AppImage` (`npm run tauri build -- --bundles appimage` on Linux, `scripts/package-portable-windows.ps1` on Windows).
2. Compute SHA-256 hashes for both artifacts and write `latest.json` using the schema above.
3. Sign the exact bytes of `latest.json` with your Ed25519 private key and base64-encode the 64-byte signature into `latest.json.sig`.
4. Upload all 4 files to a GitHub Release on `https://github.com/MichaelX17/My-Schedule-App/releases`:
   - `MyScheduleApp-v<ver>-windows-x64.exe` (or `.zip`)
   - `MyScheduleApp-v<ver>-linux-x86_64.AppImage`
   - `latest.json`
   - `latest.json.sig`
