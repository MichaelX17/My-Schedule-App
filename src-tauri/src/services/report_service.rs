use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use sqlx::{FromRow, SqlitePool};

use crate::database::repositories::{business_repo, reminder_repo};
use crate::errors::{AppError, AppResult};
use crate::models::appointment::{AppointmentRow, AppointmentStatus};
use crate::models::reports::{
    DailyBreakdownPoint, DashboardPeriod, DashboardSummary, OperationalReport,
    OperationalReportInput, ReportPeriodType, ServiceUsageItem, StatusCounts,
};
use crate::services::appointment_service;
use crate::time;

const MAX_UPCOMING_DASHBOARD_ITEMS: usize = 8;

#[derive(FromRow)]
struct PeriodAppointmentRow {
    client_id: String,
    start_utc: String,
    end_utc: String,
    status: String,
}

#[derive(FromRow)]
struct ServiceUsageRow {
    service_name_snapshot: String,
    appointment_count: i64,
    total_minutes: i64,
}

/// Computes the inclusive business-local `(start_date, end_date)` for a weekly (Mon–Sun)
/// or monthly (1st–last day of month) reporting period.
pub fn compute_report_period(
    period_type: ReportPeriodType,
    anchor: NaiveDate,
) -> AppResult<(NaiveDate, NaiveDate)> {
    match period_type {
        ReportPeriodType::Weekly => {
            let offset = i64::from(anchor.weekday().num_days_from_monday());
            let start = anchor - Duration::days(offset);
            let end = start + Duration::days(6);
            Ok((start, end))
        }
        ReportPeriodType::Monthly => {
            let first = NaiveDate::from_ymd_opt(anchor.year(), anchor.month(), 1)
                .ok_or_else(|| AppError::Validation("Invalid report anchor date.".into()))?;
            let next_month_first = if anchor.month() == 12 {
                NaiveDate::from_ymd_opt(anchor.year() + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(anchor.year(), anchor.month() + 1, 1)
            }
            .ok_or_else(|| AppError::Validation("Report date range is out of bounds.".into()))?;
            let last = next_month_first.pred_opt().ok_or_else(|| {
                AppError::Validation("Report date range is out of bounds.".into())
            })?;
            Ok((first, last))
        }
    }
}

fn tally_status(counts: &mut StatusCounts, status: &str) {
    counts.total += 1;
    match status {
        "scheduled" => counts.scheduled += 1,
        "completed" => counts.completed += 1,
        "cancelled" => counts.cancelled += 1,
        _ => {}
    }
}

async fn fetch_appointments_starting_in_utc_range(
    pool: &SqlitePool,
    from_utc_inclusive: &str,
    to_utc_exclusive: &str,
) -> AppResult<Vec<PeriodAppointmentRow>> {
    let rows = sqlx::query_as::<_, PeriodAppointmentRow>(
        r#"
        SELECT client_id, start_utc, end_utc, status
        FROM appointments
        WHERE start_utc >= ?1 AND start_utc < ?2
        ORDER BY start_utc ASC, id ASC
        "#,
    )
    .bind(from_utc_inclusive)
    .bind(to_utc_exclusive)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn fetch_hydrated_appointments_starting_in_range(
    pool: &SqlitePool,
    from_utc_inclusive: &str,
    to_utc_exclusive: &str,
    business_tz: &str,
) -> AppResult<Vec<crate::models::appointment::AppointmentDetail>> {
    let rows = sqlx::query_as::<_, AppointmentRow>(
        r#"
        SELECT
            a.id,
            a.client_id,
            c.full_name AS client_name,
            c.is_archived AS client_is_archived,
            a.session_plan_id,
            sp.title AS session_plan_title,
            a.start_utc,
            a.end_utc,
            a.timezone,
            a.status,
            a.notes,
            a.created_at,
            a.updated_at
        FROM appointments a
        INNER JOIN clients c ON c.id = a.client_id
        LEFT JOIN session_plans sp ON sp.id = a.session_plan_id
        WHERE a.start_utc >= ?1 AND a.start_utc < ?2
        ORDER BY a.start_utc ASC, a.id ASC
        "#,
    )
    .bind(from_utc_inclusive)
    .bind(to_utc_exclusive)
    .fetch_all(pool)
    .await?;

    let mut details = Vec::with_capacity(rows.len());
    for row in rows {
        details.push(appointment_service::hydrate_detail(pool, row, business_tz).await?);
    }
    Ok(details)
}

async fn fetch_upcoming_scheduled_rows(
    pool: &SqlitePool,
    now_utc_iso: &str,
    limit: i64,
) -> AppResult<Vec<AppointmentRow>> {
    let rows = sqlx::query_as::<_, AppointmentRow>(
        r#"
        SELECT
            a.id,
            a.client_id,
            c.full_name AS client_name,
            c.is_archived AS client_is_archived,
            a.session_plan_id,
            sp.title AS session_plan_title,
            a.start_utc,
            a.end_utc,
            a.timezone,
            a.status,
            a.notes,
            a.created_at,
            a.updated_at
        FROM appointments a
        INNER JOIN clients c ON c.id = a.client_id
        LEFT JOIN session_plans sp ON sp.id = a.session_plan_id
        WHERE a.status = 'scheduled' AND a.end_utc > ?1
        ORDER BY a.start_utc ASC, a.id ASC
        LIMIT ?2
        "#,
    )
    .bind(now_utc_iso)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Computes the live operational dashboard summary at the current UTC instant.
pub async fn get_dashboard_summary(
    pool: &SqlitePool,
    period: DashboardPeriod,
) -> AppResult<DashboardSummary> {
    get_dashboard_summary_at(pool, period, Utc::now()).await
}

/// Deterministic dashboard computation with injectable `now_utc` for testing.
pub async fn get_dashboard_summary_at(
    pool: &SqlitePool,
    period: DashboardPeriod,
    now_utc: DateTime<Utc>,
) -> AppResult<DashboardSummary> {
    let profile = business_repo::get(pool).await?;
    let business_name = profile
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "My Schedule App".to_string());
    let business_tz = profile
        .as_ref()
        .map(|p| p.timezone.clone())
        .unwrap_or_else(|| "UTC".to_string());

    let today_local = time::local_date_for_utc(now_utc, &business_tz)?;
    let now_iso = time::format_utc_iso(now_utc);

    // 1. Today's appointments (starting on today_local in business timezone)
    let (today_from_utc, today_to_utc) =
        time::local_date_range_to_utc(today_local, today_local, &business_tz)?;
    let today_from_iso = time::format_utc_iso(today_from_utc);
    let today_to_iso = time::format_utc_iso(today_to_utc);

    let today_appointments = fetch_hydrated_appointments_starting_in_range(
        pool,
        &today_from_iso,
        &today_to_iso,
        &business_tz,
    )
    .await?;

    let mut today_counts = StatusCounts::default();
    for appt in &today_appointments {
        tally_status(&mut today_counts, &appt.status);
    }

    // 2. Selected period status counts
    let (period_start, period_end) = match period {
        DashboardPeriod::Today => (today_local, today_local),
        DashboardPeriod::ThisWeek => compute_report_period(ReportPeriodType::Weekly, today_local)?,
        DashboardPeriod::ThisMonth => {
            compute_report_period(ReportPeriodType::Monthly, today_local)?
        }
    };
    let (period_from_utc, period_to_utc) =
        time::local_date_range_to_utc(period_start, period_end, &business_tz)?;
    let period_rows = fetch_appointments_starting_in_utc_range(
        pool,
        &time::format_utc_iso(period_from_utc),
        &time::format_utc_iso(period_to_utc),
    )
    .await?;

    let mut period_counts = StatusCounts::default();
    for row in &period_rows {
        tally_status(&mut period_counts, &row.status);
    }

    // 3. Upcoming scheduled appointments (strictly excludes cancelled & completed)
    let upcoming_rows =
        fetch_upcoming_scheduled_rows(pool, &now_iso, MAX_UPCOMING_DASHBOARD_ITEMS as i64).await?;
    let mut upcoming_appointments = Vec::with_capacity(upcoming_rows.len());
    for row in upcoming_rows {
        upcoming_appointments
            .push(appointment_service::hydrate_detail(pool, row, &business_tz).await?);
    }

    let upcoming_scheduled_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM appointments WHERE status = 'scheduled' AND end_utc > ?1",
    )
    .bind(&now_iso)
    .fetch_one(pool)
    .await?;

    let needs_outcome_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM appointments WHERE status = 'scheduled' AND end_utc <= ?1",
    )
    .bind(&now_iso)
    .fetch_one(pool)
    .await?;

    let active_clients_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE is_archived = 0")
            .fetch_one(pool)
            .await?;

    let active_session_plans_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM session_plans WHERE status = 'active'")
            .fetch_one(pool)
            .await?;

    let unread_notifications_count = reminder_repo::count_unread_notifications(pool).await?;

    Ok(DashboardSummary {
        business_name,
        timezone: business_tz,
        today_local: time::format_local_date(today_local),
        period,
        period_start_local: time::format_local_date(period_start),
        period_end_local: time::format_local_date(period_end),
        today_appointments,
        upcoming_appointments,
        today_counts,
        period_counts,
        upcoming_scheduled_count,
        needs_outcome_count,
        active_clients_count,
        active_session_plans_count,
        unread_notifications_count,
    })
}

/// Computes a weekly or monthly operational report at the current UTC instant.
pub async fn get_operational_report(
    pool: &SqlitePool,
    input: OperationalReportInput,
) -> AppResult<OperationalReport> {
    get_operational_report_at(pool, input, Utc::now()).await
}

/// Deterministic weekly/monthly operational report computation with injectable `now_utc`.
pub async fn get_operational_report_at(
    pool: &SqlitePool,
    input: OperationalReportInput,
    now_utc: DateTime<Utc>,
) -> AppResult<OperationalReport> {
    let business_tz = appointment_service::resolve_business_timezone(pool).await?;
    let today_local = time::local_date_for_utc(now_utc, &business_tz)?;

    let anchor = match input.anchor_date.as_deref().map(str::trim) {
        Some(raw) if !raw.is_empty() => time::parse_local_date(raw)?,
        _ => today_local,
    };

    let (period_start, period_end) = compute_report_period(input.period_type, anchor)?;
    let (from_utc, to_utc_exclusive) =
        time::local_date_range_to_utc(period_start, period_end, &business_tz)?;
    let from_utc_iso = time::format_utc_iso(from_utc);
    let to_utc_iso = time::format_utc_iso(to_utc_exclusive);

    let rows = fetch_appointments_starting_in_utc_range(pool, &from_utc_iso, &to_utc_iso).await?;

    // Pre-seed every local day in the period so charts and tables include zero-activity days.
    let mut by_day: BTreeMap<String, StatusCounts> = BTreeMap::new();
    let mut cursor = period_start;
    while cursor <= period_end {
        by_day.insert(time::format_local_date(cursor), StatusCounts::default());
        cursor = cursor
            .succ_opt()
            .ok_or_else(|| AppError::Validation("Report date range is out of bounds.".into()))?;
    }

    let mut counts = StatusCounts::default();
    let mut total_active_minutes: i64 = 0;
    let mut active_clients: HashSet<String> = HashSet::new();

    for row in &rows {
        tally_status(&mut counts, &row.status);

        let start_dt = time::parse_utc_iso(&row.start_utc)?;
        let end_dt = time::parse_utc_iso(&row.end_utc)?;
        let local_day = time::format_local_date(time::local_date_for_utc(start_dt, &business_tz)?);

        if let Some(day_counts) = by_day.get_mut(&local_day) {
            tally_status(day_counts, &row.status);
        }

        // Cancelled appointments are excluded from active session minutes & clients served.
        if row.status != AppointmentStatus::Cancelled.as_str() {
            total_active_minutes += (end_dt - start_dt).num_minutes().max(0);
            active_clients.insert(row.client_id.clone());
        }
    }

    let daily_breakdown: Vec<DailyBreakdownPoint> = by_day
        .into_iter()
        .map(|(date_local, c)| DailyBreakdownPoint {
            date_local,
            scheduled: c.scheduled,
            completed: c.completed,
            cancelled: c.cancelled,
            total: c.total,
        })
        .collect();

    // Non-financial service usage from historical appointment-service snapshots (non-cancelled).
    let service_rows = sqlx::query_as::<_, ServiceUsageRow>(
        r#"
        SELECT
            s.service_name_snapshot,
            COUNT(DISTINCT s.appointment_id) AS appointment_count,
            COALESCE(SUM(s.duration_minutes_snapshot), 0) AS total_minutes
        FROM appointment_services s
        INNER JOIN appointments a ON a.id = s.appointment_id
        WHERE a.start_utc >= ?1
          AND a.start_utc < ?2
          AND a.status != 'cancelled'
        GROUP BY s.service_name_snapshot
        ORDER BY appointment_count DESC, s.service_name_snapshot ASC
        "#,
    )
    .bind(&from_utc_iso)
    .bind(&to_utc_iso)
    .fetch_all(pool)
    .await?;

    let top_services: Vec<ServiceUsageItem> = service_rows
        .into_iter()
        .map(|r| ServiceUsageItem {
            service_name: r.service_name_snapshot,
            appointment_count: r.appointment_count,
            total_minutes: r.total_minutes,
        })
        .collect();

    let completion_rate_percent = if counts.total > 0 {
        let raw = (counts.completed as f64 / counts.total as f64) * 100.0;
        Some((raw * 10.0).round() / 10.0)
    } else {
        None
    };

    Ok(OperationalReport {
        period_type: input.period_type,
        timezone: business_tz,
        anchor_date: time::format_local_date(anchor),
        period_start_local: time::format_local_date(period_start),
        period_end_local: time::format_local_date(period_end),
        period_start_utc: from_utc_iso,
        period_end_utc_exclusive: to_utc_iso,
        counts,
        completion_rate_percent,
        total_active_minutes,
        unique_clients_served: active_clients.len() as i64,
        daily_breakdown,
        top_services,
    })
}
