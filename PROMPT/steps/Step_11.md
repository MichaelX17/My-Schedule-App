# Step 11 — Portable Windows packaging and release readiness

## Objective

Prepare a reliable Windows-first distribution, including portable behavior and clean-machine verification.

## Tasks

1. Review current Tauri 2 Windows build configuration and packaging options.
2. Define release artifacts clearly (portable executable/package and any installer package if justified).
3. Implement portable storage behavior and fallback diagnostics end-to-end.
4. Verify writable adjacent folder behavior and protected-directory fallback.
5. Ensure user data is not stored inside temporary extraction directories or overwritten during app replacement.
6. Confirm database, backups, exports, logos, logs, and update staging directories are created safely.
7. Verify app icon, product name, version, publisher metadata where available, and Windows app identity requirements.
8. Document Windows prerequisites, including WebView2 behavior and any required runtime/build tools.
9. Test on a clean Windows environment or clearly state what could not be tested.
10. Test first launch, restart, upgrade over existing data, backup/restore, offline launch, and non-ASCII paths.
11. Document how to build release artifacts reproducibly.
12. Update `Progress.md`.

## Acceptance criteria

- Windows release artifacts build.
- Portable/fallback storage is verified.
- Existing data survives ordinary app upgrades.
- Prerequisites and distribution instructions are accurate.
- Any untested environment requirement is explicitly documented.

## Stop condition

Stop after Step 11.
