# Step 10 — Settings, themes, and application preferences

## Objective

Complete the settings experience and make configurable behavior understandable to the operator.

## Tasks

1. Implement business profile editing using Step 02 services.
2. Implement appearance settings:
   - Light;
   - Dark;
   - System (default);
   - customizable accent color with accessible contrast.
3. Persist preferences locally and apply them consistently at startup.
4. Implement timezone display/configuration. Changing the business timezone must not rewrite stored UTC instants; explain how future calendar/report/reminder interpretation changes.
5. Implement reminder preferences and explain notification permission/closed-app limitations.
6. Implement backup preferences including automatic backup enablement, retention (default 30), and location information.
7. Implement storage diagnostics showing whether data is stored adjacent to the executable or in the per-user fallback. Avoid exposing sensitive full paths unnecessarily; provide copy/reveal only when appropriate.
8. Implement About/version information and diagnostics useful for support.
9. Add confirmation for operations that affect data interpretation or restore behavior.
10. Test preference persistence, theme behavior, timezone changes, and invalid settings.
11. Update `Progress.md`.

## Acceptance criteria

- Settings are persisted and applied after restart.
- Light/dark/system themes work.
- Accent color remains readable.
- Business/timezone/reminder/backup preferences are clear.
- Tests/build pass.

## Stop condition

Stop after Step 10.
