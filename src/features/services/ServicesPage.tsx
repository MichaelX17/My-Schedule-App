import { useEffect, useState } from "react";
import {
  Archive,
  ArchiveRestore,
  Briefcase,
  CheckCircle2,
  Clock,
  PauseCircle,
  Pencil,
  PlayCircle,
  Plus,
  Search,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { extractErrorMessage } from "@/services/business";
import type {
  Service,
  ServiceSortBy,
  ServiceStatusFilter,
} from "@/services/serviceCatalog";
import { ServiceFormDialog } from "./components/ServiceFormDialog";
import {
  useActivateService,
  useArchiveService,
  useCreateService,
  useDeactivateService,
  useRestoreService,
  useServicesList,
  useUpdateService,
} from "./hooks/useServices";
import { formatDuration, type normalizeServiceFormInput } from "./validation";

const FILTER_TABS: { value: ServiceStatusFilter; label: string }[] = [
  { value: "active", label: "Active" },
  { value: "inactive", label: "Inactive" },
  { value: "archived", label: "Archived" },
  { value: "all", label: "All" },
];

export function ServicesPage() {
  const [searchInput, setSearchInput] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [filter, setFilter] = useState<ServiceStatusFilter>("active");
  const [sortBy, setSortBy] = useState<ServiceSortBy>("name_asc");
  const [createOpen, setCreateOpen] = useState(false);
  const [editingService, setEditingService] = useState<Service | null>(null);
  const [confirmArchive, setConfirmArchive] = useState<Service | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedSearch(searchInput.trim());
    }, 200);
    return () => window.clearTimeout(timer);
  }, [searchInput]);

  const { data, isLoading, isError, error } = useServicesList({
    search: debouncedSearch || null,
    filter,
    sort_by: sortBy,
  });

  const createService = useCreateService();
  const updateService = useUpdateService();
  const activateService = useActivateService();
  const deactivateService = useDeactivateService();
  const archiveService = useArchiveService();
  const restoreService = useRestoreService();

  const items = data?.items ?? [];
  const activeCount = data?.active_count ?? 0;
  const inactiveCount = data?.inactive_count ?? 0;
  const archivedCount = data?.archived_count ?? 0;

  const handleCreateSubmit = async (
    values: ReturnType<typeof normalizeServiceFormInput>
  ) => {
    await createService.mutateAsync(values);
  };

  const handleEditSubmit = async (
    values: ReturnType<typeof normalizeServiceFormInput>
  ) => {
    if (!editingService) return;
    await updateService.mutateAsync({
      id: editingService.id,
      ...values,
    });
  };

  const handleToggleActive = async (service: Service) => {
    setActionError(null);
    try {
      if (service.is_active) {
        await deactivateService.mutateAsync(service.id);
      } else {
        await activateService.mutateAsync(service.id);
      }
    } catch (err) {
      setActionError(extractErrorMessage(err));
    }
  };

  const handleConfirmArchive = async () => {
    if (!confirmArchive) return;
    setActionError(null);
    try {
      await archiveService.mutateAsync(confirmArchive.id);
      setConfirmArchive(null);
    } catch (err) {
      setActionError(extractErrorMessage(err));
    }
  };

  const handleRestore = async (service: Service) => {
    setActionError(null);
    try {
      await restoreService.mutateAsync(service.id);
    } catch (err) {
      setActionError(extractErrorMessage(err));
    }
  };

  return (
    <div className="mx-auto w-full max-w-5xl p-6 sm:p-8 space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Services</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Define the services you offer and their default durations. Inactive
            or archived services remain preserved in historical appointments.
          </p>
        </div>
        <Button onClick={() => setCreateOpen(true)}>
          <Plus className="size-4" />
          New Service
        </Button>
      </div>

      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-3">
        <div
          role="tablist"
          aria-label="Service status filter"
          className="inline-flex flex-wrap items-center rounded-lg border border-border bg-card p-1 self-start"
        >
          {FILTER_TABS.map((tab) => {
            const count =
              tab.value === "active"
                ? activeCount
                : tab.value === "inactive"
                  ? inactiveCount
                  : tab.value === "archived"
                    ? archivedCount
                    : activeCount + inactiveCount + archivedCount;
            const isSelected = filter === tab.value;
            return (
              <button
                key={tab.value}
                type="button"
                role="tab"
                aria-selected={isSelected}
                onClick={() => setFilter(tab.value)}
                className={`inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition-colors ${
                  isSelected
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                <span>{tab.label}</span>
                <span
                  className={`rounded-full px-1.5 py-0.5 text-[10px] ${
                    isSelected
                      ? "bg-primary-foreground/20 text-primary-foreground"
                      : "bg-secondary text-secondary-foreground"
                  }`}
                >
                  {count}
                </span>
              </button>
            );
          })}
        </div>

        <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 flex-1 lg:max-w-lg">
          <div className="relative flex-1">
            <Search className="pointer-events-none absolute left-2.5 top-2.5 size-4 text-muted-foreground" />
            <Input
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              placeholder="Search service name or description..."
              aria-label="Search services"
              className="pl-8"
            />
          </div>

          <select
            aria-label="Sort services"
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as ServiceSortBy)}
            className="h-9 rounded-md border border-input bg-background px-3 text-sm text-foreground shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <option value="name_asc">Name (A–Z)</option>
            <option value="name_desc">Name (Z–A)</option>
            <option value="duration_asc">Shortest Duration</option>
            <option value="duration_desc">Longest Duration</option>
            <option value="recently_updated">Recently Updated</option>
          </select>
        </div>
      </div>

      {(isError || actionError) && (
        <div
          role="alert"
          className="rounded-xl border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive"
        >
          {actionError ?? `Unable to load services: ${extractErrorMessage(error)}`}
        </div>
      )}

      {isLoading ? (
        <div className="rounded-xl border border-border bg-card p-8 text-center text-sm text-muted-foreground">
          Loading services...
        </div>
      ) : items.length === 0 ? (
        <div className="rounded-xl border border-border bg-card p-10 text-center space-y-3">
          <div className="mx-auto size-10 rounded-full bg-secondary flex items-center justify-center text-muted-foreground">
            <Briefcase className="size-5" />
          </div>
          {debouncedSearch ? (
            <>
              <h2 className="text-base font-medium">No matching services</h2>
              <p className="text-sm text-muted-foreground max-w-sm mx-auto">
                No services matched &ldquo;{debouncedSearch}&rdquo; in this view.
              </p>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setSearchInput("")}
              >
                Clear Search
              </Button>
            </>
          ) : filter === "active" ? (
            <>
              <h2 className="text-base font-medium">No active services yet</h2>
              <p className="text-sm text-muted-foreground max-w-sm mx-auto">
                Add the services you offer so they can be selected when
                scheduling appointments.
              </p>
              <Button size="sm" onClick={() => setCreateOpen(true)}>
                <Plus className="size-4" />
                Add First Service
              </Button>
            </>
          ) : (
            <>
              <h2 className="text-base font-medium">
                No {filter} services
              </h2>
              <p className="text-sm text-muted-foreground max-w-sm mx-auto">
                Services in this state will appear here.
              </p>
            </>
          )}
        </div>
      ) : (
        <div className="rounded-xl border border-border bg-card overflow-hidden">
          <ul className="divide-y divide-border">
            {items.map((service) => (
              <li
                key={service.id}
                className="flex flex-col md:flex-row md:items-center justify-between gap-3 px-5 py-4"
              >
                <div className="min-w-0 space-y-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="font-medium text-sm">{service.name}</span>
                    <span className="inline-flex items-center gap-1 rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium text-secondary-foreground">
                      <Clock className="size-3" />
                      {formatDuration(service.default_duration_minutes)}
                    </span>
                    {service.is_archived ? (
                      <span className="inline-flex items-center gap-1 rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium text-muted-foreground">
                        <Archive className="size-3" />
                        Archived
                      </span>
                    ) : service.is_active ? (
                      <span className="inline-flex items-center gap-1 rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium text-secondary-foreground">
                        <CheckCircle2 className="size-3 text-primary" />
                        Active
                      </span>
                    ) : (
                      <span className="inline-flex items-center gap-1 rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium text-muted-foreground">
                        <PauseCircle className="size-3" />
                        Inactive
                      </span>
                    )}
                  </div>
                  {service.description && (
                    <p className="text-xs text-muted-foreground line-clamp-2">
                      {service.description}
                    </p>
                  )}
                </div>

                <div className="flex flex-wrap items-center gap-1.5 shrink-0">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setEditingService(service)}
                  >
                    <Pencil className="size-3.5" />
                    Edit
                  </Button>

                  {service.is_archived ? (
                    <Button
                      variant="secondary"
                      size="sm"
                      disabled={restoreService.isPending}
                      onClick={() => handleRestore(service)}
                    >
                      <ArchiveRestore className="size-3.5" />
                      Restore
                    </Button>
                  ) : (
                    <>
                      <Button
                        variant="outline"
                        size="sm"
                        disabled={
                          activateService.isPending || deactivateService.isPending
                        }
                        onClick={() => handleToggleActive(service)}
                      >
                        {service.is_active ? (
                          <>
                            <PauseCircle className="size-3.5" />
                            Deactivate
                          </>
                        ) : (
                          <>
                            <PlayCircle className="size-3.5" />
                            Activate
                          </>
                        )}
                      </Button>

                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setConfirmArchive(service)}
                      >
                        <Archive className="size-3.5" />
                        Archive
                      </Button>
                    </>
                  )}
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}

      <ServiceFormDialog
        open={createOpen}
        mode="create"
        isSubmitting={createService.isPending}
        onClose={() => setCreateOpen(false)}
        onSubmit={handleCreateSubmit}
      />

      <ServiceFormDialog
        open={Boolean(editingService)}
        mode="edit"
        initialService={editingService}
        isSubmitting={updateService.isPending}
        onClose={() => setEditingService(null)}
        onSubmit={handleEditSubmit}
      />

      {confirmArchive && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="archive-service-title"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4"
        >
          <div className="w-full max-w-md rounded-xl border border-border bg-card p-6 text-card-foreground shadow-lg space-y-4">
            <h2 id="archive-service-title" className="text-lg font-semibold">
              Archive &ldquo;{confirmArchive.name}&rdquo;?
            </h2>
            <p className="text-sm text-muted-foreground leading-relaxed">
              Archiving removes this service from new appointment pickers while
              keeping it intact in historical appointments. You can restore it
              at any time.
            </p>
            <div className="flex items-center justify-end gap-2 pt-2">
              <Button
                variant="outline"
                onClick={() => setConfirmArchive(null)}
                disabled={archiveService.isPending}
              >
                Cancel
              </Button>
              <Button
                onClick={handleConfirmArchive}
                disabled={archiveService.isPending}
              >
                {archiveService.isPending ? "Archiving..." : "Archive Service"}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
