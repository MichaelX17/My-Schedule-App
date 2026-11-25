import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AppSettings, UpdateAppSettingsInput } from "@/services/settings";
import { SettingsPage } from "../SettingsPage";

const mockGetSettings = vi.fn<() => Promise<AppSettings>>();
const mockUpdateSettings = vi.fn<
  (input: UpdateAppSettingsInput) => Promise<AppSettings>
>();
const mockUpdateProfile = vi.fn();

vi.mock("@/services/tauri", () => ({
  isTauriRuntime: () => true,
  invokeCommand: vi.fn(),
  getAppInfo: async () => ({ name: "My Schedule App", version: "0.1.0" }),
  getStorageInfo: async () => ({
    mode: "portable",
    data_dir: "/portable/MySchedule/data",
    fallback_reason: null,
  }),
  getDatabaseHealth: async () => ({
    ok: true,
    integrity_ok: true,
    foreign_keys_enabled: true,
    journal_mode: "wal",
    applied_migrations: 7,
    latest_migration_version: 7,
  }),
}));

vi.mock("@/services/updater", () => ({
  getUpdaterInfo: async () => ({
    current_version: "0.1.0",
    platform_key: "linux-x86_64",
    github_repo: "MichaelX17/My-Schedule-App",
    releases_url: "https://github.com/MichaelX17/My-Schedule-App/releases",
    signing_key_configured: false,
  }),
  checkForUpdates: vi.fn(),
  downloadAndStageUpdate: vi.fn(),
}));

vi.mock("@/services/settings", async () => {
  const actual = await vi.importActual<typeof import("@/services/settings")>(
    "@/services/settings"
  );
  return {
    ...actual,
    getAppSettings: () => mockGetSettings(),
    updateAppSettings: (input: UpdateAppSettingsInput) =>
      mockUpdateSettings(input),
  };
});

const mockOnboardingResult = {
  data: {
    completed: true,
    detected_timezone: "Europe/Madrid",
    business: {
      id: "b1",
      name: "Studio Test",
      logo_filename: null,
      address: null,
      city: null,
      region: null,
      postal_code: null,
      country: null,
      phone: null,
      email: null,
      website: null,
      timezone: "Europe/Madrid",
      onboarding_completed: true,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
    },
  },
};

vi.mock("@/features/onboarding/hooks/useBusinessProfile", () => ({
  useOnboardingStatus: () => mockOnboardingResult,
  useBusinessLogo: () => ({ data: null }),
  useUpdateBusinessProfile: () => ({
    mutateAsync: mockUpdateProfile,
    isPending: false,
  }),
  useUploadBusinessLogo: () => ({ mutateAsync: vi.fn(), isPending: false }),
  useRemoveBusinessLogo: () => ({ mutateAsync: vi.fn(), isPending: false }),
}));

const defaultSettings: AppSettings = {
  theme_mode: "system",
  accent_color: "slate",
  reminders_enabled: true,
  reminder_one_day_before: true,
  reminder_one_hour_before: true,
  reminder_day_of_launch: true,
  os_notifications_enabled: true,
  auto_backup_enabled: true,
  auto_backup_retention: 30,
  default_appointment_duration_minutes: 60,
  updated_at: "2026-11-24T00:00:00Z",
};

function renderSettings() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <SettingsPage />
      </MemoryRouter>
    </QueryClientProvider>
  );
}

describe("SettingsPage", () => {
  beforeEach(() => {
    mockGetSettings.mockReset();
    mockUpdateSettings.mockReset();
    mockUpdateProfile.mockReset();

    mockGetSettings.mockResolvedValue(defaultSettings);
    mockUpdateSettings.mockImplementation(async (patch) => ({
      ...defaultSettings,
      ...patch,
    }));
    mockUpdateProfile.mockResolvedValue({});
  });

  it("updates theme/accent, hides storage path by default, and confirms timezone changes", async () => {
    const user = userEvent.setup();
    renderSettings();

    // Storage path is hidden by default until explicitly revealed
    expect(
      await screen.findByText(/Portable \(adjacent data folder/i)
    ).toBeInTheDocument();
    expect(
      screen.queryByText("/portable/MySchedule/data")
    ).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Reveal Path/i }));
    expect(
      screen.getByText("/portable/MySchedule/data")
    ).toBeInTheDocument();

    // Switch theme to Dark and accent to Emerald
    await user.click(screen.getByRole("button", { name: "Dark" }));
    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith({ theme_mode: "dark" });
    });

    await user.click(screen.getByRole("button", { name: "Emerald" }));
    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith({
        accent_color: "emerald",
      });
    });

    // Changing timezone requires explicit confirmation dialog
    await user.selectOptions(
      screen.getByLabelText(/Business Timezone/i),
      "America/New_York"
    );
    await user.click(
      screen.getByRole("button", { name: /Save Business Details/i })
    );

    expect(
      screen.getByRole("dialog", { name: /Confirm Business Timezone Change/i })
    ).toBeInTheDocument();
    expect(mockUpdateProfile).not.toHaveBeenCalled();

    await user.click(
      screen.getByRole("button", { name: /Confirm Timezone Change/i })
    );
    await waitFor(() => {
      expect(mockUpdateProfile).toHaveBeenCalledWith(
        expect.objectContaining({ timezone: "America/New_York" })
      );
    });
  });
});
