import { NavLink, Outlet } from "react-router-dom";
import {
  BarChart3,
  Bell,
  Briefcase,
  Building2,
  CalendarClock,
  CalendarDays,
  DatabaseBackup,
  Home,
  Moon,
  Settings,
  Sun,
  Users,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  useBusinessLogo,
  useOnboardingStatus,
} from "@/features/onboarding/hooks/useBusinessProfile";
import { NotificationToastBanner } from "@/features/reminders/components/NotificationToastBanner";
import {
  useReminderEventsSubscription,
  useUnreadNotificationCount,
} from "@/features/reminders/hooks/useReminders";
import { useUpdateAppSettings } from "@/features/settings/hooks/useAppSettings";
import { useThemeStore } from "@/stores/theme.store";

export function AppShell() {
  useReminderEventsSubscription();
  const { data: status } = useOnboardingStatus();
  const business = status?.business ?? null;
  const { data: logo } = useBusinessLogo(Boolean(business?.logo_filename));
  const { data: unreadCount = 0 } = useUnreadNotificationCount();
  const { mode, setMode } = useThemeStore();
  const updateSettings = useUpdateAppSettings();

  const toggleTheme = () => {
    const nextMode = mode === "dark" ? "light" : "dark";
    setMode(nextMode);
    void updateSettings.mutateAsync({ theme_mode: nextMode }).catch(() => {});
  };

  return (
    <div className="h-screen overflow-hidden bg-background text-foreground flex flex-col sm:flex-row">
      <aside className="w-full sm:w-60 shrink-0 sm:h-full border-b sm:border-b-0 sm:border-r border-border bg-card flex flex-col overflow-hidden">
        <div className="shrink-0 flex items-center justify-between gap-3 px-4 py-4 border-b border-border">
          <div className="flex items-center gap-2.5 min-w-0">
            {logo ? (
              <img
                src={logo.data_url}
                alt=""
                className="size-8 rounded-md object-cover border border-border shrink-0"
              />
            ) : (
              <div className="size-8 rounded-md bg-secondary text-secondary-foreground flex items-center justify-center shrink-0">
                <Building2 className="size-4" />
              </div>
            )}
            <div className="min-w-0">
              <p className="text-sm font-semibold truncate">
                {business?.name ?? "My Schedule App"}
              </p>
              <p className="text-[11px] text-muted-foreground truncate">
                My Schedule App
              </p>
            </div>
          </div>

          <Button
            variant="ghost"
            size="icon"
            onClick={toggleTheme}
            aria-label="Toggle theme"
          >
            {mode === "dark" ? (
              <Sun className="size-4" />
            ) : (
              <Moon className="size-4" />
            )}
          </Button>
        </div>

        <nav
          aria-label="Main navigation"
          className="p-2 flex sm:flex-col gap-1 overflow-x-auto sm:overflow-x-visible sm:overflow-y-auto flex-1"
        >
          <NavLink
            to="/"
            end
            className={({ isActive }) =>
              `flex items-center gap-2.5 shrink-0 whitespace-nowrap rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              }`
            }
          >
            <Home className="size-4" />
            <span>Dashboard</span>
          </NavLink>

          <NavLink
            to="/calendar"
            className={({ isActive }) =>
              `flex items-center gap-2.5 shrink-0 whitespace-nowrap rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              }`
            }
          >
            <CalendarDays className="size-4" />
            <span>Calendar</span>
          </NavLink>

          <NavLink
            to="/appointments"
            className={({ isActive }) =>
              `flex items-center gap-2.5 shrink-0 whitespace-nowrap rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              }`
            }
          >
            <CalendarClock className="size-4" />
            <span>Appointments</span>
          </NavLink>

          <NavLink
            to="/clients"
            className={({ isActive }) =>
              `flex items-center gap-2.5 shrink-0 whitespace-nowrap rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              }`
            }
          >
            <Users className="size-4" />
            <span>Clients</span>
          </NavLink>

          <NavLink
            to="/services"
            className={({ isActive }) =>
              `flex items-center gap-2.5 shrink-0 whitespace-nowrap rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              }`
            }
          >
            <Briefcase className="size-4" />
            <span>Services</span>
          </NavLink>

          <NavLink
            to="/reminders"
            className={({ isActive }) =>
              `flex items-center justify-between gap-2 shrink-0 whitespace-nowrap rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              }`
            }
          >
            <span className="flex items-center gap-2.5">
              <Bell className="size-4" />
              <span>Reminders</span>
            </span>
            {unreadCount > 0 && (
              <span
                aria-label={`${unreadCount} unread notifications`}
                className="rounded-full bg-primary px-1.5 py-0.5 text-[10px] font-semibold text-primary-foreground tabular-nums"
              >
                {unreadCount > 99 ? "99+" : unreadCount}
              </span>
            )}
          </NavLink>

          <NavLink
            to="/reports"
            className={({ isActive }) =>
              `flex items-center gap-2.5 shrink-0 whitespace-nowrap rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              }`
            }
          >
            <BarChart3 className="size-4" />
            <span>Reports</span>
          </NavLink>

          <NavLink
            to="/backups"
            className={({ isActive }) =>
              `flex items-center gap-2.5 shrink-0 whitespace-nowrap rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              }`
            }
          >
            <DatabaseBackup className="size-4" />
            <span>Backups &amp; Exports</span>
          </NavLink>

          <NavLink
            to="/settings"
            className={({ isActive }) =>
              `flex items-center gap-2.5 shrink-0 whitespace-nowrap rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              }`
            }
          >
            <Settings className="size-4" />
            <span>Settings</span>
          </NavLink>
        </nav>
      </aside>

      <main className="flex-1 min-w-0 min-h-0 flex flex-col overflow-hidden">
        <NotificationToastBanner />
        <Outlet />
      </main>
    </div>
  );
}
