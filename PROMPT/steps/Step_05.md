# Step 05 — Appointments and optional session plans

## Objective

Implement the core appointment workflow, including inline client creation, multi-service appointments, and optional session plans.

## Data model requirements

Appointments are individual scheduled events. Session plans are optional groupings of appointments for one client.

Appointment should include:
- stable ID;
- client reference;
- start/end UTC instants;
- business timezone context or a clear rule for resolving it;
- status (`scheduled`, `completed`, `cancelled`; add `no_show` only if fully defined);
- optional notes;
- created/updated timestamps;
- optional session-plan reference.

Appointment services should be a join table and preserve service snapshots (at least service name and duration at booking time). Do not rely only on the current service row for historical display.

Session plan should include:
- stable ID;
- client reference;
- optional title/description;
- planned session count;
- status/archival semantics as needed;
- timestamps.

## Tasks

1. Review current migrations and design a normalized schema with appropriate foreign keys and indexes.
2. Implement migrations. Protect existing data; no destructive schema changes.
3. Implement Rust models, repositories, services, and typed commands for:
   - list/search/filter appointments by date/status/client;
   - get appointment details;
   - create/update appointment;
   - reschedule;
   - cancel;
   - complete;
   - create/update/archive session plan;
   - list appointments for a plan;
   - create a client inline as part of a safe workflow.
4. Use database transactions for appointment + service rows + reminder invalidation/update hooks.
5. Validate:
   - client exists and is selectable;
   - at least one service is selected if the product workflow requires it (otherwise document the chosen rule);
   - end is after start;
   - duration is within safe bounds;
   - session plan belongs to the same client;
   - service snapshots are captured;
   - no invalid status transitions.
6. Decide and document overlap behavior. Do not silently prohibit overlapping appointments unless explicitly required; at minimum detect and clearly warn.
7. Implement appointment list/detail/create/edit UI. The client selector must allow selecting an existing client or creating one inline without losing the appointment form state.
8. Allow one or more services per appointment.
9. Implement optional session-plan create/select workflow and show planned count vs linked appointments. Independent appointments must remain first-class.
10. Preserve history when client/service is archived.
11. Add tests for transactions, snapshots, time validation, status transitions, inline client creation, plan/client consistency, and rescheduling.
12. Update `Progress.md`.

## Time requirements

- Store instants in UTC.
- Use the business's IANA timezone for display and local-date operations.
- Handle daylight-saving gaps/ambiguities deliberately. Do not silently shift user-entered times without feedback.

## Acceptance criteria

- Complete appointment lifecycle works and persists.
- Multiple services per appointment work.
- Service snapshots remain stable after service rename/deactivation.
- Independent appointments and plan-linked appointments both work.
- Inline client creation works.
- Tests/build pass.

## Stop condition

Stop after Step 05.
