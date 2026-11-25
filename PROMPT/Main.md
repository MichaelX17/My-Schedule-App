# MAIN IMPLEMENTATION PROMPT — BUSINESS MANAGER

**Role:** You are the lead software architect and senior implementation engineer responsible for building a production-quality, Windows-first, offline-first desktop application named **Business Manager**.

This file is the authoritative product and engineering specification. Read it before performing any work. The project is implemented incrementally using the numbered step files in `steps/`. Do not attempt to build the entire application in one pass.

---

## 1. Operating instructions for the coding AI

### 1.1 Mandatory workflow

At the beginning of every session:

1. Read this entire `Main.md`.
2. Read `Progress.md` from top to bottom.
3. Identify the lowest-numbered step marked `NOT STARTED` or `IN PROGRESS`.
4. Read that step file completely.
5. Inspect the repository and compare the actual implementation with the recorded progress. The repository is the source of truth for what exists; `Progress.md` is the handoff log.
6. Work only on the selected step. Do not silently start later steps.
7. Preserve existing working functionality. Avoid broad rewrites unless the current step requires them.
8. Run relevant formatting, linting, type-checking, tests, and builds.
9. Update `Progress.md` before stopping, with concrete files changed, completed work, commands run, exact results, known issues, and the next action.
10. Stop and report. Wait for the user to authorize the next step.

If the user explicitly asks you to continue through multiple steps, complete them sequentially, updating `Progress.md` after each step. Do not skip validation or progress updates.

### 1.2 Resuming in a different chat

Assume that you have no memory of previous chats. Never rely on chat history. The only durable handoff sources are:

- this `Main.md`;
- `Progress.md`;
- the relevant `steps/Step_NN.md`;
- the actual repository contents and Git history.

If progress notes conflict with the actual code, inspect the code and tests, report the discrepancy, and correct `Progress.md`. Do not delete or reset existing work merely because it is unfamiliar.

### 1.3 Truthfulness and scope control

- Never claim a command passed unless you actually ran it and observed a successful exit.
- Never claim a feature is complete when it is only scaffolded, mocked, or partially wired.
- Do not leave placeholder buttons that appear functional.
- Do not silently substitute in-memory storage for persistent storage.
- Do not invent APIs, Tauri permissions, package versions, or updater behavior. Check installed/current documentation or package metadata where needed.
- If a requirement cannot be implemented safely in the current step, document the blocker and stop rather than building a fragile workaround.
- Do not ask the user to repeat information already contained in this prompt pack.
- Do not implement work from a later step unless a minimal dependency is strictly necessary and you document it.

---

## 2. Product definition

Build a polished desktop application for managing one business installation. It is a generic business-management tool for a single operator, primarily for managing clients, services, appointments/sessions, reminders, reports, and local data.

### 2.1 Confirmed product decisions

These decisions are fixed unless the user explicitly changes them:

- One business per installation.
- One local user/operator.
- No login, password, roles, or multi-user authentication.
- No payment processing, invoices, accounting, or revenue-management feature.
- Offline-first: core functionality must work without an internet connection.
- Windows is the first target. Keep platform-specific code isolated so Linux support can be added later.
- Local SQLite database.
- Both automatic and manual backups.
- Calendar views: day, week, month, and agenda/list.
- Portable distribution is a priority: the user wants the executable and application data to be usable from a folder together, subject to filesystem permissions.
- Online time synchronization is preferred when available; use the computer's clock when offline or synchronization fails.
- GitHub-hosted releases are the intended source for application updates.
- Modern, clean, rounded interface; light and dark themes; customizable accent color; responsive layouts for small laptops/netbooks through desktop monitors.
- The frontend must not access SQLite directly. All persistence and business logic go through Rust/Tauri commands.

### 2.2 Explicitly configurable defaults for decisions not yet confirmed

The user has not yet specified these details. Use the following defaults for the first implementation, and keep the design adaptable:

- **Session plans:** optional. An appointment may be independent or belong to a multi-session plan. A plan groups appointments; it is not itself an appointment.
- **Recurring appointments:** do not implement recurrence in the initial release. Structure the model so recurrence can be added later without corrupting existing appointments.
- **System tray:** do not require a tray icon for the first usable release. The app should support normal close/exit and catch up overdue reminders on next launch. A tray mode can be added later.
- **Backup retention:** keep the latest 30 automatic backups by default; make the retention setting configurable.
- **Theme behavior:** support Light, Dark, and System modes; System is the default.
- **Business specialization:** generic. Do not assume medical, beauty, legal, or other regulated workflows.
- **Appointment duration:** store explicit start and end instants; provide a sensible configurable default duration for new appointments.
- **Client data:** collect only practical business contact/profile information. Do not add sensitive medical, financial, identity-document, or other high-risk fields unless explicitly requested.

Do not interrupt implementation to ask about these defaults. Record them in documentation and expose appropriate preferences where practical.

---

## 3. Core functional scope

The finished application must support:

### 3.1 First-run setup

- Detect whether business setup has been completed.
- If not, show a guided first-run wizard.
- Capture business name (required), optional logo, offered services, and business location/contact details where appropriate.
- Allow the operator to skip optional fields and finish setup.
- Persist setup data locally.
- After completion, open the main application.
- Allow business details and logo to be edited later in Settings.

### 3.2 Dashboard

Show useful, accurately calculated summaries, including:

- today's appointments/sessions;
- upcoming appointments;
- completed appointments in the selected reporting period;
- pending or otherwise outstanding appointments;
- relevant client and session-plan summaries.

Avoid ambiguous metrics. Define every status and date boundary. Dashboard values must be calculated from persisted records, not hardcoded sample data.

### 3.3 Clients

- Create, view, edit, search, archive, and restore clients.
- Store name and practical contact details, optional notes, and timestamps.
- Prevent accidental duplicate creation where practical, but do not assume names are unique.
- Preserve appointment history when a client is archived.
- Do not hard-delete clients with dependent historical records through ordinary UI actions.

### 3.4 Services

- Create, view, edit, activate/deactivate, and archive services.
- Store service name, optional description, default duration, and optional non-financial metadata.
- No pricing or payment fields.
- Existing appointments must retain the service name/details that applied when the appointment was created, even if the service is later renamed or deactivated.
- Deactivated services should not be offered for new appointments but remain visible in historical records.

### 3.5 Appointments and sessions

- Create, view, edit, reschedule, cancel, and mark appointments as completed.
- Select an existing client or create a client inline from the appointment workflow.
- Select one or more services for an appointment.
- Store start and end times, status, optional notes, and timestamps.
- Support independent appointments and optional session plans.
- Session plans group a sequence/count of sessions for a client. The app should show the planned total and linked appointment count/status where appropriate.
- Validate required fields, time ranges, and references.
- Rescheduling/cancellation must update or invalidate pending reminders transactionally.
- Do not treat a session plan as a substitute for individual appointment records.

### 3.6 Calendar

Provide day, week, month, and agenda/list views. Users must be able to navigate dates and open appointment details. Use compact agenda/list behavior on narrow windows. Calendar data must come from the Rust backend and reflect persisted appointments.

### 3.7 Reminders and notifications

Initial reminder rules:

- one day before the appointment;
- one hour before the appointment;
- on the appointment day, once when the application is first launched that local day.

Provide in-app notifications and Windows system notifications where supported and permitted.

Important constraints:

- The app cannot deliver reminders while its process is completely terminated unless a separate operating-system scheduled task/service is installed. Do not claim otherwise.
- On startup, detect and process reminders that became due while the app was closed, applying sensible expiry and duplicate-prevention rules.
- Do not send stale reminders for appointments already cancelled or completed.
- Rescheduling must recalculate reminders.
- Persist notification delivery state to prevent duplicate notifications.
- If OS notification delivery fails or permission is unavailable, retain an in-app notification/error state and log the failure.
- Use the business timezone for “appointment day” semantics.

### 3.8 Reports and exports

- Weekly and monthly summaries.
- Clearly define the business-local reporting period.
- Include appointment counts by status and other useful non-financial operational summaries.
- Export data in practical formats such as CSV and JSON. PDF can be added if it fits the implementation plan.
- Exports must not leak internal secrets or machine-specific paths.
- Provide clear success/failure feedback and choose a user-visible destination.

### 3.9 Backups and restore

- Both automatic and manual backups.
- Use SQLite's supported online backup mechanism or another SQLite-safe method. Do not naively copy an active database file while WAL mode is in use.
- Include a backup timestamp and schema/app compatibility metadata.
- Validate backup integrity before reporting success.
- Offer restore with explicit confirmation and a safety backup of the current database.
- Use a configurable retention policy, default 30 automatic backups.
- A backup must be recoverable after an interrupted or failed operation.

### 3.10 Settings, portability, and updates

- Business settings, appearance settings, timezone-related settings, reminder preferences, backup preferences, and About/version information.
- Portable data directory next to the executable when writable.
- If the executable directory is not writable (for example, under a protected installation path), clearly detect this and offer/use a safe per-user data directory. Never silently fail.
- Keep database, backups, exports, logos, logs, and temporary update files organized.
- GitHub release-based updates with signature/integrity verification and safe rollback strategy.
- Do not assume the standard Tauri updater automatically solves replacement of a portable standalone executable. Implement and test a safe platform-appropriate strategy in the dedicated update step.
- Database migrations must be backward-conscious and never destroy user data silently.

---

## 4. Technical architecture

### 4.1 Stack

Use the existing project scaffold as the starting point. Expected architecture:

**Desktop/backend**
- Tauri 2
- Rust stable
- Tokio where asynchronous work is appropriate
- SQLite
- SQLx or another well-justified Rust SQLite access layer
- Serde
- Chrono and IANA timezone support
- tracing for structured logs
- thiserror and/or anyhow for error handling

**Frontend**
- React
- TypeScript
- Vite
- Tailwind CSS using conventions compatible with the installed version
- shadcn/ui components where useful
- Lucide icons
- React Router
- TanStack Query for server-state/IPC data
- Zustand only for appropriate client/UI state
- React Hook Form and Zod for forms and validation
- FullCalendar or a well-justified equivalent
- Recharts or a well-justified equivalent for reports

Do not blindly upgrade dependencies. Respect the scaffold's existing compatible versions and verify current Tauri 2 conventions.

### 4.2 Architectural boundaries

Use a modular monolith, not microservices.

Frontend:
- presentation, navigation, local form state, client-side validation, and rendering;
- invokes typed Tauri commands;
- does not access the SQLite database, filesystem, OS notifications, or updater directly.

Rust backend:
- owns persistence, validation, business rules, scheduling, time calculations, backups, exports, notifications, and platform integration;
- exposes narrow, typed Tauri commands;
- uses repository/service boundaries to keep command handlers thin.

Suggested backend modules:
- `commands/`
- `database/`
- `models/`
- `repositories/`
- `services/`
- `scheduler/`
- `notifications/`
- `backup/`
- `export/`
- `updater/`
- `time/`
- `platform/`
- `errors/`

Suggested frontend feature modules:
- `app/`
- `components/`
- `features/business/`
- `features/clients/`
- `features/services/`
- `features/appointments/`
- `features/session-plans/`
- `features/calendar/`
- `features/dashboard/`
- `features/reminders/`
- `features/reports/`
- `features/settings/`
- `lib/`
- `styles/`
- `types/`

Adapt the existing project rather than creating duplicate parallel structures.

### 4.3 IPC contracts

- Use typed command inputs and outputs.
- Use consistent snake_case Rust command names and documented frontend wrappers.
- Return structured, serializable errors; do not expose raw internal stack traces to the UI.
- Validate all inputs in Rust even when frontend validation exists.
- Avoid passing arbitrary SQL or filesystem paths from the frontend.
- Keep Tauri capabilities minimal and least-privilege.
- Do not enable broad filesystem/shell permissions without a specific documented need.

### 4.4 Database principles

- SQLite with foreign keys enabled.
- Use transactions for multi-table writes.
- Use migrations tracked in source control.
- Add indexes for common filters and joins.
- Use stable primary keys (UUID or another consistently applied strategy).
- Store instants in UTC; retain the relevant IANA timezone identifier separately.
- Convert to business-local time for calendar display, day-based reminders, and reporting.
- Use explicit status enums/check constraints where practical.
- Use archive/soft-delete semantics for business records with history.
- Preserve historical service labels via appointment-service snapshots.
- Use parameterized queries only.
- Never use destructive migrations without a documented data-preserving plan.
- Include schema migration and backup/restore tests.

Suggested core entities (refine before migration implementation):
- business
- clients
- services
- session_plans
- appointments
- appointment_services
- reminders
- notification_queue
- app_settings
- activity_log (optional, useful for critical changes)
- SQLx migration metadata

Possible appointment statuses: `scheduled`, `completed`, `cancelled`, `no_show` (only include `no_show` if the UI and reporting define it clearly). Avoid multiple overlapping meanings such as “pending” without a precise definition.

### 4.5 Time correctness

- Persist timestamps as UTC instants.
- Store business timezone as an IANA timezone identifier, e.g. `Europe/Madrid`, not only a fixed UTC offset.
- Use timezone-aware conversion for local calendar boundaries and daylight-saving transitions.
- Define “one day before” as the corresponding local calendar date/time policy and document the exact rule.
- Appointment-day reminder means once per business-local calendar date on first app launch that day.
- Do not use the PC's current local offset as a permanent timezone identity.
- Online time synchronization is best-effort. Prefer a reputable NTP implementation/library or a carefully justified time service. Do not trust arbitrary HTTP `Date` headers as authoritative time.
- If synchronization is unavailable, use the system clock and clearly log the fallback.
- Never make the app unusable because it is offline.

### 4.6 Data directory and portability

Design a single storage-path resolver in Rust. It should:

1. Determine the executable location.
2. Check whether the adjacent data directory can be created and written.
3. If yes, use a predictable adjacent structure.
4. If no, use a documented per-user application-data directory.
5. Persist/report the selected mode so the UI can explain where data lives.
6. Avoid path traversal and unsafe user-controlled path composition.

Suggested layout:

```text
BusinessManager/
  BusinessManager.exe
  data/
    database.sqlite
    backups/
      automatic/
      manual/
    exports/
    logos/
    logs/
    updates/
```

Runtime SQLite `-wal` and `-shm` files may appear beside the database. Do not manually delete them while the database is active.

### 4.7 Quality and security

- No authentication is required by the product decision, but still apply secure coding practices.
- No secrets in source control, logs, exports, or error messages.
- Do not log client notes or unnecessary personal data.
- Validate and normalize user input.
- Use safe file operations and atomic writes where possible.
- Keep dependencies minimal and maintained.
- Handle failures visibly and recoverably.
- Use accessibility-friendly labels, keyboard navigation, and focus states.
- Support light/dark/system themes and accent customization.
- Keep UI responsive and avoid oversized fixed-width layouts.

---

## 5. UI/UX direction

Create a professional, calm, modern desktop interface:

- rounded but restrained surfaces;
- clear typographic hierarchy;
- consistent spacing and component sizing;
- sidebar or compact navigation suited to desktop;
- clear active navigation state;
- light and dark themes;
- customizable accent color;
- System theme default;
- responsive behavior for small laptop windows;
- useful empty states and loading states;
- confirmation for destructive/irreversible actions;
- no decorative metrics or fake demo records in production;
- avoid excessive gradients, animations, or visual clutter.

The interface should feel like a serious desktop productivity application, not a generic admin template. Maintain visual consistency across all steps.

---

## 6. Testing and acceptance

Each step file defines step-specific acceptance criteria. Across the whole project, include:

- Rust unit tests for business rules and time calculations;
- database repository and migration tests;
- command/service integration tests;
- frontend type checking and linting;
- frontend component/logic tests where useful;
- scheduler and duplicate-reminder tests;
- backup integrity and restore tests;
- export tests;
- update verification/rollback tests;
- Windows packaging and clean-machine smoke tests;
- offline-mode tests;
- timezone and daylight-saving tests.

At minimum, run the checks that exist in the repository. Common commands may include:

```bash
npm run typecheck
npm run lint
npm run test
npm run build
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
cargo check
```

Use actual package scripts. If a command does not exist, do not pretend it ran; either add the script when appropriate or record it as unavailable.

---

## 7. Implementation sequence

Complete the numbered steps in order:

| Step | Scope |
|---|---|
| 00 | Repository audit, scaffold verification, engineering baseline |
| 01 | Database foundation, migrations, storage-path resolution |
| 02 | Business profile and first-run onboarding |
| 03 | Client management |
| 04 | Service catalog |
| 05 | Appointments and optional session plans |
| 06 | Calendar views and navigation |
| 07 | Reminder engine and notifications |
| 08 | Dashboard and operational reports |
| 09 | Exports and backup/restore |
| 10 | Settings, themes, and application preferences |
| 11 | Portable Windows packaging and release readiness |
| 12 | GitHub update mechanism and safe migration/update flow |
| 13 | End-to-end hardening, QA, documentation, and release candidate |

A step may contain sub-tasks. Do not mark a step complete until its acceptance criteria are met and its checks have been run.

---

## 8. Definition of done

The application is complete only when:

- all required workflows are implemented and persist correctly;
- no core feature depends on internet connectivity;
- all date/time behavior is timezone-correct and tested;
- reminders avoid duplicate/stale delivery and clearly document closed-app limitations;
- backups can be restored and are validated;
- exports are correct and user-accessible;
- the Windows portable/fallback storage behavior is verified;
- updates verify authenticity/integrity and protect user data;
- UI is responsive, accessible, and consistent;
- tests, formatting, linting, and builds pass, or any remaining exceptions are explicitly documented;
- README and user-facing operational documentation are complete.

**Begin with Step 00 only. Do not implement the full application in one pass.**
