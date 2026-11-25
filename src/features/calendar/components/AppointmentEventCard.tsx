import { Link } from "react-router-dom";
import { CheckCircle2, Clock, FolderKanban, XCircle } from "lucide-react";
import type {
  AppointmentDetail,
  AppointmentStatus,
} from "@/services/appointments";
import {
  extractLocalTime,
  formatAppointmentDuration,
  formatAppointmentTimeForDay,
} from "../utils/calendarDate";

interface StatusPresentation {
  label: string;
  shortLabel: string;
  icon: typeof Clock;
  cardClasses: string;
  badgeClasses: string;
  iconClasses: string;
}

function getStatusPresentation(
  status: AppointmentStatus
): StatusPresentation {
  switch (status) {
    case "completed":
      return {
        label: "Completed",
        shortLabel: "Done",
        icon: CheckCircle2,
        cardClasses:
          "border-l-[3px] border-l-emerald-500 bg-emerald-500/10 hover:bg-emerald-500/15 dark:bg-emerald-500/12 dark:hover:bg-emerald-500/20",
        badgeClasses:
          "bg-emerald-500/18 text-emerald-700 dark:text-emerald-300",
        iconClasses: "text-emerald-600 dark:text-emerald-400",
      };
    case "cancelled":
      return {
        label: "Cancelled",
        shortLabel: "Cancelled",
        icon: XCircle,
        cardClasses:
          "border-l-[3px] border-l-muted-foreground/50 bg-muted/60 hover:bg-muted/85 opacity-80",
        badgeClasses: "bg-muted text-muted-foreground",
        iconClasses: "text-muted-foreground",
      };
    case "scheduled":
    default:
      return {
        label: "Scheduled",
        shortLabel: "Scheduled",
        icon: Clock,
        cardClasses:
          "border-l-[3px] border-l-primary bg-secondary/70 hover:bg-accent/70",
        badgeClasses: "bg-primary/12 text-primary",
        iconClasses: "text-primary",
      };
  }
}

interface AppointmentEventCardProps {
  appointment: AppointmentDetail;
  dayYmd: string;
  variant?: "compact" | "standard" | "detailed";
}

export function AppointmentEventCard({
  appointment,
  dayYmd,
  variant = "standard",
}: AppointmentEventCardProps) {
  const status = getStatusPresentation(appointment.status);
  const StatusIcon = status.icon;
  const timeLabel = formatAppointmentTimeForDay(appointment, dayYmd);
  const startTime = extractLocalTime(appointment.start_local);
  const durationLabel = formatAppointmentDuration(appointment);
  const serviceSummary = appointment.services
    .map((s) => s.service_name_snapshot)
    .join(", ");

  const ariaLabel = `${timeLabel}, ${appointment.client_name}, ${serviceSummary || "Appointment"}, status ${status.label}`;
  const tooltipLines = [
    `[${status.label}] ${timeLabel}${durationLabel ? ` (${durationLabel})` : ""}`,
    `Client: ${appointment.client_name}`,
    serviceSummary ? `Services: ${serviceSummary}` : null,
    appointment.session_plan_title
      ? `Session Plan: ${appointment.session_plan_title}`
      : null,
    appointment.notes ? `Notes: ${appointment.notes}` : null,
  ].filter(Boolean);
  const tooltipText = tooltipLines.join("\n");

  if (variant === "compact") {
    return (
      <Link
        to={`/appointments/${appointment.id}`}
        aria-label={ariaLabel}
        title={tooltipText}
        className={`group block w-full rounded-md px-2 py-1.5 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${status.cardClasses}`}
      >
        <div className="flex items-center justify-between gap-1 min-w-0">
          <span className="inline-flex items-center gap-1 min-w-0">
            <StatusIcon
              className={`size-3 shrink-0 ${status.iconClasses}`}
              aria-hidden="true"
            />
            <span className="text-[11px] font-semibold tabular-nums text-foreground/80 whitespace-nowrap">
              {startTime}
            </span>
          </span>
          {durationLabel && (
            <span className="text-[10px] tabular-nums text-muted-foreground whitespace-nowrap shrink-0">
              {durationLabel}
            </span>
          )}
        </div>
        <p
          className={`mt-0.5 text-xs font-semibold leading-snug text-foreground truncate ${
            appointment.status === "cancelled" ? "line-through" : ""
          }`}
        >
          {appointment.client_name}
        </p>
        {serviceSummary && (
          <p className="text-[11px] leading-snug text-muted-foreground truncate">
            {serviceSummary}
          </p>
        )}
      </Link>
    );
  }

  if (variant === "standard") {
    return (
      <Link
        to={`/appointments/${appointment.id}`}
        aria-label={ariaLabel}
        title={tooltipText}
        className={`group block w-full rounded-lg p-2.5 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${status.cardClasses}`}
      >
        <div className="flex items-center justify-between gap-1.5 min-w-0">
          <span className="text-[11px] font-semibold tabular-nums text-foreground/80 whitespace-nowrap">
            {timeLabel}
          </span>
          {durationLabel && (
            <span className="text-[10px] font-medium tabular-nums text-muted-foreground whitespace-nowrap shrink-0">
              {durationLabel}
            </span>
          )}
        </div>

        <p
          className={`mt-1 text-[13px] font-semibold leading-snug text-foreground line-clamp-2 break-words ${
            appointment.status === "cancelled" ? "line-through" : ""
          }`}
        >
          {appointment.client_name}
        </p>

        {serviceSummary && (
          <p className="mt-0.5 text-[11px] leading-snug text-muted-foreground line-clamp-2 break-words">
            {serviceSummary}
          </p>
        )}

        <div className="mt-2 flex flex-wrap items-center gap-1">
          <span
            className={`inline-flex items-center gap-1 rounded-full px-1.5 py-0.5 text-[10px] font-medium whitespace-nowrap ${status.badgeClasses}`}
          >
            <StatusIcon className="size-2.5 shrink-0" aria-hidden="true" />
            <span>{status.shortLabel}</span>
          </span>
          {appointment.session_plan_title && (
            <span
              className="inline-flex items-center gap-1 rounded-full bg-background/70 px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground max-w-full"
              title={appointment.session_plan_title}
            >
              <FolderKanban className="size-2.5 shrink-0" aria-hidden="true" />
              <span className="truncate">{appointment.session_plan_title}</span>
            </span>
          )}
        </div>
      </Link>
    );
  }

  return (
    <Link
      to={`/appointments/${appointment.id}`}
      aria-label={ariaLabel}
      title={tooltipText}
      className={`group block w-full rounded-lg p-3 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${status.cardClasses}`}
    >
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-xs font-semibold tabular-nums text-foreground/85 whitespace-nowrap">
              {timeLabel}
            </span>
            {durationLabel && (
              <span className="rounded-full bg-background/70 px-1.5 py-0.5 text-[10px] font-medium tabular-nums text-muted-foreground whitespace-nowrap">
                {durationLabel}
              </span>
            )}
          </div>
          <p
            className={`text-sm font-semibold leading-snug text-foreground mt-1 break-words ${
              appointment.status === "cancelled" ? "line-through" : ""
            }`}
          >
            {appointment.client_name}
            {appointment.client_is_archived && (
              <span className="ml-1.5 text-xs font-normal text-muted-foreground">
                (Archived)
              </span>
            )}
          </p>
        </div>

        <span
          className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium whitespace-nowrap shrink-0 ${status.badgeClasses}`}
        >
          <StatusIcon className="size-3 shrink-0" aria-hidden="true" />
          <span>{status.label}</span>
        </span>
      </div>

      {appointment.services.length > 0 && (
        <div className="mt-2 flex flex-wrap items-center gap-1.5">
          {appointment.services.map((svc) => (
            <span
              key={svc.id}
              className="inline-flex items-center gap-1 rounded-md bg-background/70 px-2 py-0.5 text-xs text-foreground/90"
            >
              <span>{svc.service_name_snapshot}</span>
              <span className="text-[11px] text-muted-foreground tabular-nums">
                ({svc.duration_minutes_snapshot}m)
              </span>
            </span>
          ))}
        </div>
      )}

      {appointment.session_plan_title && (
        <div className="mt-2 inline-flex items-center gap-1.5 text-[11px] font-medium text-muted-foreground">
          <FolderKanban className="size-3 shrink-0" aria-hidden="true" />
          <span className="truncate">{appointment.session_plan_title}</span>
        </div>
      )}

      {appointment.notes && (
        <p className="mt-1.5 text-xs text-muted-foreground line-clamp-2 break-words">
          {appointment.notes}
        </p>
      )}
    </Link>
  );
}
