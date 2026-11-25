# Step 04 — Service catalog

## Objective

Implement the business's offered services without pricing/payment functionality.

## Tasks

1. Design service schema: stable ID, name, optional description, default duration, active/archive state, timestamps.
2. Add migration, Rust repository/service/commands, typed frontend wrappers.
3. Implement list, search, create, edit, activate/deactivate, and archive.
4. Validate names and duration ranges in Rust.
5. Ensure inactive/archived services are excluded from new appointment selection by default.
6. Preserve historical service details later through appointment-service snapshots; design the schema/API now so Step 05 can capture the service name and relevant attributes at booking time.
7. Avoid pricing, revenue, invoice, tax, or payment fields.
8. Add tests for duration validation, active filtering, update/archive behavior, and persistence.
9. Update `Progress.md`.

## Acceptance criteria

- Service catalog is fully persisted and manageable.
- Inactive services remain retrievable for history.
- No financial fields or workflows exist.
- Tests/build pass.

## Stop condition

Stop after Step 04.
