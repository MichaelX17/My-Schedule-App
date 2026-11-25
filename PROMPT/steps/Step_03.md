# Step 03 — Client management

## Objective

Implement client persistence and complete client CRUD/archive workflows.

## Tasks

1. Design a client schema with stable ID, required display/name fields, optional contact details, optional general notes, archive state, and created/updated timestamps.
2. Add migration, repository, service, typed Tauri commands, and frontend IPC wrappers.
3. Validate and normalize input in Rust. Treat email/phone as optional strings unless a well-justified validation rule applies; do not reject legitimate international formats.
4. Implement client list with search, sensible sorting, pagination or virtualization if needed, and clear empty/loading/error states.
5. Implement create, detail, edit, archive, and restore workflows.
6. Prevent ordinary hard deletion when appointment/session history depends on a client. Use archive semantics.
7. Provide a client detail view with an initially empty history section that can be wired to appointments in Step 05.
8. Add confirmation for archive/restore where appropriate. Do not make archive appear equivalent to permanent deletion.
9. Ensure archived clients remain available in historical appointment views but are excluded from default new-appointment selection.
10. Add Rust repository/service tests and frontend tests for validation and core interactions.
11. Update `Progress.md`.

## Constraints

- Do not add sensitive medical, financial, identity-document, or other regulated client fields.
- Do not implement appointments in this step.
- Do not use mock clients as if they were persisted data.

## Acceptance criteria

- Client create/read/update/archive/restore works against SQLite.
- Search and filtering work.
- Existing records survive restart.
- Errors are handled and shown clearly.
- Tests/build pass.

## Stop condition

Stop after Step 03.
