import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface DateTimeLocalPickerProps {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  required?: boolean;
}

const HOURS = Array.from({ length: 24 }, (_, i) => String(i).padStart(2, "0"));
const MINUTES = Array.from({ length: 60 }, (_, i) => String(i).padStart(2, "0"));

function parseDateTimeLocal(value: string): {
  date: string;
  hour: string;
  minute: string;
} {
  const [datePart = "", timePart = ""] = value.split("T");
  const [rawHour = "09", rawMinute = "00"] = timePart.split(":");
  const hour = HOURS.includes(rawHour.padStart(2, "0"))
    ? rawHour.padStart(2, "0")
    : "09";
  const minute = MINUTES.includes(rawMinute.padStart(2, "0"))
    ? rawMinute.padStart(2, "0")
    : "00";
  return { date: datePart, hour, minute };
}

/**
 * Cross-platform Date + Hour + Minute picker that outputs `YYYY-MM-DDTHH:MM`.
 * Avoids `<input type="datetime-local">`, which omits hour/minute controls on
 * Linux WebKitGTK.
 */
export function DateTimeLocalPicker({
  id,
  label,
  value,
  onChange,
  required = false,
}: DateTimeLocalPickerProps) {
  const { date, hour, minute } = parseDateTimeLocal(value);

  const emit = (nextDate: string, nextHour: string, nextMinute: string) => {
    if (!nextDate) {
      onChange("");
      return;
    }
    onChange(`${nextDate}T${nextHour}:${nextMinute}`);
  };

  return (
    <div className="space-y-1.5">
      <Label htmlFor={`${id}-date`}>
        {label}
        {required && (
          <>
            {" "}
            <span className="text-destructive">*</span>
          </>
        )}
      </Label>
      <div className="flex flex-wrap sm:flex-nowrap items-center gap-2">
        <Input
          id={`${id}-date`}
          aria-label={`${label} Date`}
          type="date"
          value={date}
          onChange={(e) => emit(e.target.value, hour, minute)}
          className="min-w-0 flex-1"
        />
        <div className="flex items-center gap-1 shrink-0">
          <select
            id={`${id}-hour`}
            aria-label={`${label} Hour`}
            value={hour}
            onChange={(e) => emit(date, e.target.value, minute)}
            className="h-9 rounded-md border border-input bg-background px-2.5 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            {HOURS.map((h) => (
              <option key={h} value={h}>
                {h}
              </option>
            ))}
          </select>
          <span
            className="text-sm font-medium text-muted-foreground"
            aria-hidden="true"
          >
            :
          </span>
          <select
            id={`${id}-minute`}
            aria-label={`${label} Minute`}
            value={minute}
            onChange={(e) => emit(date, hour, e.target.value)}
            className="h-9 rounded-md border border-input bg-background px-2.5 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            {MINUTES.map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
        </div>
      </div>
    </div>
  );
}
