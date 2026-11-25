import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  archiveClient,
  createClient,
  getClient,
  listClients,
  restoreClient,
  updateClient,
  type CreateClientInput,
  type ListClientsParams,
  type UpdateClientInput,
} from "@/services/clients";
import { isTauriRuntime } from "@/services/tauri";

export const CLIENT_QUERY_KEYS = {
  all: ["clients"] as const,
  list: (params: ListClientsParams) => ["clients", "list", params] as const,
  detail: (id: string) => ["clients", "detail", id] as const,
};

export function useClientsList(params: ListClientsParams) {
  return useQuery({
    queryKey: CLIENT_QUERY_KEYS.list(params),
    queryFn: () => listClients(params),
    enabled: isTauriRuntime(),
    placeholderData: (previous) => previous,
  });
}

export function useClientDetail(id: string | undefined) {
  return useQuery({
    queryKey: CLIENT_QUERY_KEYS.detail(id ?? ""),
    queryFn: () => getClient(id!),
    enabled: isTauriRuntime() && Boolean(id),
  });
}

export function useCreateClient() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateClientInput) => createClient(input),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: CLIENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: ["reports"] }),
      ]);
    },
  });
}

export function useUpdateClient() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateClientInput) => updateClient(input),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: CLIENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: ["appointments"] }),
        queryClient.invalidateQueries({ queryKey: ["calendar"] }),
        queryClient.invalidateQueries({ queryKey: ["reports"] }),
      ]);
    },
  });
}

export function useArchiveClient() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => archiveClient(id),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: CLIENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: ["reports"] }),
      ]);
    },
  });
}

export function useRestoreClient() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => restoreClient(id),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: CLIENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: ["reports"] }),
      ]);
    },
  });
}
