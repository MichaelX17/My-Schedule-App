import type { CreateClientInput } from "@/services/clients";
import { invokeCommand } from "@/services/tauri";

export type AppointmentStatus = "scheduled" | "completed" | "cancelled";
export type SessionPlanStatus = "active" | "completed" | "archived";
export type DstDisambiguation = "earlier" | "later";

export interface AppointmentServiceItem {
  id: string;
  appointment_id: string;
  service_id: string;
  service_name_snapshot: string;
  service_description_snapshot: string | null;
  duration_minutes_snapshot: number;
  sort_order: number;
}

export interface AppointmentDetail {
  id: string;
  client_id: string;
  client_name: string;
  client_is_archived: boolean;
  session_plan_id: string | null;
  session_plan_title: string | null;
  start_utc: string;
  end_utc: string;
  start_local: string;
  end_local: string;
  timezone: string;
  status: AppointmentStatus;
  notes: string | null;
  services: AppointmentServiceItem[];
  created_at: string;
  updated_at: string;
}

export interface OverlapWarning {
  appointment_id: string;
  client_name: string;
  start_utc: string;
  end_utc: string;
  start_local: string;
  end_local: string;
}

export interface AppointmentMutationResult {
  appointment: AppointmentDetail;
  overlap_warnings: OverlapWarning[];
}

export interface CreateAppointmentInput {
  client_id?: string | null;
  inline_client?: CreateClientInput | null;
  session_plan_id?: string | null;
  start_local: string;
  end_local: string;
  dst_disambiguation?: DstDisambiguation | null;
  service_ids: string[];
  notes?: string | null;
}

export interface UpdateAppointmentInput {
  id: string;
  session_plan_id?: string | null;
  start_local: string;
  end_local: string;
  dst_disambiguation?: DstDisambiguation | null;
  service_ids: string[];
  notes?: string | null;
}

export interface RescheduleAppointmentInput {
  id: string;
  start_local: string;
  end_local: string;
  dst_disambiguation?: DstDisambiguation | null;
}

export interface CheckOverlapsInput {
  start_local: string;
  end_local: string;
  dst_disambiguation?: DstDisambiguation | null;
  exclude_appointment_id?: string | null;
}

export interface ListAppointmentsParams {
  search?: string | null;
  status?: AppointmentStatus | null;
  client_id?: string | null;
  session_plan_id?: string | null;
  from_utc?: string | null;
  to_utc?: string | null;
}

export interface SessionPlanSummary {
  id: string;
  client_id: string;
  client_name: string;
  title: string;
  description: string | null;
  planned_sessions: number;
  status: SessionPlanStatus;
  linked_total: number;
  linked_scheduled: number;
  linked_completed: number;
  linked_cancelled: number;
  created_at: string;
  updated_at: string;
}

export interface CreateSessionPlanInput {
  client_id: string;
  title: string;
  description?: string | null;
  planned_sessions: number;
}

export interface UpdateSessionPlanInput {
  id: string;
  title: string;
  description?: string | null;
  planned_sessions: number;
  status?: SessionPlanStatus | null;
}

export interface ListSessionPlansParams {
  client_id?: string | null;
  status?: SessionPlanStatus | null;
}

export async function listAppointments(
  params?: ListAppointmentsParams
): Promise<AppointmentDetail[]> {
  return invokeCommand<AppointmentDetail[]>("list_appointments", { params });
}

export async function getAppointment(id: string): Promise<AppointmentDetail> {
  return invokeCommand<AppointmentDetail>("get_appointment", { id });
}

export async function checkAppointmentOverlaps(
  input: CheckOverlapsInput
): Promise<OverlapWarning[]> {
  return invokeCommand<OverlapWarning[]>("check_appointment_overlaps", {
    input,
  });
}

export async function createAppointment(
  input: CreateAppointmentInput
): Promise<AppointmentMutationResult> {
  return invokeCommand<AppointmentMutationResult>("create_appointment", {
    input,
  });
}

export async function updateAppointment(
  input: UpdateAppointmentInput
): Promise<AppointmentMutationResult> {
  return invokeCommand<AppointmentMutationResult>("update_appointment", {
    input,
  });
}

export async function rescheduleAppointment(
  input: RescheduleAppointmentInput
): Promise<AppointmentMutationResult> {
  return invokeCommand<AppointmentMutationResult>("reschedule_appointment", {
    input,
  });
}

export async function completeAppointment(
  id: string
): Promise<AppointmentDetail> {
  return invokeCommand<AppointmentDetail>("complete_appointment", { id });
}

export async function cancelAppointment(
  id: string
): Promise<AppointmentDetail> {
  return invokeCommand<AppointmentDetail>("cancel_appointment", { id });
}

export async function listSessionPlans(
  params?: ListSessionPlansParams
): Promise<SessionPlanSummary[]> {
  return invokeCommand<SessionPlanSummary[]>("list_session_plans", { params });
}

export async function createSessionPlan(
  input: CreateSessionPlanInput
): Promise<SessionPlanSummary> {
  return invokeCommand<SessionPlanSummary>("create_session_plan", { input });
}

export async function updateSessionPlan(
  input: UpdateSessionPlanInput
): Promise<SessionPlanSummary> {
  return invokeCommand<SessionPlanSummary>("update_session_plan", { input });
}

export async function archiveSessionPlan(
  id: string
): Promise<SessionPlanSummary> {
  return invokeCommand<SessionPlanSummary>("archive_session_plan", { id });
}
