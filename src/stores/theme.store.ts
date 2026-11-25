import { create } from "zustand";
import type { AccentColor, ThemeMode } from "@/services/settings";

export type { AccentColor, ThemeMode };

interface ThemeState {
  mode: ThemeMode;
  accent: AccentColor;
  setMode: (mode: ThemeMode) => void;
  setAccent: (accent: AccentColor) => void;
}

export const useThemeStore = create<ThemeState>((set) => ({
  mode: "system",
  accent: "slate",
  setMode: (mode) => set({ mode }),
  setAccent: (accent) => set({ accent }),
}));
