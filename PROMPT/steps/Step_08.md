# Step 08 — Dashboard and operational reports

## Objective

Implement a useful dashboard and weekly/monthly operational summaries based on real persisted data.

## Tasks

1. Define appointment status semantics in code and documentation.
2. Implement Rust query/service layer for dashboard statistics and date-range summaries.
3. Use business-local day/week/month boundaries converted safely to UTC.
4. Dashboard should include:
   - today's appointments;
   - upcoming appointments;
   - completed count for the selected period;
   - scheduled/outstanding counts;
   - concise session-plan/client context where useful.
5. Implement weekly and monthly report views.
6. Include counts by appointment status and other useful non-financial operational indicators.
7. Clearly label the reporting period and timezone.
8. Ensure cancelled appointments are not accidentally counted as completed or upcoming.
9. Avoid misleading labels such as “pending” unless precisely defined.
10. Charts must have accessible labels and a useful tabular/text alternative.
11. Do not include pricing, revenue, payment, or accounting metrics.
12. Add tests for report boundaries, status aggregation, empty periods, and timezone transitions.
13. Update `Progress.md`.

## Acceptance criteria

- Dashboard values reflect the database.
- Weekly/monthly reports use correct business-local periods.
- Metrics are consistently defined.
- No financial features are introduced.
- Tests/build pass.

## Stop condition

Stop after Step 08.
