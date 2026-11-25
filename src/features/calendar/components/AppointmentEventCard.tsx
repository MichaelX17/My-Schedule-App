import { Link } from "react-router-dom";
import { CheckCircle2, Clock, FolderKanban, XCircle } from "lucide-react";
import type {
  AppointmentDetail,
  AppointmentStatus,
} from "@/services/appointments";
import { formatAppointmentTimeForDay } from "../utils/calendarDate";

interface StatusPresentation {
  label: string;
  shortLabel: string;
  icon: typeof Clock;
  cardClasses: string;
  badgeClasses: string;
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
          "border-l-4 border-l-emerald-600/80 dark:border-l-emerald-400/80 border-border bg-emerald-500/5 hover:bg-emerald-500/10",
        badgeClasses:
          "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300",
      };
    case "cancelled":
      return {
        label: "Cancelled",
        shortLabel: "Cancelled",
        icon: XCircle,
        cardClasses:
          "border-l-4 border-l-muted-foreground/60 border-border bg-muted/40 hover:bg-muted/70 opacity-85",
        badgeClasses: "bg-muted text-muted-foreground",
      };
    case "scheduled":
    default:
      return {
        label: "Scheduled",
        shortLabel: "Scheduled",
        icon: Clock,
        cardClasses:
          "border-l-4 border-l-primary border-border bg-card hover:bg-accent/50",
        badgeClasses: "bg-primary/10 text-primary",
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
  const serviceSummary = appointment.services
    .map((s) => s.service_name_snapshot)
    .join(", ");

  const ariaLabel = `${timeLabel}, ${appointment.client_name}, ${serviceSummary || "Appointment"}, status ${status.label}`;

  if (variant === "compact") {
    return (
      <Link
        to={`/appointments/${appointment.id}`}
        aria-label={ariaLabel}
        className={`group block w-full rounded border px-1.5 py-1 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${status.cardClasses}`}
      >
        <div className="flex items-center gap-1 min-w-0">
          <StatusIcon className="size-3 shrink-0 text-muted-foreground" aria-hidden="true" />
          <span className="text-[11px] font-medium tabular-nums text-muted-foreground shrink-0">
            {timeLabel.split("–")[0]}
          </span>
          <span
            className={`text-xs font-medium truncate ${
              appointment.status === "cancelled" ? "line-through" : ""
            }`}
          >
            {appointment.client_name}
          </span>
        </div>
        {serviceSummary && (
          <p className="text-[11px] text-muted-foreground truncate pl-4">
            {serviceSummary}
          </p>
        )}
      </Link>
    );
  }

  return (
    <Link
      to={`/appointments/${appointment.id}`}
      aria-label={ariaLabel}
      className={`group block w-full rounded-lg border p-2.5 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${status.cardClasses}`}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          <p className="text-xs font-medium tabular-nums text-muted-foreground">
            {timeLabel}
          </p>
          <p
            className={`text-sm font-semibold truncate mt-0.5 ${
              appointment.status === "cancelled" ? "line-through" : ""
            }`}
          >
            {appointment.client_name}
          </p>
        </div>

        <span
          className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium shrink-0 ${status.badgeClasses}`}
        >
          <StatusIcon className="size-3" aria-hidden="true" />
          <span>{variant === "detailed" ? status.label : status.shortLabel}</span>
        </span>
      </div>

      {serviceSummary && (
        <p className="mt-1 text-xs text-muted-foreground truncate">
          {serviceSummary}
        </p>
      )}

      {variant === "detailed" && appointment.session_plan_title && (
        <div className="mt-1.5 inline-flex items-center gap-1 text-[11px] text-muted-foreground">
          <FolderKanban className="size-3 shrink-0" aria-hidden="true" />
          <span className="truncate">{appointment.session_plan_title}</span>
        </div>
      )}
    </Link>
  );
}
