import { invokeCommand, type BackendError } from "@/services/tauri";

export interface BusinessProfile {
  id: string;
  name: string;
  logo_filename: string | null;
  address: string | null;
  city: string | null;
  region: string | null;
  postal_code: string | null;
  country: string | null;
  phone: string | null;
  email: string | null;
  website: string | null;
  timezone: string;
  onboarding_completed: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateBusinessProfileInput {
  name: string;
  address?: string | null;
  city?: string | null;
  region?: string | null;
  postal_code?: string | null;
  country?: string | null;
  phone?: string | null;
  email?: string | null;
  website?: string | null;
  timezone: string;
  logo_bytes?: number[] | null;
}

export interface UpdateBusinessProfileInput {
  name: string;
  address?: string | null;
  city?: string | null;
  region?: string | null;
  postal_code?: string | null;
  country?: string | null;
  phone?: string | null;
  email?: string | null;
  website?: string | null;
  timezone: string;
}

export interface OnboardingStatus {
  completed: boolean;
  business: BusinessProfile | null;
  detected_timezone: string;
}

export interface BusinessLogoData {
  filename: string;
  mime_type: string;
  data_url: string;
}

export function extractErrorMessage(err: unknown): string {
  if (typeof err === "object" && err !== null && "message" in err) {
    const backendErr = err as Partial<BackendError>;
    if (typeof backendErr.message === "string" && backendErr.message.length > 0) {
      return backendErr.message;
    }
  }
  if (err instanceof Error && err.message.length > 0) {
    return err.message;
  }
  if (typeof err === "string" && err.length > 0) {
    return err;
  }
  return "An unexpected error occurred. Please try again.";
}

export async function getOnboardingStatus(): Promise<OnboardingStatus> {
  return invokeCommand<OnboardingStatus>("get_onboarding_status");
}

export async function getBusinessProfile(): Promise<BusinessProfile | null> {
  return invokeCommand<BusinessProfile | null>("get_business_profile");
}

export async function createBusinessProfile(
  input: CreateBusinessProfileInput
): Promise<BusinessProfile> {
  return invokeCommand<BusinessProfile>("create_business_profile", { input });
}

export async function updateBusinessProfile(
  input: UpdateBusinessProfileInput
): Promise<BusinessProfile> {
  return invokeCommand<BusinessProfile>("update_business_profile", { input });
}

export async function uploadBusinessLogo(
  bytes: number[]
): Promise<BusinessLogoData> {
  return invokeCommand<BusinessLogoData>("upload_business_logo", { bytes });
}

export async function getBusinessLogo(): Promise<BusinessLogoData | null> {
  return invokeCommand<BusinessLogoData | null>("get_business_logo");
}

export async function removeBusinessLogo(): Promise<BusinessProfile> {
  return invokeCommand<BusinessProfile>("remove_business_logo");
}
