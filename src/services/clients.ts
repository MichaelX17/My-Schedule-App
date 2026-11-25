import { invokeCommand } from "@/services/tauri";

export interface Client {
  id: string;
  full_name: string;
  phone: string | null;
  email: string | null;
  address: string | null;
  notes: string | null;
  is_archived: boolean;
  archived_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateClientInput {
  full_name: string;
  phone?: string | null;
  email?: string | null;
  address?: string | null;
  notes?: string | null;
}

export interface UpdateClientInput {
  id: string;
  full_name: string;
  phone?: string | null;
  email?: string | null;
  address?: string | null;
  notes?: string | null;
}

export type ClientArchiveFilter = "active" | "archived" | "all";
export type ClientSortBy =
  | "name_asc"
  | "name_desc"
  | "recently_updated"
  | "newest";

export interface ListClientsParams {
  search?: string | null;
  filter?: ClientArchiveFilter;
  sort_by?: ClientSortBy;
  limit?: number;
  offset?: number;
}

export interface ClientListResult {
  items: Client[];
  total: number;
  active_count: number;
  archived_count: number;
}

export async function listClients(
  params?: ListClientsParams
): Promise<ClientListResult> {
  return invokeCommand<ClientListResult>("list_clients", { params });
}

export async function getClient(id: string): Promise<Client> {
  return invokeCommand<Client>("get_client", { id });
}

export async function createClient(input: CreateClientInput): Promise<Client> {
  return invokeCommand<Client>("create_client", { input });
}

export async function updateClient(input: UpdateClientInput): Promise<Client> {
  return invokeCommand<Client>("update_client", { input });
}

export async function archiveClient(id: string): Promise<Client> {
  return invokeCommand<Client>("archive_client", { id });
}

export async function restoreClient(id: string): Promise<Client> {
  return invokeCommand<Client>("restore_client", { id });
}
