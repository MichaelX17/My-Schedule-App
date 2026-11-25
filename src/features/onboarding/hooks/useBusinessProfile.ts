import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createBusinessProfile,
  getBusinessLogo,
  getOnboardingStatus,
  removeBusinessLogo,
  updateBusinessProfile,
  uploadBusinessLogo,
  type CreateBusinessProfileInput,
  type UpdateBusinessProfileInput,
} from "@/services/business";
import { isTauriRuntime } from "@/services/tauri";

export const BUSINESS_QUERY_KEYS = {
  onboarding: ["business", "onboarding-status"] as const,
  logo: ["business", "logo"] as const,
};

export function useOnboardingStatus() {
  return useQuery({
    queryKey: BUSINESS_QUERY_KEYS.onboarding,
    queryFn: getOnboardingStatus,
    enabled: isTauriRuntime(),
    staleTime: 60_000,
  });
}

export function useBusinessLogo(enabled: boolean) {
  return useQuery({
    queryKey: BUSINESS_QUERY_KEYS.logo,
    queryFn: getBusinessLogo,
    enabled: isTauriRuntime() && enabled,
    staleTime: 60_000,
  });
}

export function useCreateBusinessProfile() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateBusinessProfileInput) =>
      createBusinessProfile(input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["business"] });
    },
  });
}

export function useUpdateBusinessProfile() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateBusinessProfileInput) =>
      updateBusinessProfile(input),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["business"] }),
        queryClient.invalidateQueries({ queryKey: ["appointments"] }),
        queryClient.invalidateQueries({ queryKey: ["calendar"] }),
        queryClient.invalidateQueries({ queryKey: ["reminders"] }),
        queryClient.invalidateQueries({ queryKey: ["reports"] }),
      ]);
    },
  });
}

export function useUploadBusinessLogo() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (bytes: number[]) => uploadBusinessLogo(bytes),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["business"] });
    },
  });
}

export function useRemoveBusinessLogo() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => removeBusinessLogo(),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["business"] });
    },
  });
}
