import { useMemo, useRef, useState } from "react";
import { useForm, useWatch } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import {
  Building2,
  Check,
  Clock,
  ImagePlus,
  MapPin,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  detectBrowserTimezone,
  getAvailableTimezones,
} from "@/lib/timezones";
import { extractErrorMessage } from "@/services/business";
import { useCreateBusinessProfile } from "./hooks/useBusinessProfile";

const MAX_LOGO_BYTES = 2 * 1024 * 1024;
const ALLOWED_LOGO_TYPES = ["image/png", "image/jpeg", "image/webp"];

const onboardingSchema = z.object({
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
        const trimmed = val.trim();
        return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmed);
      },
      { message: "Please enter a valid email address or leave it blank." }
    ),
  website: z.string().max(255).optional(),
  timezone: z.string().trim().min(1, "Business timezone is required."),
});

type OnboardingFormValues = z.infer<typeof onboardingSchema>;

interface OnboardingWizardProps {
  detectedTimezone: string;
}

const STEPS = [
  { id: 0, title: "Business Identity", icon: Building2 },
  { id: 1, title: "Location & Contact", icon: MapPin },
  { id: 2, title: "Timezone & Finish", icon: Clock },
] as const;

export function OnboardingWizard({ detectedTimezone }: OnboardingWizardProps) {
  const defaultTz = detectedTimezone || detectBrowserTimezone();
  const [step, setStep] = useState(0);
  const [logoBytes, setLogoBytes] = useState<number[] | null>(null);
  const [logoPreviewUrl, setLogoPreviewUrl] = useState<string | null>(null);
  const [logoError, setLogoError] = useState<string | null>(null);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [tzFilter, setTzFilter] = useState("");
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const createProfile = useCreateBusinessProfile();

  const {
    control,
    register,
    handleSubmit,
    trigger,
    setValue,
    formState: { errors },
  } = useForm<OnboardingFormValues>({
    resolver: zodResolver(onboardingSchema),
    defaultValues: {
      name: "",
      address: "",
      city: "",
      region: "",
      postal_code: "",
      country: "",
      phone: "",
      email: "",
      website: "",
      timezone: defaultTz,
    },
  });

  const [watchedName, watchedTimezone] = useWatch({
    control,
    name: ["name", "timezone"],
  });

  const allTimezones = useMemo(
    () => getAvailableTimezones(defaultTz),
    [defaultTz]
  );

  const filteredTimezones = useMemo(() => {
    const query = tzFilter.trim().toLowerCase();
    if (!query) return allTimezones;
    return allTimezones.filter((tz) => tz.toLowerCase().includes(query));
  }, [allTimezones, tzFilter]);

  const handleLogoFileChange = async (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    setLogoError(null);
    const file = event.target.files?.[0];
    if (!file) return;

    if (!ALLOWED_LOGO_TYPES.includes(file.type)) {
      setLogoError("Please choose a PNG, JPEG, or WebP image.");
      event.target.value = "";
      return;
    }

    if (file.size === 0 || file.size > MAX_LOGO_BYTES) {
      setLogoError("Logo image must be smaller than 2 MB.");
      event.target.value = "";
      return;
    }

    const buffer = await file.arrayBuffer();
    const bytes = Array.from(new Uint8Array(buffer));
    setLogoBytes(bytes);

    if (logoPreviewUrl) {
      URL.revokeObjectURL(logoPreviewUrl);
    }
    setLogoPreviewUrl(URL.createObjectURL(file));
  };

  const handleClearLogo = () => {
    if (logoPreviewUrl) {
      URL.revokeObjectURL(logoPreviewUrl);
    }
    setLogoPreviewUrl(null);
    setLogoBytes(null);
    setLogoError(null);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  };

  const handleNextStep = async () => {
    setSubmitError(null);
    if (step === 0) {
      const valid = await trigger("name");
      if (!valid) return;
      setStep(1);
    } else if (step === 1) {
      const valid = await trigger([
        "address",
        "city",
        "region",
        "postal_code",
        "country",
        "phone",
        "email",
        "website",
      ]);
      if (!valid) return;
      setStep(2);
    }
  };

  const handleSkipContactStep = () => {
    setSubmitError(null);
    setValue("address", "");
    setValue("city", "");
    setValue("region", "");
    setValue("postal_code", "");
    setValue("country", "");
    setValue("phone", "");
    setValue("email", "");
    setValue("website", "");
    setStep(2);
  };

  const onSubmit = async (values: OnboardingFormValues) => {
    setSubmitError(null);
    try {
      await createProfile.mutateAsync({
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
        logo_bytes: logoBytes,
      });
    } catch (err) {
      setSubmitError(extractErrorMessage(err));
    }
  };

  return (
    <div className="min-h-screen bg-background text-foreground flex items-center justify-center p-4 sm:p-8">
      <div className="w-full max-w-2xl rounded-xl border border-border bg-card text-card-foreground shadow-sm overflow-hidden">
        <div className="border-b border-border px-6 py-5 sm:px-8">
          <p className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
            First-Run Setup
          </p>
          <h1 className="mt-1 text-2xl font-semibold tracking-tight">
            Welcome to My Schedule App
          </h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Set up your local business profile. Only your business name and
            timezone are required — everything else can be skipped or updated
            later in Settings.
          </p>

          <nav aria-label="Setup steps" className="mt-5 flex items-center gap-2">
            {STEPS.map((s, idx) => {
              const Icon = s.icon;
              const isActive = idx === step;
              const isDone = idx < step;
              return (
                <div
                  key={s.id}
                  className={`flex items-center gap-2 rounded-lg px-3 py-1.5 text-xs font-medium transition-colors ${
                    isActive
                      ? "bg-primary text-primary-foreground"
                      : isDone
                        ? "bg-secondary text-secondary-foreground"
                        : "text-muted-foreground"
                  }`}
                >
                  {isDone ? (
                    <Check className="size-3.5" />
                  ) : (
                    <Icon className="size-3.5" />
                  )}
                  <span className="hidden sm:inline">{s.title}</span>
                </div>
              );
            })}
          </nav>
        </div>

        <form onSubmit={handleSubmit(onSubmit)} className="p-6 sm:p-8 space-y-6">
          {step === 0 && (
            <div className="space-y-5">
              <div className="space-y-2">
                <Label htmlFor="business-name">
                  Business Name <span className="text-destructive">*</span>
                </Label>
                <Input
                  id="business-name"
                  placeholder="e.g. Studio Norte"
                  autoFocus
                  aria-invalid={Boolean(errors.name)}
                  {...register("name")}
                />
                {errors.name && (
                  <p className="text-xs text-destructive" role="alert">
                    {errors.name.message}
                  </p>
                )}
              </div>

              <div className="space-y-2">
                <Label htmlFor="business-logo">
                  Business Logo{" "}
                  <span className="text-xs font-normal text-muted-foreground">
                    (Optional)
                  </span>
                </Label>
                <div className="flex flex-col sm:flex-row items-start sm:items-center gap-4 rounded-lg border border-dashed border-border p-4 bg-muted/30">
                  {logoPreviewUrl ? (
                    <img
                      src={logoPreviewUrl}
                      alt="Selected business logo preview"
                      className="size-16 rounded-lg object-cover border border-border bg-background"
                    />
                  ) : (
                    <div className="size-16 rounded-lg border border-border bg-background flex items-center justify-center text-muted-foreground">
                      <ImagePlus className="size-6" />
                    </div>
                  )}

                  <div className="flex-1 space-y-2">
                    <p className="text-xs text-muted-foreground">
                      PNG, JPEG, or WebP up to 2 MB. Copied safely into local
                      application storage.
                    </p>
                    <div className="flex flex-wrap items-center gap-2">
                      <input
                        ref={fileInputRef}
                        id="business-logo"
                        type="file"
                        accept="image/png,image/jpeg,image/webp"
                        onChange={handleLogoFileChange}
                        className="sr-only"
                      />
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => fileInputRef.current?.click()}
                      >
                        {logoPreviewUrl ? "Change Logo" : "Choose Image"}
                      </Button>
                      {logoPreviewUrl && (
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={handleClearLogo}
                        >
                          <Trash2 className="size-3.5" />
                          Remove
                        </Button>
                      )}
                    </div>
                    {logoError && (
                      <p className="text-xs text-destructive" role="alert">
                        {logoError}
                      </p>
                    )}
                  </div>
                </div>
              </div>
            </div>
          )}

          {step === 1 && (
            <div className="space-y-4">
              <div className="flex items-center justify-between gap-2">
                <div>
                  <h2 className="text-base font-medium">
                    Location &amp; Contact Details
                  </h2>
                  <p className="text-xs text-muted-foreground">
                    All fields on this step are optional.
                  </p>
                </div>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={handleSkipContactStep}
                >
                  Skip for now
                </Button>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="sm:col-span-2 space-y-1.5">
                  <Label htmlFor="address">Street Address</Label>
                  <Input
                    id="address"
                    placeholder="123 Main Street"
                    {...register("address")}
                  />
                </div>

                <div className="space-y-1.5">
                  <Label htmlFor="city">City</Label>
                  <Input id="city" placeholder="Madrid" {...register("city")} />
                </div>

                <div className="space-y-1.5">
                  <Label htmlFor="region">State / Region</Label>
                  <Input
                    id="region"
                    placeholder="Community of Madrid"
                    {...register("region")}
                  />
                </div>

                <div className="space-y-1.5">
                  <Label htmlFor="postal_code">Postal Code</Label>
                  <Input
                    id="postal_code"
                    placeholder="28001"
                    {...register("postal_code")}
                  />
                </div>

                <div className="space-y-1.5">
                  <Label htmlFor="country">Country</Label>
                  <Input
                    id="country"
                    placeholder="Spain"
                    {...register("country")}
                  />
                </div>

                <div className="space-y-1.5">
                  <Label htmlFor="phone">Phone</Label>
                  <Input
                    id="phone"
                    placeholder="+34 600 000 000"
                    {...register("phone")}
                  />
                </div>

                <div className="space-y-1.5">
                  <Label htmlFor="email">Email</Label>
                  <Input
                    id="email"
                    type="email"
                    placeholder="contact@example.com"
                    aria-invalid={Boolean(errors.email)}
                    {...register("email")}
                  />
                  {errors.email && (
                    <p className="text-xs text-destructive" role="alert">
                      {errors.email.message}
                    </p>
                  )}
                </div>

                <div className="sm:col-span-2 space-y-1.5">
                  <Label htmlFor="website">Website</Label>
                  <Input
                    id="website"
                    placeholder="https://example.com"
                    {...register("website")}
                  />
                </div>
              </div>
            </div>
          )}

          {step === 2 && (
            <div className="space-y-5">
              <div className="space-y-2">
                <Label htmlFor="timezone-filter">
                  Business Timezone <span className="text-destructive">*</span>
                </Label>
                <p className="text-xs text-muted-foreground">
                  Detected from your computer:{" "}
                  <span className="font-mono font-medium text-foreground">
                    {defaultTz}
                  </span>
                  . All calendar days, reminders, and reports use this IANA
                  timezone.
                </p>

                <Input
                  id="timezone-filter"
                  placeholder="Filter timezones (e.g. Madrid, New_York, UTC)..."
                  value={tzFilter}
                  onChange={(e) => setTzFilter(e.target.value)}
                />

                <select
                  id="timezone"
                  aria-label="Business Timezone"
                  className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm text-foreground shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  {...register("timezone")}
                >
                  {filteredTimezones.map((tz) => (
                    <option key={tz} value={tz}>
                      {tz}
                    </option>
                  ))}
                </select>
                {errors.timezone && (
                  <p className="text-xs text-destructive" role="alert">
                    {errors.timezone.message}
                  </p>
                )}
              </div>

              <div className="rounded-lg border border-border bg-muted/30 p-4 space-y-1.5 text-sm">
                <p className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
                  Summary
                </p>
                <p>
                  <span className="text-muted-foreground">Business:</span>{" "}
                  <span className="font-medium">
                    {watchedName?.trim() || "—"}
                  </span>
                </p>
                <p>
                  <span className="text-muted-foreground">Timezone:</span>{" "}
                  <span className="font-mono text-xs">{watchedTimezone}</span>
                </p>
                <p>
                  <span className="text-muted-foreground">Logo:</span>{" "}
                  <span>{logoBytes ? "Included" : "Skipped"}</span>
                </p>
              </div>
            </div>
          )}

          {submitError && (
            <div
              role="alert"
              className="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive"
            >
              {submitError}
            </div>
          )}

          <div className="flex items-center justify-between pt-2 border-t border-border">
            <div>
              {step > 0 && (
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setStep((prev) => prev - 1)}
                  disabled={createProfile.isPending}
                >
                  Back
                </Button>
              )}
            </div>

            <div className="flex items-center gap-2">
              {step < STEPS.length - 1 ? (
                <Button type="button" onClick={handleNextStep}>
                  Continue
                </Button>
              ) : (
                <Button type="submit" disabled={createProfile.isPending}>
                  {createProfile.isPending ? "Saving..." : "Complete Setup"}
                </Button>
              )}
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}
