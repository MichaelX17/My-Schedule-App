# Step 12 — GitHub updater and safe update/migration flow

## Objective

Implement a secure, recoverable update mechanism for the actual distribution format.

## Tasks

1. Inspect the selected Windows release artifact and decide whether the standard Tauri updater is suitable. Do not assume it can safely replace a portable standalone executable.
2. Document the chosen update architecture before implementation:
   - release metadata source;
   - version comparison;
   - artifact selection;
   - signature/hash verification;
   - staging;
   - process shutdown/replacement;
   - restart;
   - rollback/recovery.
3. Use signed update metadata/artifacts where supported. Never rely only on an unsigned hash fetched from the same untrusted location.
4. Keep signing private keys out of source control and logs. Use environment/CI secrets for release signing.
5. Implement update check and user-facing version information.
6. Implement user-consented download and installation. Do not silently replace the executable without an explicit product policy.
7. Before a potentially schema-changing update, create and validate a database backup.
8. Ensure migrations run only after the new executable is staged and verified, with a recovery plan if migration fails.
9. Protect user data and configuration during executable replacement.
10. Handle offline mode, rate limits, missing release metadata, invalid signatures, interrupted downloads, and failed restarts.
11. Test version ordering, invalid metadata/signatures, interrupted staging, rollback, and migration failure.
12. Document GitHub release publishing and signing procedures.
13. Update `Progress.md`.

## Acceptance criteria

- Update checks work against the configured GitHub release source.
- Artifacts are verified before execution/replacement.
- User data is backed up before risky migrations.
- Failure does not leave the app unusable or destroy data.
- Offline behavior is graceful.
- Security-sensitive signing material is not committed.
- Tests/build pass.

## Stop condition

Stop after Step 12.
