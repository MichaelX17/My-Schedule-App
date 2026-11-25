# Step 06 — Calendar views and navigation

## Objective

Implement the day, week, month, and agenda/list calendar experience using persisted appointments.

## Tasks

1. Select/configure the calendar library compatible with the existing React/Tauri stack. Avoid unnecessary dependency churn.
2. Implement typed backend queries for a date range, returning appointment data needed for rendering.
3. Implement day, week, month, and agenda/list views.
4. Provide previous/next navigation, today shortcut, date selection, and clear current-range heading.
5. Render appointment status and service/client context with accessible labels; do not rely on color alone.
6. Clicking an appointment opens its detail/edit workflow.
7. Support narrow desktop windows by using compact layouts and agenda/list as a practical fallback.
8. Ensure date ranges are computed in the business timezone and correctly translated to UTC query boundaries.
9. Avoid implementing drag-and-drop rescheduling unless it can be made reliable and keyboard-accessible; a deliberate edit/reschedule dialog is acceptable.
10. Add tests for date-range conversion, boundary appointments, timezone transitions, and view navigation.
11. Update `Progress.md`.

## Acceptance criteria

- All four views work.
- Data is loaded from SQLite through Rust commands.
- Appointment selection opens the correct record.
- Timezone boundaries are correct.
- Responsive behavior is usable.
- Tests/build pass.

## Stop condition

Stop after Step 06.
