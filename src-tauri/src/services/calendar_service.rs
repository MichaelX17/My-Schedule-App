use chrono::{Datelike, Duration, NaiveDate};
use sqlx::SqlitePool;

use crate::database::repositories::appointment_repo;
use crate::errors::{AppError, AppResult};
use crate::models::calendar::{CalendarRangeInput, CalendarRangeResponse, CalendarView};
use crate::services::appointment_service;
use crate::time;

/// Number of business-local calendar days included in the Agenda view window.
pub const AGENDA_WINDOW_DAYS: i64 = 14;

/// Computes the inclusive `(range_start, range_end)` business-local dates for a given
/// calendar view and anchor date.
///
/// - `Day`: `(anchor, anchor)`
/// - `Week`: Monday through Sunday containing `anchor`
/// - `Month`: Monday of the week containing the 1st of `anchor`'s month through Sunday
///   of the week containing the last day of `anchor`'s month
/// - `Agenda`: `anchor` through `anchor + 13 days` (14-day window)
pub fn compute_calendar_range(
    view: CalendarView,
    anchor: NaiveDate,
) -> AppResult<(NaiveDate, NaiveDate)> {
    match view {
        CalendarView::Day => Ok((anchor, anchor)),
        CalendarView::Week => {
            let days_from_mon = i64::from(anchor.weekday().num_days_from_monday());
            let start = anchor - Duration::days(days_from_mon);
            let end = start + Duration::days(6);
            Ok((start, end))
        }
        CalendarView::Month => {
            let first_of_month = NaiveDate::from_ymd_opt(anchor.year(), anchor.month(), 1)
                .ok_or_else(|| AppError::Validation("Invalid calendar anchor date.".into()))?;
            let first_of_next_month = if anchor.month() == 12 {
                NaiveDate::from_ymd_opt(anchor.year() + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(anchor.year(), anchor.month() + 1, 1)
            }
            .ok_or_else(|| AppError::Validation("Calendar date range is out of bounds.".into()))?;
            let last_of_month = first_of_next_month.pred_opt().ok_or_else(|| {
                AppError::Validation("Calendar date range is out of bounds.".into())
            })?;

            let start_offset = i64::from(first_of_month.weekday().num_days_from_monday());
            let end_offset = 6 - i64::from(last_of_month.weekday().num_days_from_monday());

            let grid_start = first_of_month - Duration::days(start_offset);
            let grid_end = last_of_month + Duration::days(end_offset);
            Ok((grid_start, grid_end))
        }
        CalendarView::Agenda => {
            let end = anchor + Duration::days(AGENDA_WINDOW_DAYS - 1);
            Ok((anchor, end))
        }
    }
}

/// Queries appointments overlapping the requested business-timezone calendar view range.
pub async fn get_calendar_range(
    pool: &SqlitePool,
    input: CalendarRangeInput,
) -> AppResult<CalendarRangeResponse> {
    let business_tz = appointment_service::resolve_business_timezone(pool).await?;
    let today = time::today_in_timezone(&business_tz)?;

    let anchor = match input.anchor_date.as_deref().map(str::trim) {
        Some(raw) if !raw.is_empty() => time::parse_local_date(raw)?,
        _ => today,
    };

    let (range_start, range_end) = compute_calendar_range(input.view, anchor)?;
    let (from_utc, to_utc_exclusive) =
        time::local_date_range_to_utc(range_start, range_end, &business_tz)?;

    let from_utc_iso = time::format_utc_iso(from_utc);
    let to_utc_iso = time::format_utc_iso(to_utc_exclusive);

    let rows =
        appointment_repo::list_rows_in_utc_range(pool, &from_utc_iso, &to_utc_iso, input.status)
            .await?;

    let mut appointments = Vec::with_capacity(rows.len());
    for row in rows {
        appointments.push(appointment_service::hydrate_detail(pool, row, &business_tz).await?);
    }

    Ok(CalendarRangeResponse {
        view: input.view,
        timezone: business_tz,
        today_local: time::format_local_date(today),
        anchor_date: time::format_local_date(anchor),
        range_start_local: time::format_local_date(range_start),
        range_end_local: time::format_local_date(range_end),
        range_start_utc: from_utc_iso,
        range_end_utc_exclusive: to_utc_iso,
        appointments,
    })
}
