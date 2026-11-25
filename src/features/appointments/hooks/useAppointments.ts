import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  archiveSessionPlan,
  cancelAppointment,
  completeAppointment,
  createAppointment,
  createSessionPlan,
  createSessionPlanWithAppointments,
  deleteAppointment,
  deleteSessionPlan,
  getAppointment,
  listAppointments,
  listSessionPlans,
  rescheduleAppointment,
  updateAppointment,
  updateSessionPlan,
  type CreateAppointmentInput,
  type CreateSessionPlanInput,
  type CreateSessionPlanWithAppointmentsInput,
  type ListAppointmentsParams,
  type ListSessionPlansParams,
  type RescheduleAppointmentInput,
  type UpdateAppointmentInput,
  type UpdateSessionPlanInput,
} from "@/services/appointments";
import { CALENDAR_QUERY_KEYS } from "@/features/calendar/hooks/useCalendar";
import { REMINDER_QUERY_KEYS } from "@/features/reminders/hooks/useReminders";
import { REPORT_QUERY_KEYS } from "@/features/reports/hooks/useReports";
import { isTauriRuntime } from "@/services/tauri";

export const APPOINTMENT_QUERY_KEYS = {
  all: ["appointments"] as const,
  list: (params: ListAppointmentsParams) =>
    ["appointments", "list", params] as const,
  detail: (id: string) => ["appointments", "detail", id] as const,
};

export const SESSION_PLAN_QUERY_KEYS = {
  all: ["session-plans"] as const,
  list: (params: ListSessionPlansParams) =>
    ["session-plans", "list", params] as const,
};

export function useAppointmentsList(params: ListAppointmentsParams) {
  return useQuery({
    queryKey: APPOINTMENT_QUERY_KEYS.list(params),
    queryFn: () => listAppointments(params),
    enabled: isTauriRuntime(),
    placeholderData: (prev) => prev,
  });
}

export function useAppointmentDetail(id: string | undefined) {
  return useQuery({
    queryKey: APPOINTMENT_QUERY_KEYS.detail(id ?? ""),
    queryFn: () => getAppointment(id!),
    enabled: isTauriRuntime() && Boolean(id),
  });
}

export function useSessionPlansList(params: ListSessionPlansParams) {
  return useQuery({
    queryKey: SESSION_PLAN_QUERY_KEYS.list(params),
    queryFn: () => listSessionPlans(params),
    enabled: isTauriRuntime(),
    placeholderData: (prev) => prev,
  });
}

export function useCreateAppointment() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateAppointmentInput) => createAppointment(input),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: APPOINTMENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: CALENDAR_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REMINDER_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REPORT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: SESSION_PLAN_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: ["clients"] }),
      ]);
    },
  });
}

export function useUpdateAppointment() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateAppointmentInput) => updateAppointment(input),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: APPOINTMENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: CALENDAR_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REMINDER_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REPORT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: SESSION_PLAN_QUERY_KEYS.all }),
      ]);
    },
  });
}

export function useRescheduleAppointment() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: RescheduleAppointmentInput) =>
      rescheduleAppointment(input),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: APPOINTMENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: CALENDAR_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REMINDER_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REPORT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: SESSION_PLAN_QUERY_KEYS.all }),
      ]);
    },
  });
}

export function useCompleteAppointment() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => completeAppointment(id),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: APPOINTMENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: CALENDAR_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REMINDER_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REPORT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: SESSION_PLAN_QUERY_KEYS.all }),
      ]);
    },
  });
}

export function useCancelAppointment() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => cancelAppointment(id),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: APPOINTMENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: CALENDAR_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REMINDER_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REPORT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: SESSION_PLAN_QUERY_KEYS.all }),
      ]);
    },
  });
}

export function useDeleteAppointment() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteAppointment(id),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: APPOINTMENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: CALENDAR_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REMINDER_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REPORT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: SESSION_PLAN_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: ["clients"] }),
      ]);
    },
  });
}

export function useCreateSessionPlan() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateSessionPlanInput) => createSessionPlan(input),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: SESSION_PLAN_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REPORT_QUERY_KEYS.all }),
      ]);
    },
  });
}

export function useCreateSessionPlanWithAppointments() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: CreateSessionPlanWithAppointmentsInput) =>
      createSessionPlanWithAppointments(input),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: SESSION_PLAN_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: APPOINTMENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: CALENDAR_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REMINDER_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REPORT_QUERY_KEYS.all }),
      ]);
    },
  });
}

export function useUpdateSessionPlan() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateSessionPlanInput) => updateSessionPlan(input),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: SESSION_PLAN_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REPORT_QUERY_KEYS.all }),
      ]);
    },
  });
}

export function useArchiveSessionPlan() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => archiveSessionPlan(id),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: SESSION_PLAN_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REPORT_QUERY_KEYS.all }),
      ]);
    },
  });
}

export function useDeleteSessionPlan() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      id,
      deleteLinkedAppointments,
    }: {
      id: string;
      deleteLinkedAppointments: boolean;
    }) => deleteSessionPlan(id, deleteLinkedAppointments),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: SESSION_PLAN_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: APPOINTMENT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: CALENDAR_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REMINDER_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: REPORT_QUERY_KEYS.all }),
        queryClient.invalidateQueries({ queryKey: ["clients"] }),
      ]);
    },
  });
}
