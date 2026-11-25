# Step 07 — Reminder engine and notifications

## Objective

Implement persistent reminder scheduling, duplicate prevention, in-app notifications, and Windows system notifications.

## Reminder rules

- One day before appointment.
- One hour before appointment.
- On the appointment's business-local calendar day, once when the app is first launched that day.

Document the precise local-time rule for the one-day-before reminder (for example, same local clock time on the preceding calendar date) and make it consistent.

## Tasks

1. Design persistent reminder and notification-queue tables with unique/idempotency keys.
2. Implement reminder generation/update/cancellation when appointments are created, edited, rescheduled, completed, or cancelled.
3. Implement a Rust scheduler service that runs while the app process is active.
4. On startup, perform catch-up processing for due reminders. Apply expiry rules so old reminders do not arrive unexpectedly.
5. Implement the day-of first-launch rule using a persisted business-local date marker so it occurs once per local day.
6. Implement in-app notification center/toasts/modal behavior with read/unread or dismissed state.
7. Integrate Windows system notifications using a supported Tauri 2/plugin/platform approach. Verify the exact current API and required capabilities.
8. Handle permission denial, unsupported platform, and notification delivery errors gracefully.
9. Ensure cancelled/completed appointments do not generate reminders.
10. Prevent duplicates across restarts, clock changes, and repeated scheduler ticks.
11. Log operational outcomes without logging client notes or unnecessary personal data.
12. Add tests for due-time calculation, daylight-saving transitions, rescheduling, cancellation, stale reminders, duplicate prevention, catch-up, and day-of once-per-day behavior.
13. Clearly disclose in the UI/help documentation: when the app process is fully terminated, reminders cannot be delivered unless a separate OS task/helper is configured. Do not claim background reminders work after exit.
14. Update `Progress.md`.

## Acceptance criteria

- Reminder queue persists.
- All three reminder rules work according to documented policy.
- No duplicate or cancelled-appointment notifications.
- In-app notifications work.
- Windows notification integration is verified or its blocker is clearly recorded.
- Closed-app limitations are accurately communicated.
- Tests/build pass.

## Stop condition

Stop after Step 07.
