import { useQuery } from "@tanstack/react-query";
import {
  getCalendarRange,
  type CalendarRangeInput,
} from "@/services/calendar";
import { isTauriRuntime } from "@/services/tauri";

export const CALENDAR_QUERY_KEYS = {
  all: ["calendar"] as const,
  range: (input: CalendarRangeInput) => ["calendar", "range", input] as const,
};

export function useCalendarRange(input: CalendarRangeInput) {
  return useQuery({
    queryKey: CALENDAR_QUERY_KEYS.range(input),
    queryFn: () => getCalendarRange(input),
    enabled: isTauriRuntime(),
    placeholderData: (prev) => prev,
  });
}
