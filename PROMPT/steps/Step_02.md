# Step 02 — Business profile and first-run onboarding

## Objective

Implement the single-business profile, setup-completion state, and first-run wizard.

## Prerequisites

Steps 00–01 complete. Read `Main.md`, `Progress.md`, and inspect existing migrations before changing the schema.

## Data requirements

Create a business profile model suitable for one business per installation. Include:
- stable ID;
- required business name;
- optional logo reference;
- optional address/location and practical contact fields;
- IANA timezone identifier;
- created/updated timestamps.

Keep the schema extensible. Do not add payment, tax, medical, or regulated data fields.

## Tasks

1. Design and implement a migration for the business profile and setup metadata.
2. Enforce the one-business-per-installation invariant in the backend/database, not merely in the UI.
3. Add typed Rust repository/service/command layers to:
   - retrieve business profile;
   - create initial profile;
   - update profile;
   - determine whether onboarding is complete.
4. Validate required business name and timezone identifier in Rust. Frontend validation is additional, not a substitute.
5. Build a multi-step or clearly grouped first-run wizard:
   - business name;
   - optional logo;
   - optional location/contact details;
   - offered services may be deferred to Step 04; do not create a duplicate service subsystem here;
   - timezone selection/default detection with explicit display and ability to change.
6. Allow optional fields to be skipped. Do not trap the user in the wizard because optional details are absent.
7. On successful completion, route to a simple main-app shell/placeholder. Do not build the dashboard yet.
8. Add a Settings entry point or placeholder that will later allow business details to be edited.
9. Handle logo import safely: validate supported image types/size, copy into managed app storage, and store a relative managed reference rather than arbitrary external paths.
10. Add tests for one-business invariant, validation, onboarding state, update behavior, and persistence across restart.
11. Update `Progress.md`.

## Acceptance criteria

- Fresh install displays onboarding.
- Completed setup survives restart and does not display onboarding again.
- Business name is required; optional fields can be skipped.
- Business profile can be retrieved and edited through backend commands.
- Logo storage does not depend on the original source file remaining in place.
- Tests and build pass.

## Stop condition

Stop after Step 02. Do not start Step 03 without authorization.
