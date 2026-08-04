"use client";

import { useState } from "react";
import {
  Plus,
  Search,
  Clock,
  CheckCircle2,
  AlertTriangle,
  Shield,
  ShieldAlert,
  Flame,
  ChevronDown,
  ChevronRight,
  X,
  ListChecks,
  Info,
} from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { DataState } from "@/components/ui/data-state";
import { useApiData, useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { formatNumber, relativeTime } from "@/lib/utils";

/** Mirrors the portal's `IncidentResponse`. */
interface IncidentRecord {
  id: string;
  incident_number: string;
  title: string;
  description: string | null;
  severity: string;
  status: string;
  priority: string;
  category: string | null;
  classification: string | null;
  commander_id: string | null;
  assigned_team: string | null;
  affected_systems: unknown;
  affected_users_count: number | null;
  impact_scope: string | null;
  mitre_tactics: unknown;
  mitre_techniques: unknown;
  attack_vector: string | null;
  root_cause: string | null;
  sla_breached: boolean;
  first_detected_at: string | null;
  first_response_at: string | null;
  contained_at: string | null;
  closed_at: string | null;
  created_at: string;
  updated_at: string;
}

/** Mirrors the portal's `TimelineEntry`. */
interface TimelineEntry {
  id: string;
  incident_id: string;
  actor_id: string | null;
  event_type: string;
  title: string;
  content: string | null;
  is_automated: boolean;
  occurred_at: string;
}

/** Mirrors the portal's `IncidentTask`. */
interface IncidentTask {
  id: string;
  incident_id: string;
  title: string;
  description: string | null;
  status: string;
  priority: string;
  assigned_to: string | null;
  due_at: string | null;
  completed_at: string | null;
  completed_by: string | null;
  created_at: string;
}

/** Mirrors the portal's `IncidentSummary`. */
interface IncidentSummary {
  total: number;
  open: number;
  critical: number;
  sla_breached: number;
  unassigned: number;
  closed_last_30d: number;
  mttr_minutes: number | null;
}

const severityVariants: Record<string, "critical" | "high" | "medium" | "low"> = {
  critical: "critical",
  high: "high",
  medium: "medium",
  low: "low",
};

const statusConfig: Record<string, { label: string; color: string; icon: typeof Clock }> = {
  open: { label: "Open", color: "bg-red-500/20 text-red-400 border-red-500/30", icon: AlertTriangle },
  triage: { label: "Triage", color: "bg-orange-500/20 text-orange-400 border-orange-500/30", icon: Search },
  investigating: { label: "Investigating", color: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30", icon: Search },
  contained: { label: "Contained", color: "bg-blue-500/20 text-blue-400 border-blue-500/30", icon: Shield },
  eradicated: { label: "Eradicated", color: "bg-blue-500/20 text-blue-400 border-blue-500/30", icon: Shield },
  resolved: { label: "Resolved", color: "bg-green-500/20 text-green-400 border-green-500/30", icon: CheckCircle2 },
  closed: { label: "Closed", color: "bg-green-500/20 text-green-400 border-green-500/30", icon: CheckCircle2 },
};

/** Statuses come from the database, so unknown values still need a presentation. */
function statusFor(status: string) {
  return (
    statusConfig[status] ?? {
      label: status.replace(/_/g, " "),
      color: "bg-muted text-muted-foreground border-border",
      icon: Clock,
    }
  );
}

/** MTTR arrives in minutes; hours read better past the hour mark. */
function formatMinutes(minutes: number | null | undefined): string {
  if (minutes === null || minutes === undefined || Number.isNaN(minutes)) return "—";
  if (minutes < 60) return `${Math.round(minutes)}m`;
  return `${(minutes / 60).toFixed(1)}h`;
}

/** A task counts as done once the backend records a completion. */
function isTaskDone(task: IncidentTask): boolean {
  return task.completed_at !== null || task.status === "completed" || task.status === "done";
}

export default function IncidentsPage() {
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedRow, setExpandedRow] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newSeverity, setNewSeverity] = useState<string>("medium");
  const [newDescription, setNewDescription] = useState("");

  const summary = useApiData<IncidentSummary>(() => api.incidents.summary());
  const incidents = useApiList<IncidentRecord>(() => api.incidents.list());

  // Detail data is only fetched for the row the analyst opened; `expandedRow`
  // goes through `deps` so the hook refetches when the selection changes, and
  // resolves empty rather than hitting the API when nothing is selected.
  const timeline = useApiData<TimelineEntry[]>(
    () => (expandedRow ? api.incidents.timeline(expandedRow) : Promise.resolve([])),
    [expandedRow]
  );
  const tasks = useApiData<IncidentTask[]>(
    () => (expandedRow ? api.incidents.tasks(expandedRow) : Promise.resolve([])),
    [expandedRow]
  );

  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  const createIncident = async () => {
    if (!newTitle.trim()) return;
    setSaving(true);
    setSaveError(null);
    try {
      await api.incidents.create({
        title: newTitle,
        description: newDescription || undefined,
        severity: newSeverity,
        priority: newSeverity,
      });
      setShowCreateModal(false);
      setNewTitle("");
      setNewDescription("");
      setNewSeverity("medium");
      incidents.refetch();
      summary.refetch();
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : "Failed to create incident");
    } finally {
      setSaving(false);
    }
  };

  const filtered = incidents.items.filter((incident) => {
    if (statusFilter !== "all" && incident.status !== statusFilter) return false;
    if (
      searchQuery &&
      !incident.title.toLowerCase().includes(searchQuery.toLowerCase()) &&
      !incident.incident_number.toLowerCase().includes(searchQuery.toLowerCase())
    )
      return false;
    return true;
  });

  const stats = [
    { label: "Open", value: formatNumber(summary.data?.open), icon: AlertTriangle, wrapper: "bg-red-500/10", color: "text-red-400" },
    { label: "Critical", value: formatNumber(summary.data?.critical), icon: Flame, wrapper: "bg-orange-500/10", color: "text-orange-400" },
    { label: "SLA Breached", value: formatNumber(summary.data?.sla_breached), icon: ShieldAlert, wrapper: "bg-yellow-500/10", color: "text-yellow-400" },
    { label: "Closed (30d)", value: formatNumber(summary.data?.closed_last_30d), icon: CheckCircle2, wrapper: "bg-green-500/10", color: "text-green-400" },
    { label: "MTTR", value: formatMinutes(summary.data?.mttr_minutes), icon: Clock, wrapper: "bg-purple-500/10", color: "text-purple-400" },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Incident Response</h2>
          <p className="text-muted-foreground">Manage and track security incidents</p>
        </div>
        <Button onClick={() => setShowCreateModal(true)}>
          <Plus className="mr-2 h-4 w-4" aria-hidden="true" />
          Create Incident
        </Button>
      </div>

      {/* Summary Cards */}
      <DataState
        loading={summary.loading}
        error={summary.error}
        onRetry={summary.refetch}
        loadingLabel="Loading incident summary"
      >
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-5">
          {stats.map((stat) => (
            <Card key={stat.label} className="border-border">
              <CardContent className="p-4">
                <div className="flex items-center gap-3">
                  <div className={`rounded-lg p-2 ${stat.wrapper}`}>
                    <stat.icon className={`h-5 w-5 ${stat.color}`} aria-hidden="true" />
                  </div>
                  <div>
                    <p className="text-2xl font-bold text-foreground">{stat.value}</p>
                    <p className="text-xs text-muted-foreground">{stat.label}</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </DataState>

      {/* Filter Tabs + Search */}
      <Card className="border-border">
        <CardContent className="p-4">
          <div className="flex flex-wrap items-center gap-4">
            <Tabs value={statusFilter} onValueChange={setStatusFilter} className="flex-1">
              <TabsList>
                <TabsTrigger value="all">All</TabsTrigger>
                <TabsTrigger value="open">Open</TabsTrigger>
                <TabsTrigger value="investigating">Investigating</TabsTrigger>
                <TabsTrigger value="contained">Contained</TabsTrigger>
                <TabsTrigger value="resolved">Resolved</TabsTrigger>
              </TabsList>
            </Tabs>
            <div className="relative min-w-[200px]">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
              <Input placeholder="Search incidents..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} className="pl-9" />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Incident List */}
      <DataState
        loading={incidents.loading}
        error={incidents.error}
        isEmpty={incidents.items.length === 0}
        onRetry={incidents.refetch}
        loadingLabel="Loading incidents"
        emptyTitle="No incidents yet"
        emptyDescription="Incidents raised from alerts or opened manually appear here."
      >
        <div className="space-y-2">
          {filtered.length === 0 && (
            <Card className="border-border"><CardContent className="p-8 text-center"><p className="text-muted-foreground">No incidents match the current filter.</p></CardContent></Card>
          )}
          {filtered.map((incident) => {
            const isExpanded = expandedRow === incident.id;
            const statusCfg = statusFor(incident.status);
            const StatusIcon = statusCfg.icon;
            const timelineEntries = timeline.data ?? [];
            const taskList = tasks.data ?? [];
            return (
              <Card key={incident.id} className="border-border hover:border-primary/30 transition-colors">
                <CardContent className="p-0">
                  <button
                    className="flex w-full items-center gap-4 p-4 text-left"
                    onClick={() => setExpandedRow(isExpanded ? null : incident.id)}
                    aria-expanded={isExpanded}
                    aria-label={`Expand incident ${incident.incident_number}`}
                  >
                    {isExpanded ? <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" aria-hidden="true" /> : <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" aria-hidden="true" />}
                    <span className="w-20 shrink-0 text-xs font-mono text-muted-foreground">{incident.incident_number}</span>
                    <span className="flex-1 truncate text-sm font-medium text-foreground">{incident.title}</span>
                    <Badge variant={severityVariants[incident.severity] ?? "outline"}>{incident.severity}</Badge>
                    <span className={`inline-flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-xs font-semibold ${statusCfg.color}`}>
                      <StatusIcon className="h-3 w-3" aria-hidden="true" />
                      {statusCfg.label}
                    </span>
                    <span className="hidden w-28 shrink-0 truncate text-xs text-muted-foreground lg:block">{incident.assigned_team ?? "Unassigned"}</span>
                    <span className="hidden w-32 shrink-0 text-xs text-muted-foreground md:block">{new Date(incident.created_at).toLocaleDateString()}</span>
                    <span className="hidden w-32 shrink-0 text-xs text-muted-foreground xl:block">{relativeTime(incident.updated_at)}</span>
                  </button>


                  {isExpanded && (
                    <div className="border-t border-border bg-muted/30 p-4 space-y-4">
                      <p className="text-sm text-muted-foreground">{incident.description ?? "No description recorded."}</p>
                      <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
                        {/* Timeline */}
                        <div className="space-y-2">
                          <h4 className="text-sm font-semibold text-foreground flex items-center gap-2"><Clock className="h-4 w-4" aria-hidden="true" /> Timeline</h4>
                          <DataState
                            loading={timeline.loading}
                            error={timeline.error}
                            isEmpty={timelineEntries.length === 0}
                            onRetry={timeline.refetch}
                            loadingLabel="Loading timeline"
                            emptyTitle="No timeline entries"
                          >
                            <div className="space-y-2 max-h-48 overflow-y-auto">
                              {timelineEntries.map((entry) => (
                                <div key={entry.id} className="flex gap-2 text-xs">
                                  <span className="shrink-0 text-muted-foreground w-14">{new Date(entry.occurred_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</span>
                                  <span className="text-foreground">{entry.title}</span>
                                  <span className="ml-auto text-muted-foreground">{entry.is_automated ? "System" : entry.event_type.replace(/_/g, " ")}</span>
                                </div>
                              ))}
                            </div>
                          </DataState>
                        </div>
                        {/* Tasks */}
                        <div className="space-y-2">
                          <h4 className="text-sm font-semibold text-foreground flex items-center gap-2"><ListChecks className="h-4 w-4" aria-hidden="true" /> Tasks</h4>
                          <DataState
                            loading={tasks.loading}
                            error={tasks.error}
                            isEmpty={taskList.length === 0}
                            onRetry={tasks.refetch}
                            loadingLabel="Loading tasks"
                            emptyTitle="No tasks assigned"
                          >
                            <div className="space-y-2 max-h-48 overflow-y-auto">
                              {taskList.map((task) => (
                                <div key={task.id} className="flex items-center gap-2">
                                  <Checkbox id={`${incident.id}-${task.id}`} checked={isTaskDone(task)} disabled />
                                  <label htmlFor={`${incident.id}-${task.id}`} className={`text-xs ${isTaskDone(task) ? "line-through text-muted-foreground" : "text-foreground"}`}>{task.title}</label>
                                  <Badge variant="outline" className="ml-auto text-xs">{task.priority}</Badge>
                                </div>
                              ))}
                            </div>
                          </DataState>
                        </div>

                        {/* Incident details */}
                        <div className="space-y-2">
                          <h4 className="text-sm font-semibold text-foreground flex items-center gap-2"><Info className="h-4 w-4" aria-hidden="true" /> Details</h4>
                          <dl className="space-y-1 text-xs">
                            <div className="flex justify-between gap-2">
                              <dt className="text-muted-foreground">Priority</dt>
                              <dd className="capitalize text-foreground">{incident.priority}</dd>
                            </div>
                            <div className="flex justify-between gap-2">
                              <dt className="text-muted-foreground">Category</dt>
                              <dd className="text-foreground">{incident.category ?? "—"}</dd>
                            </div>
                            <div className="flex justify-between gap-2">
                              <dt className="text-muted-foreground">Attack vector</dt>
                              <dd className="text-foreground">{incident.attack_vector ?? "—"}</dd>
                            </div>
                            <div className="flex justify-between gap-2">
                              <dt className="text-muted-foreground">Impact scope</dt>
                              <dd className="text-foreground">{incident.impact_scope ?? "—"}</dd>
                            </div>
                            <div className="flex justify-between gap-2">
                              <dt className="text-muted-foreground">Affected users</dt>
                              <dd className="text-foreground">{formatNumber(incident.affected_users_count)}</dd>
                            </div>
                            <div className="flex justify-between gap-2">
                              <dt className="text-muted-foreground">First detected</dt>
                              <dd className="text-foreground">{relativeTime(incident.first_detected_at)}</dd>
                            </div>
                            <div className="flex justify-between gap-2">
                              <dt className="text-muted-foreground">First response</dt>
                              <dd className="text-foreground">{relativeTime(incident.first_response_at)}</dd>
                            </div>
                            <div className="flex justify-between gap-2">
                              <dt className="text-muted-foreground">Contained</dt>
                              <dd className="text-foreground">{relativeTime(incident.contained_at)}</dd>
                            </div>
                            <div className="flex justify-between gap-2">
                              <dt className="text-muted-foreground">SLA</dt>
                              <dd>
                                {incident.sla_breached
                                  ? <Badge variant="destructive" className="text-xs">breached</Badge>
                                  : <Badge variant="low" className="text-xs">within target</Badge>}
                              </dd>
                            </div>
                          </dl>
                          {incident.root_cause && (
                            <p className="text-xs text-muted-foreground"><span className="font-medium text-foreground">Root cause:</span> {incident.root_cause}</p>
                          )}
                        </div>
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            );
          })}
          <p className="text-xs text-muted-foreground">
            Showing {formatNumber(filtered.length)} of {formatNumber(incidents.total)} incidents
          </p>
        </div>
      </DataState>


      {/* Create Incident Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" role="dialog" aria-modal="true" aria-labelledby="create-incident-title">
          <div className="w-full max-w-lg rounded-lg border border-border bg-card p-6 shadow-xl">
            <div className="flex items-center justify-between mb-4">
              <h3 id="create-incident-title" className="text-lg font-semibold text-foreground">Create Incident</h3>
              <Button variant="ghost" size="icon" onClick={() => setShowCreateModal(false)} aria-label="Close modal"><X className="h-4 w-4" aria-hidden="true" /></Button>
            </div>
            <div className="space-y-4">
              <div className="space-y-1">
                <Label htmlFor="incident-title">Title</Label>
                <Input id="incident-title" placeholder="Incident title..." value={newTitle} onChange={(e) => setNewTitle(e.target.value)} />
              </div>
              <div className="space-y-1">
                <Label htmlFor="incident-severity">Severity</Label>
                <Select value={newSeverity} onValueChange={setNewSeverity}>
                  <SelectTrigger id="incident-severity"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="critical">Critical</SelectItem>
                    <SelectItem value="high">High</SelectItem>
                    <SelectItem value="medium">Medium</SelectItem>
                    <SelectItem value="low">Low</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label htmlFor="incident-description">Description</Label>
                <textarea id="incident-description" className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2" placeholder="Describe the incident..." value={newDescription} onChange={(e) => setNewDescription(e.target.value)} />
              </div>
              <div className="space-y-1">
                <Label>Link Alerts</Label>
                <Input placeholder="e.g., ALT-101, ALT-102" disabled />
                <p className="text-xs text-muted-foreground">Alert linking available after creation</p>
              </div>
              {/* Create Incident Modal Actions */}
              <div className="flex items-center justify-between gap-2 pt-2">
                <p className="text-xs text-destructive">{saveError}</p>
                <div className="flex gap-2 ml-auto">
                  <Button variant="outline" onClick={() => setShowCreateModal(false)}>Cancel</Button>
                  <Button onClick={createIncident} disabled={saving || !newTitle.trim()}>
                    {saving ? "Creating..." : "Create Incident"}
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

