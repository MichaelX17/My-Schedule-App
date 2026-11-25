import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { AppShell } from "@/components/layout/AppShell";
import { AppointmentDetailPage } from "@/features/appointments/AppointmentDetailPage";
import { AppointmentsPage } from "@/features/appointments/AppointmentsPage";
import { BackupsExportsPage } from "@/features/backups/BackupsExportsPage";
import { CalendarPage } from "@/features/calendar/CalendarPage";
import { ClientDetailPage } from "@/features/clients/ClientDetailPage";
import { ClientsPage } from "@/features/clients/ClientsPage";
import { DashboardPage } from "@/features/dashboard/DashboardPage";
import { useOnboardingStatus } from "@/features/onboarding/hooks/useBusinessProfile";
import { OnboardingWizard } from "@/features/onboarding/OnboardingWizard";
import { RemindersPage } from "@/features/reminders/RemindersPage";
import { ReportsPage } from "@/features/reports/ReportsPage";
import { ServicesPage } from "@/features/services/ServicesPage";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { extractErrorMessage } from "@/services/business";
import { isTauriRuntime } from "@/services/tauri";

export function AppRouter() {
  const inTauri = isTauriRuntime();
  const { data: status, isLoading, isError, error } = useOnboardingStatus();

  if (!inTauri) {
    return (
      <div className="min-h-screen bg-background text-foreground flex items-center justify-center p-6">
        <div className="w-full max-w-md rounded-xl border border-border bg-card p-6 text-card-foreground shadow-sm space-y-2">
          <h1 className="text-lg font-semibold">Desktop Backend Required</h1>
          <p className="text-sm text-muted-foreground">
            My Schedule App stores business data in a local SQLite database
            managed by the Tauri desktop backend. Launch the app via{" "}
            <code className="font-mono text-xs">npm run tauri dev</code>.
          </p>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="min-h-screen bg-background text-foreground flex items-center justify-center p-6">
        <p className="text-sm text-muted-foreground">
          Loading local workspace...
        </p>
      </div>
    );
  }

  if (isError || !status) {
    return (
      <div className="min-h-screen bg-background text-foreground flex items-center justify-center p-6">
        <div
          role="alert"
          className="w-full max-w-md rounded-xl border border-destructive/40 bg-card p-6 space-y-2"
        >
          <h1 className="text-lg font-semibold text-destructive">
            Unable to Load Business Profile
          </h1>
          <p className="text-sm text-muted-foreground">
            {extractErrorMessage(error)}
          </p>
        </div>
      </div>
    );
  }

  if (!status.completed) {
    return (
      <BrowserRouter>
        <Routes>
          <Route
            path="/onboarding"
            element={
              <OnboardingWizard detectedTimezone={status.detected_timezone} />
            }
          />
          <Route path="*" element={<Navigate to="/onboarding" replace />} />
        </Routes>
      </BrowserRouter>
    );
  }

  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route path="/" element={<DashboardPage />} />
          <Route path="/calendar" element={<CalendarPage />} />
          <Route path="/appointments" element={<AppointmentsPage />} />
          <Route
            path="/appointments/:appointmentId"
            element={<AppointmentDetailPage />}
          />
          <Route path="/clients" element={<ClientsPage />} />
          <Route path="/clients/:clientId" element={<ClientDetailPage />} />
          <Route path="/services" element={<ServicesPage />} />
          <Route path="/reminders" element={<RemindersPage />} />
          <Route path="/reports" element={<ReportsPage />} />
          <Route path="/backups" element={<BackupsExportsPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
