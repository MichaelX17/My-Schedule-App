import { invokeCommand } from "@/services/tauri";

export interface Service {
  id: string;
  name: string;
  description: string | null;
  default_duration_minutes: number;
  is_active: boolean;
  is_archived: boolean;
  archived_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateServiceInput {
  name: string;
  description?: string | null;
  default_duration_minutes: number;
}

export interface UpdateServiceInput {
  id: string;
  name: string;
  description?: string | null;
  default_duration_minutes: number;
}

export type ServiceStatusFilter = "active" | "inactive" | "archived" | "all";
export type ServiceSortBy =
  | "name_asc"
  | "name_desc"
  | "duration_asc"
  | "duration_desc"
  | "recently_updated";

export interface ListServicesParams {
  search?: string | null;
  filter?: ServiceStatusFilter;
  sort_by?: ServiceSortBy;
}

export interface ServiceListResult {
  items: Service[];
  total: number;
  active_count: number;
  inactive_count: number;
  archived_count: number;
}

export async function listServices(
  params?: ListServicesParams
): Promise<ServiceListResult> {
  return invokeCommand<ServiceListResult>("list_services", { params });
}

export async function getService(id: string): Promise<Service> {
  return invokeCommand<Service>("get_service", { id });
}

export async function createService(input: CreateServiceInput): Promise<Service> {
  return invokeCommand<Service>("create_service", { input });
}

export async function updateService(input: UpdateServiceInput): Promise<Service> {
  return invokeCommand<Service>("update_service", { input });
}

export async function activateService(id: string): Promise<Service> {
  return invokeCommand<Service>("activate_service", { id });
}

export async function deactivateService(id: string): Promise<Service> {
  return invokeCommand<Service>("deactivate_service", { id });
}

export async function archiveService(id: string): Promise<Service> {
  return invokeCommand<Service>("archive_service", { id });
}

export async function restoreService(id: string): Promise<Service> {
  return invokeCommand<Service>("restore_service", { id });
}

export async function deleteService(id: string): Promise<void> {
  return invokeCommand<void>("delete_service", { id });
}
