import { useEffect, type ReactNode } from "react";
import { getAppSettings } from "@/services/settings";
import { isTauriRuntime } from "@/services/tauri";
import { useThemeStore } from "@/stores/theme.store";

interface ThemeProviderProps {
  children: ReactNode;
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const mode = useThemeStore((state) => state.mode);
  const accent = useThemeStore((state) => state.accent);
  const setMode = useThemeStore((state) => state.setMode);
  const setAccent = useThemeStore((state) => state.setAccent);

  // Hydrate persisted theme & accent from SQLite on desktop startup.
  useEffect(() => {
    if (!isTauriRuntime()) return;
    let cancelled = false;
    getAppSettings()
      .then((settings) => {
        if (cancelled) return;
        setMode(settings.theme_mode);
        setAccent(settings.accent_color);
      })
      .catch(() => {
        // Keep safe defaults (system + slate) if settings load fails.
      });
    return () => {
      cancelled = true;
    };
  }, [setMode, setAccent]);

  useEffect(() => {
    const root = document.documentElement;
    root.dataset.accent = accent;

    const mediaQuery = window.matchMedia
      ? window.matchMedia("(prefers-color-scheme: dark)")
      : null;

    const applyTheme = () => {
      const resolvedDark =
        mode === "dark" || (mode === "system" && Boolean(mediaQuery?.matches));
      root.classList.toggle("dark", resolvedDark);
    };

    applyTheme();
    mediaQuery?.addEventListener("change", applyTheme);
    return () => mediaQuery?.removeEventListener("change", applyTheme);
  }, [mode, accent]);

  return <>{children}</>;
}
