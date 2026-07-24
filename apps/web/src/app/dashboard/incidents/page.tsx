"use client";

import { useState } from "react";
import {
  Plus,
  Search,
  Clock,
  CheckCircle2,
  AlertTriangle,
  Shield,
  ChevronDown,
  ChevronRight,
  X,
  MessageSquare,
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

type IncidentSeverity = "critical" | "high" | "medium" | "low";
type IncidentStatus = "open" | "investigating" | "contained" | "resolved";

interface TimelineEvent {
  id: string;
  timestamp: string;
  action: string;
  actor: string;
}

interface PlaybookStep {
  id: string;
  label: string;
  completed: boolean;
}

interface Incident {
  id: string;
  title: string;
  severity: IncidentSeverity;
  status: IncidentStatus;
  assignedTo: string;
  createdAt: string;
  updatedAt: string;
  description: string;
  linkedAlerts: string[];
  timeline: TimelineEvent[];
  playbook: PlaybookStep[];
  notes: string[];
}

const mockIncidents: Incident[] = [
  {
    id: "INC-001",
    title: "Ransomware detected on workstation WS-042",
    severity: "critical",
    status: "investigating",
    assignedTo: "Sarah Chen",
    createdAt: "2024-01-15T08:30:00Z",
    updatedAt: "2024-01-15T10:45:00Z",
    description: "Ransomware binary executed on WS-042, lateral movement detected.",
    linkedAlerts: ["ALT-101", "ALT-102"],
    timeline: [
      { id: "t1", timestamp: "2024-01-15T08:30:00Z", action: "Incident created from alert ALT-101", actor: "System" },
      { id: "t2", timestamp: "2024-01-15T08:35:00Z", action: "Assigned to Sarah Chen", actor: "Admin" },
      { id: "t3", timestamp: "2024-01-15T09:00:00Z", action: "Status changed to Investigating", actor: "Sarah Chen" },
      { id: "t4", timestamp: "2024-01-15T10:45:00Z", action: "Isolated host WS-042", actor: "Sarah Chen" },
    ],
    playbook: [
      { id: "p1", label: "Identify affected systems", completed: true },
      { id: "p2", label: "Isolate compromised hosts", completed: true },
      { id: "p3", label: "Collect forensic evidence", completed: false },
      { id: "p4", label: "Eradicate threat", completed: false },
      { id: "p5", label: "Restore from clean backup", completed: false },
    ],
    notes: ["Malware hash matches Conti variant", "User clicked phishing link at 08:15"],
  },
  {
    id: "INC-002",
    title: "Unauthorized access to production database",
    severity: "high",
    status: "open",
    assignedTo: "Unassigned",
    createdAt: "2024-01-15T09:15:00Z",
    updatedAt: "2024-01-15T09:15:00Z",
    description: "Suspicious queries on prod-db-01 from unknown service account.",
    linkedAlerts: ["ALT-110"],
    timeline: [
      { id: "t1", timestamp: "2024-01-15T09:15:00Z", action: "Incident created from alert ALT-110", actor: "System" },
    ],
    playbook: [
      { id: "p1", label: "Identify the service account origin", completed: false },
      { id: "p2", label: "Revoke compromised credentials", completed: false },
      { id: "p3", label: "Audit accessed data", completed: false },
    ],
    notes: [],
  },
  {
    id: "INC-003",
    title: "DDoS attack on public API gateway",
    severity: "high",
    status: "contained",
    assignedTo: "Mike Torres",
    createdAt: "2024-01-14T22:00:00Z",
    updatedAt: "2024-01-15T06:30:00Z",
    description: "Volumetric DDoS targeting api.example.com, peaking at 45Gbps.",
    linkedAlerts: ["ALT-098", "ALT-099"],
    timeline: [
      { id: "t1", timestamp: "2024-01-14T22:00:00Z", action: "Incident created", actor: "System" },
      { id: "t2", timestamp: "2024-01-14T22:10:00Z", action: "Assigned to Mike Torres", actor: "Admin" },
      { id: "t3", timestamp: "2024-01-15T06:30:00Z", action: "Traffic normalized", actor: "Mike Torres" },
    ],
    playbook: [
      { id: "p1", label: "Enable rate limiting", completed: true },
      { id: "p2", label: "Activate DDoS mitigation", completed: true },
      { id: "p3", label: "Block malicious IPs", completed: true },
      { id: "p4", label: "Monitor for resurgence", completed: false },
    ],
    notes: ["Attack from botnet, mostly Eastern Europe IPs"],
  },
  {
    id: "INC-004",
    title: "Phishing campaign targeting finance team",
    severity: "medium",
    status: "resolved",
    assignedTo: "Sarah Chen",
    createdAt: "2024-01-13T14:00:00Z",
    updatedAt: "2024-01-14T11:00:00Z",
    description: "Coordinated phishing emails sent to 15 finance team members.",
    linkedAlerts: ["ALT-090"],
    timeline: [
      { id: "t1", timestamp: "2024-01-13T14:00:00Z", action: "Incident created", actor: "System" },
      { id: "t2", timestamp: "2024-01-14T11:00:00Z", action: "All accounts secured", actor: "Sarah Chen" },
    ],
    playbook: [
      { id: "p1", label: "Block phishing domain", completed: true },
      { id: "p2", label: "Reset affected passwords", completed: true },
      { id: "p3", label: "Security awareness reminder", completed: true },
    ],
    notes: ["Domain registered 24h before campaign"],
  },
  {
    id: "INC-005",
    title: "Exposed S3 bucket with customer PII",
    severity: "critical",
    status: "open",
    assignedTo: "Unassigned",
    createdAt: "2024-01-15T11:00:00Z",
    updatedAt: "2024-01-15T11:00:00Z",
    description: "Public S3 bucket with unencrypted customer records discovered.",
    linkedAlerts: ["ALT-115"],
    timeline: [
      { id: "t1", timestamp: "2024-01-15T11:00:00Z", action: "Incident created from scan", actor: "System" },
    ],
    playbook: [
      { id: "p1", label: "Restrict bucket permissions", completed: false },
      { id: "p2", label: "Audit access logs", completed: false },
      { id: "p3", label: "Assess data exposure scope", completed: false },
    ],
    notes: [],
  },
];

const teamMembers = ["Sarah Chen", "Mike Torres", "Alex Kim", "Jordan Lee", "Casey Nguyen"];

const statusConfig: Record<IncidentStatus, { label: string; color: string; icon: typeof Clock }> = {
  open: { label: "Open", color: "bg-red-500/20 text-red-400 border-red-500/30", icon: AlertTriangle },
  investigating: { label: "Investigating", color: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30", icon: Search },
  contained: { label: "Contained", color: "bg-blue-500/20 text-blue-400 border-blue-500/30", icon: Shield },
  resolved: { label: "Resolved", color: "bg-green-500/20 text-green-400 border-green-500/30", icon: CheckCircle2 },
};

export default function IncidentsPage() {
  const [statusFilter, setStatusFilter] = useState<IncidentStatus | "all">("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedRow, setExpandedRow] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [incidents, setIncidents] = useState(mockIncidents);
  const [newTitle, setNewTitle] = useState("");
  const [newSeverity, setNewSeverity] = useState<IncidentSeverity>("medium");
  const [newDescription, setNewDescription] = useState("");

  const filtered = incidents.filter((incident) => {
    if (statusFilter !== "all" && incident.status !== statusFilter) return false;
    if (searchQuery && !incident.title.toLowerCase().includes(searchQuery.toLowerCase()) && !incident.id.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });

  const stats = {
    open: incidents.filter((i) => i.status === "open").length,
    investigating: incidents.filter((i) => i.status === "investigating").length,
    contained: incidents.filter((i) => i.status === "contained").length,
    resolved: incidents.filter((i) => i.status === "resolved").length,
    mttr: "4.2h",
  };

  function handleCreateIncident() {
    if (!newTitle.trim()) return;
    const newIncident: Incident = {
      id: `INC-${String(incidents.length + 1).padStart(3, "0")}`,
      title: newTitle,
      severity: newSeverity,
      status: "open",
      assignedTo: "Unassigned",
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      description: newDescription,
      linkedAlerts: [],
      timeline: [{ id: "t1", timestamp: new Date().toISOString(), action: "Incident created manually", actor: "Current User" }],
      playbook: [],
      notes: [],
    };
    setIncidents([newIncident, ...incidents]);
    setNewTitle("");
    setNewSeverity("medium");
    setNewDescription("");
    setShowCreateModal(false);
  }

  function handlePlaybookToggle(incidentId: string, stepId: string) {
    setIncidents((prev) =>
      prev.map((inc) =>
        inc.id === incidentId
          ? { ...inc, playbook: inc.playbook.map((s) => (s.id === stepId ? { ...s, completed: !s.completed } : s)) }
          : inc
      )
    );
  }

  function handleAssign(incidentId: string, assignee: string) {
    setIncidents((prev) =>
      prev.map((inc) => (inc.id === incidentId ? { ...inc, assignedTo: assignee, updatedAt: new Date().toISOString() } : inc))
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Incident Response</h2>
          <p className="text-muted-foreground">Manage and track security incidents</p>
        </div>
        <Button onClick={() => setShowCreateModal(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Incident
        </Button>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-5">
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-red-500/10 p-2"><AlertTriangle className="h-5 w-5 text-red-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.open}</p><p className="text-xs text-muted-foreground">Open</p></div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-yellow-500/10 p-2"><Search className="h-5 w-5 text-yellow-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.investigating}</p><p className="text-xs text-muted-foreground">Investigating</p></div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-blue-500/10 p-2"><Shield className="h-5 w-5 text-blue-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.contained}</p><p className="text-xs text-muted-foreground">Contained</p></div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-green-500/10 p-2"><CheckCircle2 className="h-5 w-5 text-green-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.resolved}</p><p className="text-xs text-muted-foreground">Resolved (week)</p></div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-purple-500/10 p-2"><Clock className="h-5 w-5 text-purple-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.mttr}</p><p className="text-xs text-muted-foreground">MTTR</p></div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filter Tabs + Search */}
      <Card className="border-border">
        <CardContent className="p-4">
          <div className="flex flex-wrap items-center gap-4">
            <Tabs value={statusFilter} onValueChange={(v) => setStatusFilter(v as IncidentStatus | "all")} className="flex-1">
              <TabsList>
                <TabsTrigger value="all">All</TabsTrigger>
                <TabsTrigger value="open">Open</TabsTrigger>
                <TabsTrigger value="investigating">Investigating</TabsTrigger>
                <TabsTrigger value="contained">Contained</TabsTrigger>
                <TabsTrigger value="resolved">Resolved</TabsTrigger>
              </TabsList>
            </Tabs>
            <div className="relative min-w-[200px]">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input placeholder="Search incidents..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} className="pl-9" />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Incident Table */}
      <div className="space-y-2">
        {filtered.length === 0 && (
          <Card className="border-border"><CardContent className="p-8 text-center"><p className="text-muted-foreground">No incidents match the current filter.</p></CardContent></Card>
        )}
        {filtered.map((incident) => {
          const isExpanded = expandedRow === incident.id;
          const statusCfg = statusConfig[incident.status];
          const StatusIcon = statusCfg.icon;
          return (
            <Card key={incident.id} className="border-border hover:border-primary/30 transition-colors">
              <CardContent className="p-0">
                <button
                  className="flex w-full items-center gap-4 p-4 text-left"
                  onClick={() => setExpandedRow(isExpanded ? null : incident.id)}
                  aria-expanded={isExpanded}
                  aria-label={`Expand incident ${incident.id}`}
                >
                  {isExpanded ? <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" /> : <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />}
                  <span className="w-20 shrink-0 text-xs font-mono text-muted-foreground">{incident.id}</span>
                  <span className="flex-1 truncate text-sm font-medium text-foreground">{incident.title}</span>
                  <Badge variant={incident.severity}>{incident.severity}</Badge>
                  <span className={`inline-flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-xs font-semibold ${statusCfg.color}`}>
                    <StatusIcon className="h-3 w-3" />
                    {statusCfg.label}
                  </span>
                  <span className="hidden w-28 shrink-0 text-xs text-muted-foreground lg:block">{incident.assignedTo}</span>
                  <span className="hidden w-32 shrink-0 text-xs text-muted-foreground md:block">{new Date(incident.createdAt).toLocaleDateString()}</span>
                  <span className="hidden w-32 shrink-0 text-xs text-muted-foreground xl:block">{new Date(incident.updatedAt).toLocaleString()}</span>
                </button>

                {isExpanded && (
                  <div className="border-t border-border bg-muted/30 p-4 space-y-4">
                    <p className="text-sm text-muted-foreground">{incident.description}</p>
                    <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
                      {/* Timeline */}
                      <div className="space-y-2">
                        <h4 className="text-sm font-semibold text-foreground flex items-center gap-2"><Clock className="h-4 w-4" /> Timeline</h4>
                        <div className="space-y-2 max-h-48 overflow-y-auto">
                          {incident.timeline.map((event) => (
                            <div key={event.id} className="flex gap-2 text-xs">
                              <span className="shrink-0 text-muted-foreground w-14">{new Date(event.timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</span>
                              <span className="text-foreground">{event.action}</span>
                              <span className="ml-auto text-muted-foreground">{event.actor}</span>
                            </div>
                          ))}
                        </div>
                      </div>
                      {/* Playbook */}
                      <div className="space-y-2">
                        <h4 className="text-sm font-semibold text-foreground flex items-center gap-2"><CheckCircle2 className="h-4 w-4" /> Playbook</h4>
                        <div className="space-y-2 max-h-48 overflow-y-auto">
                          {incident.playbook.length === 0 && <p className="text-xs text-muted-foreground">No playbook assigned</p>}
                          {incident.playbook.map((step) => (
                            <div key={step.id} className="flex items-center gap-2">
                              <Checkbox id={`${incident.id}-${step.id}`} checked={step.completed} onCheckedChange={() => handlePlaybookToggle(incident.id, step.id)} />
                              <label htmlFor={`${incident.id}-${step.id}`} className={`text-xs ${step.completed ? "line-through text-muted-foreground" : "text-foreground"}`}>{step.label}</label>
                            </div>
                          ))}
                        </div>
                      </div>
                      {/* Notes + Assign */}
                      <div className="space-y-3">
                        <div className="space-y-2">
                          <h4 className="text-sm font-semibold text-foreground flex items-center gap-2"><MessageSquare className="h-4 w-4" /> Notes</h4>
                          <div className="space-y-1 max-h-24 overflow-y-auto">
                            {incident.notes.length === 0 && <p className="text-xs text-muted-foreground">No notes yet</p>}
                            {incident.notes.map((note, idx) => (<p key={idx} className="text-xs text-muted-foreground">&bull; {note}</p>))}
                          </div>
                        </div>
                        <div className="space-y-1">
                          <Label className="text-xs">Assign To</Label>
                          <Select value={incident.assignedTo} onValueChange={(v) => handleAssign(incident.id, v)}>
                            <SelectTrigger className="h-8 text-xs"><SelectValue placeholder="Select assignee" /></SelectTrigger>
                            <SelectContent>
                              {teamMembers.map((member) => (<SelectItem key={member} value={member}>{member}</SelectItem>))}
                            </SelectContent>
                          </Select>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Create Incident Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" role="dialog" aria-modal="true" aria-labelledby="create-incident-title">
          <div className="w-full max-w-lg rounded-lg border border-border bg-card p-6 shadow-xl">
            <div className="flex items-center justify-between mb-4">
              <h3 id="create-incident-title" className="text-lg font-semibold text-foreground">Create Incident</h3>
              <Button variant="ghost" size="icon" onClick={() => setShowCreateModal(false)} aria-label="Close modal"><X className="h-4 w-4" /></Button>
            </div>
            <div className="space-y-4">
              <div className="space-y-1">
                <Label htmlFor="incident-title">Title</Label>
                <Input id="incident-title" placeholder="Incident title..." value={newTitle} onChange={(e) => setNewTitle(e.target.value)} />
              </div>
              <div className="space-y-1">
                <Label htmlFor="incident-severity">Severity</Label>
                <Select value={newSeverity} onValueChange={(v) => setNewSeverity(v as IncidentSeverity)}>
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
              <div className="flex justify-end gap-2 pt-2">
                <Button variant="outline" onClick={() => setShowCreateModal(false)}>Cancel</Button>
                <Button onClick={handleCreateIncident} disabled={!newTitle.trim()}>Create Incident</Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
