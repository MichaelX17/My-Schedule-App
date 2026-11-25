import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  activateService,
  archiveService,
  createService,
  deactivateService,
  listServices,
  restoreService,
  updateService,
  type CreateServiceInput,
  type ListServicesParams,
  type UpdateServiceInput,
} from "@/services/serviceCatalog";
import { isTauriRuntime } from "@/services/tauri";

export const SERVICE_QUERY_KEYS = {
  all: ["services"] as const,
  list: (params: ListServicesParams) => ["services", "list", params] as const,
};

export function useServicesList(params: ListServicesParams) {
  return useQuery({
    queryKey: SERVICE_QUERY_KEYS.list(params),
    queryFn: () => listServices(params),
    enabled: isTauriRuntime(),
    placeholderData: (previous) => previous,
  });
}

export function useCreateService() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateServiceInput) => createService(input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: SERVICE_QUERY_KEYS.all });
    },
  });
}

export function useUpdateService() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateServiceInput) => updateService(input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: SERVICE_QUERY_KEYS.all });
    },
  });
}

export function useActivateService() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => activateService(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: SERVICE_QUERY_KEYS.all });
    },
  });
}

export function useDeactivateService() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deactivateService(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: SERVICE_QUERY_KEYS.all });
    },
  });
}

export function useArchiveService() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => archiveService(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: SERVICE_QUERY_KEYS.all });
    },
  });
}

export function useRestoreService() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => restoreService(id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: SERVICE_QUERY_KEYS.all });
    },
  });
}
