import { useEffect, useMemo, useRef, useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import {
  AlertTriangle,
  Bell,
  CheckCircle2,
  Copy,
  DatabaseBackup,
  Eye,
  EyeOff,
  HardDrive,
  ImagePlus,
  Info,
  Monitor,
  Moon,
  Palette,
  Sun,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  useBusinessLogo,
  useOnboardingStatus,
  useRemoveBusinessLogo,
  useUpdateBusinessProfile,
  useUploadBusinessLogo,
} from "@/features/onboarding/hooks/useBusinessProfile";
import { getAvailableTimezones } from "@/lib/timezones";
import { extractErrorMessage } from "@/services/business";
import type { AccentColor, ThemeMode } from "@/services/settings";
import {
  getAppInfo,
  getDatabaseHealth,
  getStorageInfo,
  isTauriRuntime,
} from "@/services/tauri";
import { useAppSettings, useUpdateAppSettings } from "./hooks/useAppSettings";
import { UpdatesSection } from "./components/UpdatesSection";

const MAX_LOGO_BYTES = 2 * 1024 * 1024;
const ALLOWED_LOGO_TYPES = ["image/png", "image/jpeg", "image/webp"];

const THEME_OPTIONS: { value: ThemeMode; label: string; icon: typeof Sun }[] = [
  { value: "light", label: "Light", icon: Sun },
  { value: "dark", label: "Dark", icon: Moon },
  { value: "system", label: "System", icon: Monitor },
];

const ACCENT_OPTIONS: {
  value: AccentColor;
  label: string;
  swatchClass: string;
}[] = [
  { value: "slate", label: "Slate", swatchClass: "bg-slate-700 dark:bg-slate-200" },
  { value: "blue", label: "Blue", swatchClass: "bg-blue-600 dark:bg-blue-400" },
  { value: "emerald", label: "Emerald", swatchClass: "bg-emerald-600 dark:bg-emerald-400" },
  { value: "violet", label: "Violet", swatchClass: "bg-violet-600 dark:bg-violet-400" },
  { value: "amber", label: "Amber", swatchClass: "bg-amber-600 dark:bg-amber-400" },
  { value: "rose", label: "Rose", swatchClass: "bg-rose-600 dark:bg-rose-400" },
];

const profileSchema = z.object({
  name: z
    .string()
    .trim()
    .min(1, "Business name is required.")
    .max(120, "Business name must be 120 characters or fewer."),
  address: z.string().max(255).optional(),
  city: z.string().max(255).optional(),
  region: z.string().max(255).optional(),
  postal_code: z.string().max(255).optional(),
  country: z.string().max(255).optional(),
  phone: z.string().max(255).optional(),
  email: z
    .string()
    .max(255)
    .optional()
    .refine(
      (val) => {
        if (!val || val.trim().length === 0) return true;
        return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(val.trim());
      },
      { message: "Please enter a valid email address or leave it blank." }
    ),
  website: z.string().max(255).optional(),
  timezone: z.string().trim().min(1, "Business timezone is required."),
});

type ProfileFormValues = z.infer<typeof profileSchema>;

export function SettingsPage() {
  const { data: status } = useOnboardingStatus();
  const business = status?.business ?? null;

  const { data: logo } = useBusinessLogo(Boolean(business?.logo_filename));
  const { data: appSettings } = useAppSettings();
  const { data: appInfo } = useQuery({
    queryKey: ["system", "app-info"],
    queryFn: getAppInfo,
    enabled: isTauriRuntime(),
    staleTime: Infinity,
  });
  const { data: storageInfo } = useQuery({
    queryKey: ["system", "storage-info"],
    queryFn: getStorageInfo,
    enabled: isTauriRuntime(),
    staleTime: 60_000,
  });
  const { data: dbHealth } = useQuery({
    queryKey: ["system", "db-health"],
    queryFn: getDatabaseHealth,
    enabled: isTauriRuntime(),
    staleTime: 60_000,
  });

  const updateProfile = useUpdateBusinessProfile();
  const uploadLogo = useUploadBusinessLogo();
  const removeLogo = useRemoveBusinessLogo();
  const updateSettings = useUpdateAppSettings();

  const [saveMessage, setSaveMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [pendingTimezoneSubmit, setPendingTimezoneSubmit] =
    useState<ProfileFormValues | null>(null);
  const [revealStoragePath, setRevealStoragePath] = useState(false);
  const [retentionDraft, setRetentionDraft] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const retentionInput =
    retentionDraft ?? String(appSettings?.auto_backup_retention ?? 30);

  const timezones = useMemo(
    () => getAvailableTimezones(business?.timezone),
    [business?.timezone]
  );

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isDirty },
  } = useForm<ProfileFormValues>({
    resolver: zodResolver(profileSchema),
    defaultValues: {
      name: business?.name ?? "",
      address: business?.address ?? "",
      city: business?.city ?? "",
      region: business?.region ?? "",
      postal_code: business?.postal_code ?? "",
      country: business?.country ?? "",
      phone: business?.phone ?? "",
      email: business?.email ?? "",
      website: business?.website ?? "",
      timezone: business?.timezone ?? "UTC",
    },
  });

  useEffect(() => {
    if (business) {
      reset({
        name: business.name,
        address: business.address ?? "",
        city: business.city ?? "",
        region: business.region ?? "",
        postal_code: business.postal_code ?? "",
        country: business.country ?? "",
        phone: business.phone ?? "",
        email: business.email ?? "",
        website: business.website ?? "",
        timezone: business.timezone,
      });
    }
  }, [business, reset]);

  const commitProfileUpdate = async (values: ProfileFormValues) => {
    setSaveMessage(null);
    setErrorMessage(null);
    try {
      await updateProfile.mutateAsync({
        name: values.name,
        address: values.address || null,
        city: values.city || null,
        region: values.region || null,
        postal_code: values.postal_code || null,
        country: values.country || null,
        phone: values.phone || null,
        email: values.email || null,
        website: values.website || null,
        timezone: values.timezone,
      });
      setSaveMessage("Business profile saved.");
    } catch (err) {
      setErrorMessage(extractErrorMessage(err));
    }
  };

  const onSubmitProfile = async (values: ProfileFormValues) => {
    if (business && values.timezone !== business.timezone) {
      setPendingTimezoneSubmit(values);
      return;
    }
    await commitProfileUpdate(values);
  };

  const handleLogoFileChange = async (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    setSaveMessage(null);
    setErrorMessage(null);
    const file = event.target.files?.[0];
    if (!file) return;

    if (!ALLOWED_LOGO_TYPES.includes(file.type)) {
      setErrorMessage("Please select a PNG, JPEG, or WebP image.");
      return;
    }
    if (file.size > MAX_LOGO_BYTES) {
      setErrorMessage("Logo file is too large (maximum 2 MB).");
      return;
    }

    try {
      const buffer = await file.arrayBuffer();
      await uploadLogo.mutateAsync(Array.from(new Uint8Array(buffer)));
      setSaveMessage("Business logo updated.");
    } catch (err) {
      setErrorMessage(extractErrorMessage(err));
    } finally {
      if (fileInputRef.current) fileInputRef.current.value = "";
    }
  };

  const handleRemoveLogo = async () => {
    setSaveMessage(null);
    setErrorMessage(null);
    try {
      await removeLogo.mutateAsync();
      setSaveMessage("Business logo removed.");
    } catch (err) {
      setErrorMessage(extractErrorMessage(err));
    }
  };

  const handlePrefChange = async (
    patch: Parameters<typeof updateSettings.mutateAsync>[0],
    feedback?: string
  ) => {
    setSaveMessage(null);
    setErrorMessage(null);
    try {
      await updateSettings.mutateAsync(patch);
      if (feedback) setSaveMessage(feedback);
    } catch (err) {
      setErrorMessage(extractErrorMessage(err));
    }
  };

  const handleSaveRetention = async () => {
    const parsed = Number(retentionInput);
    if (!Number.isInteger(parsed) || parsed < 1 || parsed > 365) {
      setErrorMessage(
        "Automatic backup retention must be a whole number between 1 and 365."
      );
      return;
    }
    await handlePrefChange(
      { auto_backup_retention: parsed },
      `Automatic backup retention updated to ${parsed}.`
    );
    setRetentionDraft(null);
  };

  const handleCopyText = async (text: string, label: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setSaveMessage(`${label} copied to clipboard.`);
    } catch {
      setErrorMessage(`Unable to copy ${label.toLowerCase()} to clipboard.`);
    }
  };

  const buildSanitizedDiagnostics = () => {
    return [
      `App: ${appInfo?.name ?? "My Schedule App"} v${appInfo?.version ?? "0.1.0"}`,
      `Storage Mode: ${storageInfo?.mode ?? "unknown"}`,
      `Fallback Reason: ${storageInfo?.fallback_reason ?? "none"}`,
      `Business Timezone: ${business?.timezone ?? "unknown"}`,
      `DB Integrity: ${dbHealth?.integrity_ok ? "ok" : "unknown"}`,
      `DB Journal Mode: ${dbHealth?.journal_mode ?? "unknown"}`,
      `DB Foreign Keys: ${dbHealth?.foreign_keys_enabled ? "on" : "unknown"}`,
      `DB Migrations Applied: ${dbHealth?.applied_migrations ?? 0} (latest v${dbHealth?.latest_migration_version ?? 0})`,
    ].join("\n");
  };

  return (
    <div className="mx-auto w-full max-w-4xl p-6 sm:p-8 space-y-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Manage business details, appearance, reminders, automatic backups, and
          local storage diagnostics.
        </p>
      </div>

      {saveMessage && (
        <div
          role="status"
          className="flex items-center gap-2 rounded-xl border border-emerald-500/40 bg-emerald-500/5 px-4 py-3 text-sm text-foreground"
        >
          <CheckCircle2 className="size-4 text-emerald-600 dark:text-emerald-400 shrink-0" />
          <span>{saveMessage}</span>
        </div>
      )}

      {errorMessage && (
        <div
          role="alert"
          className="rounded-xl border border-destructive/40 bg-card px-4 py-3 text-sm text-destructive"
        >
          {errorMessage}
        </div>
      )}

      {/* 1. Appearance */}
      <section
        aria-label="Appearance settings"
        className="rounded-xl border border-border bg-card p-5 space-y-5"
      >
        <div className="flex items-center gap-2">
          <Palette className="size-4 text-primary" aria-hidden="true" />
          <h2 className="text-base font-semibold">Appearance</h2>
        </div>

        <div className="space-y-2">
          <Label>Theme</Label>
          <div
            role="group"
            aria-label="Theme mode"
            className="grid grid-cols-3 gap-2 max-w-md"
          >
            {THEME_OPTIONS.map((opt) => {
              const Icon = opt.icon;
              const active = (appSettings?.theme_mode ?? "system") === opt.value;
              return (
                <button
                  key={opt.value}
                  type="button"
                  aria-pressed={active}
                  onClick={() =>
                    void handlePrefChange(
                      { theme_mode: opt.value },
                      `Theme set to ${opt.label}.`
                    )
                  }
                  className={`flex items-center justify-center gap-2 rounded-lg border px-3 py-2 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${
                    active
                      ? "border-primary bg-primary text-primary-foreground"
                      : "border-border bg-background text-foreground hover:bg-accent"
                  }`}
                >
                  <Icon className="size-4" aria-hidden="true" />
                  <span>{opt.label}</span>
                </button>
              );
            })}
          </div>
        </div>

        <div className="space-y-2">
          <Label>Accent Color</Label>
          <p className="text-xs text-muted-foreground">
            All presets are tuned for readable contrast in both Light and Dark
            themes.
          </p>
          <div
            role="group"
            aria-label="Accent color"
            className="flex flex-wrap gap-2"
          >
            {ACCENT_OPTIONS.map((opt) => {
              const active =
                (appSettings?.accent_color ?? "slate") === opt.value;
              return (
                <button
                  key={opt.value}
                  type="button"
                  aria-pressed={active}
                  onClick={() =>
                    void handlePrefChange(
                      { accent_color: opt.value },
                      `Accent color set to ${opt.label}.`
                    )
                  }
                  className={`inline-flex items-center gap-2 rounded-lg border px-3 py-1.5 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${
                    active
                      ? "border-primary bg-accent text-accent-foreground ring-1 ring-primary"
                      : "border-border bg-background text-foreground hover:bg-accent/50"
                  }`}
                >
                  <span
                    className={`size-3 rounded-full ${opt.swatchClass}`}
                    aria-hidden="true"
                  />
                  <span>{opt.label}</span>
                </button>
              );
            })}
          </div>
        </div>
      </section>

      {/* 2. Business Profile & Timezone */}
      <section
        aria-label="Business profile settings"
        className="rounded-xl border border-border bg-card p-5 space-y-5"
      >
        <div>
          <h2 className="text-base font-semibold">
            Business Profile &amp; Timezone
          </h2>
          <p className="text-xs text-muted-foreground mt-0.5">
            Stored appointment instants always remain in UTC. Changing the
            business timezone updates how calendar views, reports, and reminders
            interpret local dates and recalculates pending reminders.
          </p>
        </div>

        {/* Logo Controls */}
        <div className="flex flex-wrap items-center gap-4 pb-4 border-b border-border">
          {logo ? (
            <img
              src={logo.data_url}
              alt="Business logo"
              className="size-14 rounded-lg border border-border object-cover"
            />
          ) : (
            <div className="size-14 rounded-lg border border-dashed border-border bg-muted/40 flex items-center justify-center text-xs text-muted-foreground">
              No logo
            </div>
          )}

          <div className="flex flex-wrap items-center gap-2">
            <input
              ref={fileInputRef}
              type="file"
              accept="image/png,image/jpeg,image/webp"
              onChange={handleLogoFileChange}
              className="hidden"
            />
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={uploadLogo.isPending}
              onClick={() => fileInputRef.current?.click()}
            >
              <ImagePlus className="size-4" />
              {logo ? "Replace Logo" : "Upload Logo"}
            </Button>
            {logo && (
              <Button
                type="button"
                variant="ghost"
                size="sm"
                disabled={removeLogo.isPending}
                onClick={handleRemoveLogo}
              >
                <Trash2 className="size-4" />
                Remove
              </Button>
            )}
          </div>
        </div>

        <form onSubmit={handleSubmit(onSubmitProfile)} className="space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <Label htmlFor="settings-business-name">Business Name *</Label>
              <Input
                id="settings-business-name"
                {...register("name")}
                aria-invalid={Boolean(errors.name)}
              />
              {errors.name && (
                <p className="text-xs text-destructive">
                  {errors.name.message}
                </p>
              )}
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="settings-timezone">
                Business Timezone (IANA) *
              </Label>
              <select
                id="settings-timezone"
                {...register("timezone")}
                className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                {timezones.map((tz) => (
                  <option key={tz} value={tz}>
                    {tz}
                  </option>
                ))}
              </select>
              {errors.timezone && (
                <p className="text-xs text-destructive">
                  {errors.timezone.message}
                </p>
              )}
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="settings-phone">Phone</Label>
              <Input id="settings-phone" {...register("phone")} />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="settings-email">Email</Label>
              <Input id="settings-email" type="email" {...register("email")} />
              {errors.email && (
                <p className="text-xs text-destructive">
                  {errors.email.message}
                </p>
              )}
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="settings-address">Address</Label>
              <Input id="settings-address" {...register("address")} />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="settings-city">City</Label>
              <Input id="settings-city" {...register("city")} />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="settings-region">State / Region</Label>
              <Input id="settings-region" {...register("region")} />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="settings-postal">Postal Code</Label>
              <Input id="settings-postal" {...register("postal_code")} />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="settings-country">Country</Label>
              <Input id="settings-country" {...register("country")} />
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="settings-website">Website</Label>
              <Input id="settings-website" {...register("website")} />
            </div>
          </div>

          <div className="flex justify-end">
            <Button
              type="submit"
              disabled={!isDirty || updateProfile.isPending}
            >
              {updateProfile.isPending ? "Saving..." : "Save Business Details"}
            </Button>
          </div>
        </form>
      </section>

      {/* 3. Reminders & Notifications */}
      <section
        aria-label="Reminder preferences"
        className="rounded-xl border border-border bg-card p-5 space-y-4"
      >
        <div className="flex items-center gap-2">
          <Bell className="size-4 text-primary" aria-hidden="true" />
          <h2 className="text-base font-semibold">
            Reminders &amp; Notifications
          </h2>
        </div>

        <div className="space-y-3 text-sm">
          <label className="flex items-center justify-between gap-3">
            <span>Enable appointment reminders</span>
            <input
              type="checkbox"
              checked={appSettings?.reminders_enabled ?? true}
              onChange={(e) =>
                void handlePrefChange({
                  reminders_enabled: e.target.checked,
                })
              }
              className="size-4 accent-primary"
            />
          </label>

          <label className="flex items-center justify-between gap-3">
            <span>One day before appointment (same business-local time)</span>
            <input
              type="checkbox"
              disabled={!(appSettings?.reminders_enabled ?? true)}
              checked={appSettings?.reminder_one_day_before ?? true}
              onChange={(e) =>
                void handlePrefChange({
                  reminder_one_day_before: e.target.checked,
                })
              }
              className="size-4 accent-primary"
            />
          </label>

          <label className="flex items-center justify-between gap-3">
            <span>One hour before appointment</span>
            <input
              type="checkbox"
              disabled={!(appSettings?.reminders_enabled ?? true)}
              checked={appSettings?.reminder_one_hour_before ?? true}
              onChange={(e) =>
                void handlePrefChange({
                  reminder_one_hour_before: e.target.checked,
                })
              }
              className="size-4 accent-primary"
            />
          </label>

          <label className="flex items-center justify-between gap-3">
            <span>
              Appointment-day summary (once on first app launch each local day)
            </span>
            <input
              type="checkbox"
              disabled={!(appSettings?.reminders_enabled ?? true)}
              checked={appSettings?.reminder_day_of_launch ?? true}
              onChange={(e) =>
                void handlePrefChange({
                  reminder_day_of_launch: e.target.checked,
                })
              }
              className="size-4 accent-primary"
            />
          </label>

          <label className="flex items-center justify-between gap-3">
            <span>
              Send desktop OS notifications (in-app alerts always remain active)
            </span>
            <input
              type="checkbox"
              disabled={!(appSettings?.reminders_enabled ?? true)}
              checked={appSettings?.os_notifications_enabled ?? true}
              onChange={(e) =>
                void handlePrefChange({
                  os_notifications_enabled: e.target.checked,
                })
              }
              className="size-4 accent-primary"
            />
          </label>
        </div>

        <p className="text-xs text-muted-foreground rounded-lg bg-muted/40 p-3">
          Desktop notifications depend on your operating system&apos;s
          notification permissions. Reminders cannot be delivered while My
          Schedule App is completely closed unless a separate OS scheduled task
          is configured; due reminders are caught up automatically on next
          launch.
        </p>
      </section>

      {/* 4. Backups & Storage Diagnostics */}
      <section
        aria-label="Backups and storage diagnostics"
        className="rounded-xl border border-border bg-card p-5 space-y-4"
      >
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <DatabaseBackup className="size-4 text-primary" aria-hidden="true" />
            <h2 className="text-base font-semibold">
              Automatic Backups &amp; Storage
            </h2>
          </div>
          <Link
            to="/backups"
            className="text-xs font-medium text-primary hover:underline"
          >
            Open Backups &amp; Exports
          </Link>
        </div>

        <label className="flex items-center justify-between gap-3 text-sm">
          <span>Enable automatic daily startup backups</span>
          <input
            type="checkbox"
            checked={appSettings?.auto_backup_enabled ?? true}
            onChange={(e) =>
              void handlePrefChange({
                auto_backup_enabled: e.target.checked,
              })
            }
            className="size-4 accent-primary"
          />
        </label>

        <div className="flex flex-wrap items-end gap-2">
          <div className="space-y-1">
            <Label htmlFor="auto-backup-retention">
              Automatic backup retention (1–365, default 30)
            </Label>
            <Input
              id="auto-backup-retention"
              type="number"
              min={1}
              max={365}
              value={retentionInput}
              onChange={(e) => setRetentionDraft(e.target.value)}
              className="w-36"
            />
          </div>
          <Button
            type="button"
            variant="outline"
            disabled={updateSettings.isPending}
            onClick={handleSaveRetention}
          >
            Save Retention
          </Button>
        </div>

        <div className="rounded-lg border border-border bg-muted/30 p-3.5 space-y-2 text-xs">
          <div className="flex items-center justify-between gap-2">
            <span className="inline-flex items-center gap-1.5 font-medium text-foreground">
              <HardDrive className="size-3.5" aria-hidden="true" />
              Storage Mode:{" "}
              {storageInfo?.mode === "portable"
                ? "Portable (adjacent data folder beside executable)"
                : storageInfo?.mode === "fallback"
                  ? "Per-user fallback directory"
                  : "Checking..."}
            </span>

            {storageInfo && (
              <div className="flex items-center gap-1">
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-7 px-2 text-xs"
                  onClick={() => setRevealStoragePath((v) => !v)}
                >
                  {revealStoragePath ? (
                    <>
                      <EyeOff className="size-3.5" />
                      Hide Path
                    </>
                  ) : (
                    <>
                      <Eye className="size-3.5" />
                      Reveal Path
                    </>
                  )}
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-7 px-2 text-xs"
                  onClick={() =>
                    void handleCopyText(storageInfo.data_dir, "Storage path")
                  }
                >
                  <Copy className="size-3.5" />
                  Copy
                </Button>
              </div>
            )}
          </div>

          {storageInfo?.fallback_reason && (
            <p className="text-muted-foreground">
              Fallback reason: {storageInfo.fallback_reason}
            </p>
          )}

          {revealStoragePath && storageInfo && (
            <p className="font-mono text-[11px] text-foreground break-all bg-background rounded p-2 border border-border">
              {storageInfo.data_dir}
            </p>
          )}
        </div>
      </section>

      {/* 5. Updates */}
      <UpdatesSection />

      {/* 6. About & Support Diagnostics */}
      <section
        aria-label="About and diagnostics"
        className="rounded-xl border border-border bg-card p-5 space-y-3"
      >
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <Info className="size-4 text-primary" aria-hidden="true" />
            <h2 className="text-base font-semibold">
              About &amp; Support Diagnostics
            </h2>
          </div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() =>
              void handleCopyText(
                buildSanitizedDiagnostics(),
                "Support diagnostics"
              )
            }
          >
            <Copy className="size-3.5" />
            Copy Sanitized Diagnostics
          </Button>
        </div>

        <dl className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-2 text-xs">
          <div className="flex justify-between border-b border-border py-1.5">
            <dt className="text-muted-foreground">Application</dt>
            <dd className="font-medium">
              {appInfo?.name ?? "My Schedule App"} v{appInfo?.version ?? "0.1.0"}
            </dd>
          </div>
          <div className="flex justify-between border-b border-border py-1.5">
            <dt className="text-muted-foreground">Database Integrity</dt>
            <dd className="font-medium">
              {dbHealth?.integrity_ok ? "Verified OK" : "Checking..."}
            </dd>
          </div>
          <div className="flex justify-between border-b border-border py-1.5">
            <dt className="text-muted-foreground">SQLite Journal / FK</dt>
            <dd className="font-mono">
              {dbHealth
                ? `${dbHealth.journal_mode.toUpperCase()} · FK ${dbHealth.foreign_keys_enabled ? "ON" : "OFF"}`
                : "..."}
            </dd>
          </div>
          <div className="flex justify-between border-b border-border py-1.5">
            <dt className="text-muted-foreground">Schema Version</dt>
            <dd className="font-mono">
              {dbHealth
                ? `v${dbHealth.latest_migration_version ?? 0} (${dbHealth.applied_migrations} migrations)`
                : "..."}
            </dd>
          </div>
        </dl>
      </section>

      {/* Timezone Change Confirmation Dialog */}
      {pendingTimezoneSubmit && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="tz-confirm-title"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
        >
          <div className="w-full max-w-md rounded-xl border border-border bg-card p-6 shadow-lg space-y-4">
            <div className="flex items-start gap-3">
              <AlertTriangle className="size-5 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
              <div className="space-y-1.5">
                <h2 id="tz-confirm-title" className="text-base font-semibold">
                  Confirm Business Timezone Change
                </h2>
                <p className="text-sm text-muted-foreground">
                  Change business timezone from{" "}
                  <strong className="text-foreground">
                    {business?.timezone}
                  </strong>{" "}
                  to{" "}
                  <strong className="text-foreground">
                    {pendingTimezoneSubmit.timezone}
                  </strong>
                  ?
                </p>
                <p className="text-xs text-muted-foreground">
                  Existing appointments remain stored at their exact UTC
                  instants and will not be rewritten. Calendar views, reports,
                  and pending reminders will be re-interpreted in the new
                  timezone.
                </p>
              </div>
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => setPendingTimezoneSubmit(null)}
              >
                Cancel
              </Button>
              <Button
                type="button"
                disabled={updateProfile.isPending}
                onClick={async () => {
                  const vals = pendingTimezoneSubmit;
                  setPendingTimezoneSubmit(null);
                  await commitProfileUpdate(vals);
                }}
              >
                Confirm Timezone Change
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
