use chrono::{
    DateTime, Duration, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, SecondsFormat, TimeZone,
    Utc,
};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};

/// Explicit disambiguation choice for ambiguous local times during DST fall-back transitions.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DstDisambiguation {
    Earlier,
    Later,
}

/// Returns the current UTC timestamp formatted as RFC 3339 with millisecond precision.
pub fn now_utc_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Formats a `DateTime<Utc>` as a sortable RFC 3339 UTC string with millisecond precision.
pub fn format_utc_iso(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Parses an RFC 3339 UTC timestamp string.
pub fn parse_utc_iso(raw: &str) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw.trim())
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| AppError::Validation("Invalid UTC timestamp format.".into()))
}

/// Detects the host OS IANA timezone identifier (e.g., `Europe/Madrid`),
/// falling back to `"UTC"` if detection fails or returns an unrecognized value.
pub fn detect_system_timezone() -> String {
    match iana_time_zone::get_timezone() {
        Ok(tz) if tz.parse::<Tz>().is_ok() => tz,
        Ok(tz) => {
            tracing::warn!(
                detected = %tz,
                "Detected OS timezone is not a recognized IANA identifier; falling back to UTC"
            );
            "UTC".to_string()
        }
        Err(err) => {
            tracing::warn!(
                error = %err,
                "Unable to detect OS timezone; falling back to UTC"
            );
            "UTC".to_string()
        }
    }
}

/// Validates and normalizes an IANA timezone identifier using `chrono-tz`.
pub fn validate_iana_timezone(input: &str) -> AppResult<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Business timezone is required.".into(),
        ));
    }

    let parsed: Tz = trimmed.parse().map_err(|_| {
        AppError::Validation(
            "Please select a valid IANA timezone identifier (for example, Europe/Madrid or America/New_York)."
                .into(),
        )
    })?;

    Ok(parsed.name().to_string())
}

/// Parses a local date-time string (`YYYY-MM-DDTHH:MM` or `YYYY-MM-DDTHH:MM:SS`)
/// in the specified IANA timezone and converts it to a UTC instant.
///
/// Handles daylight-saving transitions deliberately:
/// - Spring-forward gap (`LocalResult::None`): returns `AppError::Validation` instead of silently shifting.
/// - Fall-back overlap (`LocalResult::Ambiguous`): uses `disambiguation` if provided, otherwise returns `AppError::Validation`.
pub fn resolve_local_datetime_to_utc(
    local_dt_str: &str,
    iana_tz: &str,
    disambiguation: Option<DstDisambiguation>,
) -> AppResult<DateTime<Utc>> {
    let tz: Tz = iana_tz
        .trim()
        .parse()
        .map_err(|_| AppError::Validation("Invalid business IANA timezone identifier.".into()))?;

    let trimmed = local_dt_str.trim();
    let naive = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S"))
        .map_err(|_| {
            AppError::Validation(
                "Please enter a valid date and time in YYYY-MM-DDTHH:MM format.".into(),
            )
        })?;

    match tz.from_local_datetime(&naive) {
        LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
        LocalResult::None => Err(AppError::Validation(format!(
            "The local time {trimmed} does not exist in {tz} due to a daylight-saving time transition. Please choose a valid time."
        ))),
        LocalResult::Ambiguous(earliest, latest) => match disambiguation {
            Some(DstDisambiguation::Earlier) => Ok(earliest.with_timezone(&Utc)),
            Some(DstDisambiguation::Later) => Ok(latest.with_timezone(&Utc)),
            None => Err(AppError::Validation(format!(
                "The local time {trimmed} occurs twice in {tz} due to a daylight-saving time transition. Please choose the earlier or later occurrence."
            ))),
        },
    }
}

/// Formats a UTC RFC 3339 instant as `YYYY-MM-DDTHH:MM` in the given IANA timezone.
pub fn format_utc_in_timezone(utc_iso: &str, iana_tz: &str) -> AppResult<String> {
    let dt = parse_utc_iso(utc_iso)?;
    let tz: Tz = iana_tz.trim().parse().unwrap_or(chrono_tz::UTC);
    Ok(dt.with_timezone(&tz).format("%Y-%m-%dT%H:%M").to_string())
}

/// Parses a `YYYY-MM-DD` business-local calendar date.
pub fn parse_local_date(raw: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").map_err(|_| {
        AppError::Validation("Please provide a valid date in YYYY-MM-DD format.".into())
    })
}

/// Formats a `NaiveDate` as `YYYY-MM-DD`.
pub fn format_local_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Returns today's calendar date in the specified IANA timezone.
pub fn today_in_timezone(iana_tz: &str) -> AppResult<NaiveDate> {
    let tz: Tz = iana_tz
        .trim()
        .parse()
        .map_err(|_| AppError::Validation("Invalid business IANA timezone identifier.".into()))?;
    Ok(Utc::now().with_timezone(&tz).date_naive())
}

/// Resolves the exact start-of-day UTC instant for `date` in `tz`.
///
/// Handles DST transitions that occur at local midnight:
/// - `Single(dt)`: standard midnight instant.
/// - `Ambiguous(earliest, _)`: fall-back at midnight — the day starts at the earliest occurrence.
/// - `None`: spring-forward across midnight (e.g. `America/Havana`, `America/Sao_Paulo`) —
///   advances to the first valid local instant on `date`.
pub fn local_day_start_utc(date: NaiveDate, tz: Tz) -> AppResult<DateTime<Utc>> {
    let midnight = NaiveDateTime::new(date, NaiveTime::MIN);
    match tz.from_local_datetime(&midnight) {
        LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
        LocalResult::Ambiguous(earliest, _) => Ok(earliest.with_timezone(&Utc)),
        LocalResult::None => {
            // Find the first valid minute on `date` after the midnight spring-forward gap.
            for minutes in 1..=240 {
                let candidate = midnight + Duration::minutes(minutes);
                match tz.from_local_datetime(&candidate) {
                    LocalResult::Single(dt) => return Ok(dt.with_timezone(&Utc)),
                    LocalResult::Ambiguous(earliest, _) => {
                        return Ok(earliest.with_timezone(&Utc));
                    }
                    LocalResult::None => continue,
                }
            }
            Err(AppError::Internal(
                "Unable to resolve local start of day in business timezone.".into(),
            ))
        }
    }
}

/// Converts an inclusive business-local date range `[start_date, end_date_inclusive]`
/// into a half-open UTC query interval `[from_utc_inclusive, to_utc_exclusive)`.
///
/// An appointment overlaps the business-local date range if and only if:
/// `appointment.start_utc < to_utc_exclusive && appointment.end_utc > from_utc_inclusive`.
pub fn local_date_range_to_utc(
    start_date: NaiveDate,
    end_date_inclusive: NaiveDate,
    iana_tz: &str,
) -> AppResult<(DateTime<Utc>, DateTime<Utc>)> {
    if end_date_inclusive < start_date {
        return Err(AppError::Validation(
            "Calendar end date cannot be earlier than start date.".into(),
        ));
    }

    let tz: Tz = iana_tz
        .trim()
        .parse()
        .map_err(|_| AppError::Validation("Invalid business IANA timezone identifier.".into()))?;

    let next_day = end_date_inclusive
        .succ_opt()
        .ok_or_else(|| AppError::Validation("Calendar date range is out of bounds.".into()))?;

    let from_utc = local_day_start_utc(start_date, tz)?;
    let to_utc_exclusive = local_day_start_utc(next_day, tz)?;
    Ok((from_utc, to_utc_exclusive))
}

/// Returns the business-local calendar date for a given UTC instant.
pub fn local_date_for_utc(dt: DateTime<Utc>, iana_tz: &str) -> AppResult<NaiveDate> {
    let tz: Tz = iana_tz
        .trim()
        .parse()
        .map_err(|_| AppError::Validation("Invalid business IANA timezone identifier.".into()))?;
    Ok(dt.with_timezone(&tz).date_naive())
}

/// Computes the exact UTC due instant for the **one-day-before** reminder rule.
///
/// Documented policy:
/// - Same business-local wall-clock time on the preceding calendar date
///   (`appointment_local_date - 1 day`, same `HH:MM:SS`) in `iana_tz`.
/// - Fall-back ambiguity (`LocalResult::Ambiguous`): uses the earliest occurrence
///   so the operator is reminded early rather than late.
/// - Spring-forward gap (`LocalResult::None`): advances minute-by-minute to the
///   first valid local instant after the gap so the reminder is never dropped.
pub fn compute_one_day_before_utc(
    appointment_start_utc: DateTime<Utc>,
    iana_tz: &str,
) -> AppResult<DateTime<Utc>> {
    let tz: Tz = iana_tz
        .trim()
        .parse()
        .map_err(|_| AppError::Validation("Invalid business IANA timezone identifier.".into()))?;

    let local_start = appointment_start_utc.with_timezone(&tz);
    let prev_date = local_start
        .date_naive()
        .pred_opt()
        .ok_or_else(|| AppError::Validation("Appointment date is out of bounds.".into()))?;
    let target_naive = NaiveDateTime::new(prev_date, local_start.time());

    match tz.from_local_datetime(&target_naive) {
        LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
        LocalResult::Ambiguous(earliest, _) => Ok(earliest.with_timezone(&Utc)),
        LocalResult::None => {
            for minutes in 1..=240 {
                let candidate = target_naive + Duration::minutes(minutes);
                match tz.from_local_datetime(&candidate) {
                    LocalResult::Single(dt) => return Ok(dt.with_timezone(&Utc)),
                    LocalResult::Ambiguous(earliest, _) => {
                        return Ok(earliest.with_timezone(&Utc));
                    }
                    LocalResult::None => continue,
                }
            }
            Err(AppError::Internal(
                "Unable to resolve one-day-before reminder time in business timezone.".into(),
            ))
        }
    }
}

/// Computes the exact UTC due instant for the **one-hour-before** reminder rule
/// (`appointment_start_utc - 60 minutes` in real elapsed time, even across DST transitions).
pub fn compute_one_hour_before_utc(appointment_start_utc: DateTime<Utc>) -> DateTime<Utc> {
    appointment_start_utc - Duration::hours(1)
}
