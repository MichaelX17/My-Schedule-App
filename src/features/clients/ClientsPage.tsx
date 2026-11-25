import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  Archive,
  ChevronLeft,
  ChevronRight,
  Mail,
  Phone,
  Plus,
  Search,
  Users,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { extractErrorMessage } from "@/services/business";
import type { ClientArchiveFilter, ClientSortBy } from "@/services/clients";
import { ClientFormDialog } from "./components/ClientFormDialog";
import { useClientsList, useCreateClient } from "./hooks/useClients";
import type { normalizeClientFormInput } from "./validation";

const PAGE_SIZE = 25;

const FILTER_TABS: { value: ClientArchiveFilter; label: string }[] = [
  { value: "active", label: "Active" },
  { value: "archived", label: "Archived" },
  { value: "all", label: "All" },
];

export function ClientsPage() {
  const [searchInput, setSearchInput] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [filter, setFilter] = useState<ClientArchiveFilter>("active");
  const [sortBy, setSortBy] = useState<ClientSortBy>("name_asc");
  const [page, setPage] = useState(0);
  const [createOpen, setCreateOpen] = useState(false);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedSearch(searchInput.trim());
      setPage(0);
    }, 200);
    return () => window.clearTimeout(timer);
  }, [searchInput]);

  const { data, isLoading, isError, error } = useClientsList({
    search: debouncedSearch || null,
    filter,
    sort_by: sortBy,
    limit: PAGE_SIZE,
    offset: page * PAGE_SIZE,
  });

  const createClient = useCreateClient();

  const items = data?.items ?? [];
  const total = data?.total ?? 0;
  const activeCount = data?.active_count ?? 0;
  const archivedCount = data?.archived_count ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  const handleFilterChange = (nextFilter: ClientArchiveFilter) => {
    setFilter(nextFilter);
    setPage(0);
  };

  const handleSortChange = (nextSort: ClientSortBy) => {
    setSortBy(nextSort);
    setPage(0);
  };

  const handleCreateSubmit = async (
    values: ReturnType<typeof normalizeClientFormInput>
  ) => {
    await createClient.mutateAsync(values);
  };

  return (
    <div className="mx-auto w-full max-w-5xl p-6 sm:p-8 space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Clients</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Manage client contact profiles and general notes. Archived clients
            retain their historical records.
          </p>
        </div>
        <Button onClick={() => setCreateOpen(true)}>
          <Plus className="size-4" />
          New Client
        </Button>
      </div>

      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-3">
        <div
          role="tablist"
          aria-label="Client archive filter"
          className="inline-flex items-center rounded-lg border border-border bg-card p-1 self-start"
        >
          {FILTER_TABS.map((tab) => {
            const count =
              tab.value === "active"
                ? activeCount
                : tab.value === "archived"
                  ? archivedCount
                  : activeCount + archivedCount;
            const isSelected = filter === tab.value;
            return (
              <button
                key={tab.value}
                type="button"
                role="tab"
                aria-selected={isSelected}
                onClick={() => handleFilterChange(tab.value)}
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
              placeholder="Search name, phone, email, or notes..."
              aria-label="Search clients"
              className="pl-8"
            />
          </div>

          <select
            aria-label="Sort clients"
            value={sortBy}
            onChange={(e) => handleSortChange(e.target.value as ClientSortBy)}
            className="h-9 rounded-md border border-input bg-background px-3 text-sm text-foreground shadow-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <option value="name_asc">Name (A–Z)</option>
            <option value="name_desc">Name (Z–A)</option>
            <option value="recently_updated">Recently Updated</option>
            <option value="newest">Newest First</option>
          </select>
        </div>
      </div>

      {isError && (
        <div
          role="alert"
          className="rounded-xl border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive"
        >
          Unable to load clients: {extractErrorMessage(error)}
        </div>
      )}

      {isLoading ? (
        <div className="rounded-xl border border-border bg-card p-8 text-center text-sm text-muted-foreground">
          Loading clients...
        </div>
      ) : items.length === 0 ? (
        <div className="rounded-xl border border-border bg-card p-10 text-center space-y-3">
          <div className="mx-auto size-10 rounded-full bg-secondary flex items-center justify-center text-muted-foreground">
            <Users className="size-5" />
          </div>
          {debouncedSearch ? (
            <>
              <h2 className="text-base font-medium">No matching clients</h2>
              <p className="text-sm text-muted-foreground max-w-sm mx-auto">
                No clients matched &ldquo;{debouncedSearch}&rdquo; in this view.
              </p>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setSearchInput("")}
              >
                Clear Search
              </Button>
            </>
          ) : filter === "archived" ? (
            <>
              <h2 className="text-base font-medium">No archived clients</h2>
              <p className="text-sm text-muted-foreground max-w-sm mx-auto">
                Clients you archive will appear here and remain available in
                historical records.
              </p>
            </>
          ) : (
            <>
              <h2 className="text-base font-medium">No clients yet</h2>
              <p className="text-sm text-muted-foreground max-w-sm mx-auto">
                Add your first client to keep contact details and general notes
                organized locally.
              </p>
              <Button size="sm" onClick={() => setCreateOpen(true)}>
                <Plus className="size-4" />
                Add First Client
              </Button>
            </>
          )}
        </div>
      ) : (
        <div className="rounded-xl border border-border bg-card overflow-hidden">
          <ul className="divide-y divide-border">
            {items.map((client) => (
              <li key={client.id}>
                <Link
                  to={`/clients/${client.id}`}
                  className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 px-5 py-4 hover:bg-accent/40 transition-colors"
                >
                  <div className="min-w-0 space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-sm truncate">
                        {client.full_name}
                      </span>
                      {client.is_archived && (
                        <span className="inline-flex items-center gap-1 rounded-full bg-secondary px-2 py-0.5 text-[11px] font-medium text-secondary-foreground">
                          <Archive className="size-3" />
                          Archived
                        </span>
                      )}
                    </div>
                    <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground">
                      {client.phone && (
                        <span className="inline-flex items-center gap-1">
                          <Phone className="size-3" />
                          {client.phone}
                        </span>
                      )}
                      {client.email && (
                        <span className="inline-flex items-center gap-1">
                          <Mail className="size-3" />
                          {client.email}
                        </span>
                      )}
                      {!client.phone && !client.email && (
                        <span>No contact details recorded</span>
                      )}
                    </div>
                  </div>
                  <span className="text-xs text-muted-foreground shrink-0">
                    View details →
                  </span>
                </Link>
              </li>
            ))}
          </ul>

          {total > PAGE_SIZE && (
            <div className="flex items-center justify-between border-t border-border px-5 py-3 text-xs text-muted-foreground">
              <span>
                Showing {page * PAGE_SIZE + 1}–
                {Math.min((page + 1) * PAGE_SIZE, total)} of {total}
              </span>
              <div className="flex items-center gap-1.5">
                <Button
                  variant="outline"
                  size="sm"
                  disabled={page === 0}
                  onClick={() => setPage((p) => Math.max(0, p - 1))}
                >
                  <ChevronLeft className="size-3.5" />
                  Previous
                </Button>
                <span className="px-2">
                  Page {page + 1} of {totalPages}
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={page + 1 >= totalPages}
                  onClick={() => setPage((p) => p + 1)}
                >
                  Next
                  <ChevronRight className="size-3.5" />
                </Button>
              </div>
            </div>
          )}
        </div>
      )}

      <ClientFormDialog
        open={createOpen}
        mode="create"
        isSubmitting={createClient.isPending}
        onClose={() => setCreateOpen(false)}
        onSubmit={handleCreateSubmit}
      />
    </div>
  );
}
