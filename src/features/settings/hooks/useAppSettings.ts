import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  getAppSettings,
  updateAppSettings,
  type UpdateAppSettingsInput,
} from "@/services/settings";
import { isTauriRuntime } from "@/services/tauri";
import { useThemeStore } from "@/stores/theme.store";

export const SETTINGS_QUERY_KEYS = {
  all: ["settings"] as const,
};

export function useAppSettings() {
  return useQuery({
    queryKey: SETTINGS_QUERY_KEYS.all,
    queryFn: getAppSettings,
    enabled: isTauriRuntime(),
    staleTime: 60_000,
  });
}

export function useUpdateAppSettings() {
  const queryClient = useQueryClient();
  const setMode = useThemeStore((s) => s.setMode);
  const setAccent = useThemeStore((s) => s.setAccent);

  return useMutation({
    mutationFn: (input: UpdateAppSettingsInput) => updateAppSettings(input),
    onSuccess: async (saved) => {
      setMode(saved.theme_mode);
      setAccent(saved.accent_color);
      queryClient.setQueryData(SETTINGS_QUERY_KEYS.all, saved);
      await queryClient.invalidateQueries({ queryKey: ["backups"] });
    },
  });
}
