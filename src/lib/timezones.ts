const FALLBACK_TIMEZONES = [
  "UTC",
  "America/New_York",
  "America/Chicago",
  "America/Denver",
  "America/Los_Angeles",
  "America/Toronto",
  "America/Mexico_City",
  "America/Bogota",
  "America/Lima",
  "America/Santiago",
  "America/Buenos_Aires",
  "America/Sao_Paulo",
  "Europe/London",
  "Europe/Dublin",
  "Europe/Lisbon",
  "Europe/Madrid",
  "Europe/Paris",
  "Europe/Berlin",
  "Europe/Rome",
  "Europe/Amsterdam",
  "Europe/Zurich",
  "Europe/Warsaw",
  "Europe/Athens",
  "Asia/Dubai",
  "Asia/Kolkata",
  "Asia/Singapore",
  "Asia/Tokyo",
  "Asia/Seoul",
  "Australia/Sydney",
  "Pacific/Auckland",
];

/**
 * Returns the browser/WebView's supported IANA timezone identifiers,
 * always ensuring `UTC` and any provided `extraTimezone` are included.
 */
export function getAvailableTimezones(extraTimezone?: string): string[] {
  let zones: string[];
  try {
    const supported = Intl.supportedValuesOf?.("timeZone");
    zones = supported && supported.length > 0 ? [...supported] : [...FALLBACK_TIMEZONES];
  } catch {
    zones = [...FALLBACK_TIMEZONES];
  }

  const set = new Set(zones);
  set.add("UTC");
  if (extraTimezone && extraTimezone.trim().length > 0) {
    set.add(extraTimezone.trim());
  }

  return Array.from(set).sort((a, b) => a.localeCompare(b));
}

export function detectBrowserTimezone(): string {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
  } catch {
    return "UTC";
  }
}
