import { useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { AlertCircle, Info } from "lucide-react";
import { AppointmentFormDialog } from "@/features/appointments/components/AppointmentFormDialog";
import {
  useCreateAppointment,
  useSessionPlansList,
} from "@/features/appointments/hooks/useAppointments";
import { useClientsList } from "@/features/clients/hooks/useClients";
import { useOnboardingStatus } from "@/features/onboarding/hooks/useBusinessProfile";
import { useServicesList } from "@/features/services/hooks/useServices";
import type { AppointmentStatus } from "@/services/appointments";
import { extractErrorMessage } from "@/services/business";
import type { CalendarView } from "@/services/calendar";
import { AgendaView } from "./components/AgendaView";
import { CalendarToolbar } from "./components/CalendarToolbar";
import { DayView } from "./components/DayView";
import { MonthView } from "./components/MonthView";
import { WeekView } from "./components/WeekView";
import { useCalendarRange } from "./hooks/useCalendar";
import {
  formatRangeHeading,
  isValidYmd,
  stepAnchorDate,
} from "./utils/calendarDate";

const VALID_VIEWS: CalendarView[] = ["day", "week", "month", "agenda"];

function parseViewParam(raw: string | null): CalendarView {
  if (raw && VALID_VIEWS.includes(raw as CalendarView)) {
    return raw as CalendarView;
  }
  return "week";
}

function useIsNarrowWindow(): boolean {
  const [isNarrow, setIsNarrow] = useState<boolean>(() => {
    if (typeof window === "undefined" || !window.matchMedia) return false;
    return window.matchMedia("(max-width: 767px)").matches;
  });

  useEffect(() => {
    if (typeof window === "undefined" || !window.matchMedia) return;
    const mq = window.matchMedia("(max-width: 767px)");
    const listener = (event: MediaQueryListEvent) => setIsNarrow(event.matches);
    mq.addEventListener("change", listener);
    return () => mq.removeEventListener("change", listener);
  }, []);

  return isNarrow;
}

export function CalendarPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [statusFilter, setStatusFilter] = useState<AppointmentStatus | "all">(
    "all"
  );
  const [createApptOpen, setCreateApptOpen] = useState(false);

  const isNarrow = useIsNarrowWindow();
  const requestedView = parseViewParam(searchParams.get("view"));
  const dateParam = searchParams.get("date");
  const anchorParam = isValidYmd(dateParam) ? dateParam : null;

  // On narrow windows, Week and Month multi-column grids automatically fall back
  // to compact Agenda view while preserving the selected view preference.
  const effectiveView: CalendarView =
    isNarrow && (requestedView === "week" || requestedView === "month")
      ? "agenda"
      : requestedView;

  const { data: onboarding } = useOnboardingStatus();
  const fallbackTimezone = onboarding?.business?.timezone ?? "UTC";

  const { data, isLoading, isFetching, isError, error } = useCalendarRange({
    view: effectiveView,
    anchor_date: anchorParam,
    status: statusFilter === "all" ? null : statusFilter,
  });

  const { data: clientsData } = useClientsList({
    filter: "active",
    limit: 200,
    offset: 0,
  });
  const { data: servicesData } = useServicesList({ filter: "active" });
  const { data: plans = [] } = useSessionPlansList({});
  const createAppointment = useCreateAppointment();

  const timezone = data?.timezone ?? fallbackTimezone;
  const activeAnchor = data?.anchor_date ?? anchorParam ?? "";
  const todayLocal = data?.today_local ?? activeAnchor;
  const rangeStart = data?.range_start_local ?? activeAnchor;
  const rangeEnd = data?.range_end_local ?? activeAnchor;
  const appointments = data?.appointments ?? [];

  const updateNavigation = (nextView: CalendarView, nextDate: string | null) => {
    const next = new URLSearchParams(searchParams);
    next.set("view", nextView);
    if (nextDate) {
      next.set("date", nextDate);
    } else {
      next.delete("date");
    }
    setSearchParams(next, { replace: false });
  };

  const handleViewChange = (nextView: CalendarView) => {
    updateNavigation(nextView, anchorParam);
  };

  const handlePrevious = () => {
    if (!activeAnchor) return;
    const prevDate = stepAnchorDate(effectiveView, activeAnchor, -1);
    updateNavigation(requestedView, prevDate);
  };

  const handleNext = () => {
    if (!activeAnchor) return;
    const nextDate = stepAnchorDate(effectiveView, activeAnchor, 1);
    updateNavigation(requestedView, nextDate);
  };

  const handleToday = () => {
    updateNavigation(requestedView, data?.today_local ?? null);
  };

  const handleDateChange = (ymd: string) => {
    if (isValidYmd(ymd)) {
      updateNavigation(requestedView, ymd);
    }
  };

  const handleSelectDay = (dayYmd: string) => {
    updateNavigation("day", dayYmd);
  };

  const rangeHeading = useMemo(() => {
    if (!activeAnchor || !rangeStart || !rangeEnd) {
      return "Loading calendar...";
    }
    return formatRangeHeading(effectiveView, activeAnchor, rangeStart, rangeEnd);
  }, [effectiveView, activeAnchor, rangeStart, rangeEnd]);

  const narrowFallbackActive =
    isNarrow && (requestedView === "week" || requestedView === "month");

  return (
    <div className="mx-auto w-full max-w-6xl p-4 sm:p-6 lg:p-8 space-y-5">
      <CalendarToolbar
        view={requestedView}
        anchorDate={activeAnchor}
        rangeHeading={rangeHeading}
        timezone={timezone}
        statusFilter={statusFilter}
        onViewChange={handleViewChange}
        onPrevious={handlePrevious}
        onNext={handleNext}
        onToday={handleToday}
        onDateChange={handleDateChange}
        onStatusFilterChange={setStatusFilter}
        onNewAppointment={() => setCreateApptOpen(true)}
      />

      {narrowFallbackActive && (
        <div
          role="status"
          className="flex items-center gap-2 rounded-lg border border-border bg-muted/40 px-3 py-2 text-xs text-muted-foreground"
        >
          <Info className="size-4 shrink-0" aria-hidden="true" />
          <span>
            Showing compact Agenda view for narrow windows. Widen the window to
            view the full {requestedView} grid, or switch to Day view.
          </span>
        </div>
      )}

      {isError ? (
        <div
          role="alert"
          className="flex items-start gap-2.5 rounded-xl border border-destructive/40 bg-card p-4 text-sm text-destructive"
        >
          <AlertCircle className="size-4 mt-0.5 shrink-0" />
          <div>
            <p className="font-medium">Unable to load calendar appointments</p>
            <p className="text-xs text-muted-foreground mt-0.5">
              {extractErrorMessage(error)}
            </p>
          </div>
        </div>
      ) : isLoading && !data ? (
        <div className="rounded-xl border border-border bg-card py-16 text-center text-sm text-muted-foreground">
          Loading schedule...
        </div>
      ) : (
        <div className={isFetching ? "opacity-80 transition-opacity" : ""}>
          {effectiveView === "day" && (
            <DayView
              dayYmd={activeAnchor}
              todayYmd={todayLocal}
              appointments={appointments}
            />
          )}

          {effectiveView === "week" && (
            <WeekView
              rangeStartYmd={rangeStart}
              rangeEndYmd={rangeEnd}
              todayYmd={todayLocal}
              appointments={appointments}
              onSelectDay={handleSelectDay}
            />
          )}

          {effectiveView === "month" && (
            <MonthView
              anchorDate={activeAnchor}
              rangeStartYmd={rangeStart}
              rangeEndYmd={rangeEnd}
              todayYmd={todayLocal}
              appointments={appointments}
              onSelectDay={handleSelectDay}
            />
          )}

          {effectiveView === "agenda" && (
            <AgendaView
              rangeStartYmd={rangeStart}
              rangeEndYmd={rangeEnd}
              todayYmd={todayLocal}
              appointments={appointments}
              onSelectDay={handleSelectDay}
            />
          )}
        </div>
      )}

      {createApptOpen && (
        <AppointmentFormDialog
          open={createApptOpen}
          mode="create"
          clients={clientsData?.items ?? []}
          services={servicesData?.items ?? []}
          sessionPlans={plans}
          businessTimezone={timezone}
          isSubmitting={createAppointment.isPending}
          onClose={() => setCreateApptOpen(false)}
          onCreate={async (input) => {
            await createAppointment.mutateAsync(input);
            setCreateApptOpen(false);
          }}
        />
      )}
    </div>
  );
}
