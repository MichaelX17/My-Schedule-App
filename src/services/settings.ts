import { invokeCommand } from "@/services/tauri";

export type ThemeMode = "light" | "dark" | "system";
export type AccentColor =
  | "slate"
  | "blue"
  | "emerald"
  | "violet"
  | "amber"
  | "rose";

export interface AppSettings {
  theme_mode: ThemeMode;
  accent_color: AccentColor;
  reminders_enabled: boolean;
  reminder_one_day_before: boolean;
  reminder_one_hour_before: boolean;
  reminder_day_of_launch: boolean;
  os_notifications_enabled: boolean;
  auto_backup_enabled: boolean;
  auto_backup_retention: number;
  default_appointment_duration_minutes: number;
  updated_at: string;
}

export interface UpdateAppSettingsInput {
  theme_mode?: ThemeMode;
  accent_color?: AccentColor;
  reminders_enabled?: boolean;
  reminder_one_day_before?: boolean;
  reminder_one_hour_before?: boolean;
  reminder_day_of_launch?: boolean;
  os_notifications_enabled?: boolean;
  auto_backup_enabled?: boolean;
  auto_backup_retention?: number;
  default_appointment_duration_minutes?: number;
}

export async function getAppSettings(): Promise<AppSettings> {
  return invokeCommand<AppSettings>("get_app_settings");
}

export async function updateAppSettings(
  input: UpdateAppSettingsInput
): Promise<AppSettings> {
  return invokeCommand<AppSettings>("update_app_settings", { input });
}
