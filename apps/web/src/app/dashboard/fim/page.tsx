"use client";

import { useState, useMemo } from "react";
import {
  FileWarning,
  Shield,
  Clock,
  Filter,
  Search,
  Download,
  FilePlus,
  FileX,
  FileEdit,
  RefreshCw,
  CheckCircle2,
  AlertTriangle,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

// --- Types ---

type ChangeType = "created" | "modified" | "deleted";
type Severity = "critical" | "high" | "medium" | "low";

interface FileChangeEvent {
  id: string;
  timestamp: string;
  filePath: string;
  changeType: ChangeType;
  hashBefore: string | null;
  hashAfter: string | null;
  severity: Severity;
  server: string;
  agent: string;
  user?: string;
}

interface BaselineEntry {
  id: string;
  name: string;
  createdAt: string;
  fileCount: number;
  server: string;
  status: "active" | "outdated" | "comparing";
}

// --- Mock Data ---

const mockEvents: FileChangeEvent[] = [
  { id: "1", timestamp: "2024-01-15T14:32:01Z", filePath: "/etc/shadow", changeType: "modified", hashBefore: "a3f2b8c1d4e5f6a7", hashAfter: "b7c9d0e1f2a3b4c5", severity: "critical", server: "prod-web-01", agent: "agent-fw-01", user: "root" },
  { id: "2", timestamp: "2024-01-15T14:28:45Z", filePath: "/usr/bin/curl", changeType: "modified", hashBefore: "d4e5f6a7b8c9d0e1", hashAfter: "f2a3b4c5d6e7f8a9", severity: "high", server: "prod-web-01", agent: "agent-fw-01" },
  { id: "3", timestamp: "2024-01-15T14:15:22Z", filePath: "/tmp/.hidden_script.sh", changeType: "created", hashBefore: null, hashAfter: "e7f8a9b0c1d2e3f4", severity: "critical", server: "prod-db-01", agent: "agent-fw-02" },
  { id: "4", timestamp: "2024-01-15T13:55:10Z", filePath: "/var/log/auth.log", changeType: "modified", hashBefore: "c1d2e3f4a5b6c7d8", hashAfter: "a5b6c7d8e9f0a1b2", severity: "low", server: "prod-web-02", agent: "agent-fw-03" },
  { id: "5", timestamp: "2024-01-15T13:42:33Z", filePath: "/etc/cron.d/backdoor", changeType: "created", hashBefore: null, hashAfter: "b0c1d2e3f4a5b6c7", severity: "critical", server: "prod-web-01", agent: "agent-fw-01" },
  { id: "6", timestamp: "2024-01-15T13:30:05Z", filePath: "/etc/nginx/nginx.conf", changeType: "modified", hashBefore: "f4a5b6c7d8e9f0a1", hashAfter: "d8e9f0a1b2c3d4e5", severity: "medium", server: "prod-web-02", agent: "agent-fw-03", user: "deploy" },
  { id: "7", timestamp: "2024-01-15T13:12:48Z", filePath: "/opt/app/config.yml", changeType: "modified", hashBefore: "a9b0c1d2e3f4a5b6", hashAfter: "e3f4a5b6c7d8e9f0", severity: "low", server: "prod-app-01", agent: "agent-fw-04", user: "deploy" },
  { id: "8", timestamp: "2024-01-15T12:58:19Z", filePath: "/var/spool/cron/root", changeType: "deleted", hashBefore: "b6c7d8e9f0a1b2c3", hashAfter: null, severity: "high", server: "prod-db-01", agent: "agent-fw-02" },
  { id: "9", timestamp: "2024-01-15T12:45:00Z", filePath: "/etc/passwd", changeType: "modified", hashBefore: "c7d8e9f0a1b2c3d4", hashAfter: "f0a1b2c3d4e5f6a7", severity: "critical", server: "prod-web-01", agent: "agent-fw-01" },
  { id: "10", timestamp: "2024-01-15T12:30:11Z", filePath: "/usr/local/bin/health_check", changeType: "created", hashBefore: null, hashAfter: "d2e3f4a5b6c7d8e9", severity: "medium", server: "prod-app-01", agent: "agent-fw-04", user: "deploy" },
];

const mockBaselines: BaselineEntry[] = [
  { id: "bl-1", name: "Production Baseline v2.1", createdAt: "2024-01-10T08:00:00Z", fileCount: 12450, server: "prod-web-01", status: "active" },
  { id: "bl-2", name: "Database Server Baseline", createdAt: "2024-01-08T10:30:00Z", fileCount: 8920, server: "prod-db-01", status: "active" },
  { id: "bl-3", name: "App Server Baseline v1.9", createdAt: "2024-01-05T14:00:00Z", fileCount: 10200, server: "prod-app-01", status: "outdated" },
  { id: "bl-4", name: "Web Server 02 Baseline", createdAt: "2024-01-12T09:15:00Z", fileCount: 11800, server: "prod-web-02", status: "active" },
];

const servers = ["prod-web-01", "prod-web-02", "prod-db-01", "prod-app-01"];

// --- Helpers ---

function formatTimestamp(iso: string): string {
  const date = new Date(iso);
  return date.toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  });
}

function truncateHash(hash: string | null): string {
  if (!hash) return "—";
  return hash.slice(0, 8) + "…";
}

const severityOrder: Record<Severity, number> = { critical: 0, high: 1, medium: 2, low: 3 };

const changeTypeIcons: Record<ChangeType, typeof FilePlus> = {
  created: FilePlus,
  modified: FileEdit,
  deleted: FileX,
};

const changeTypeColors: Record<ChangeType, string> = {
  created: "text-green-400",
  modified: "text-yellow-400",
  deleted: "text-red-400",
};

// --- Component ---

export default function FIMPage() {
  const [searchQuery, setSearchQuery] = useState("");
  const [serverFilter, setServerFilter] = useState<string>("all");
  const [severityFilter, setSeverityFilter] = useState<string>("all");
  const [changeTypeFilter, setChangeTypeFilter] = useState<string>("all");
  const [activeTab, setActiveTab] = useState("events");

  const filteredEvents = useMemo(() => {
    return mockEvents
      .filter((event) => {
        if (searchQuery && !event.filePath.toLowerCase().includes(searchQuery.toLowerCase())) return false;
        if (serverFilter !== "all" && event.server !== serverFilter) return false;
        if (severityFilter !== "all" && event.severity !== severityFilter) return false;
        if (changeTypeFilter !== "all" && event.changeType !== changeTypeFilter) return false;
        return true;
      })
      .sort((a, b) => severityOrder[a.severity] - severityOrder[b.severity]);
  }, [searchQuery, serverFilter, severityFilter, changeTypeFilter]);

  const stats = useMemo(() => ({
    totalChanges: mockEvents.length,
    criticalChanges: mockEvents.filter((e) => e.severity === "critical").length,
    modifiedFiles: mockEvents.filter((e) => e.changeType === "modified").length,
    newFiles: mockEvents.filter((e) => e.changeType === "created").length,
    deletedFiles: mockEvents.filter((e) => e.changeType === "deleted").length,
  }), []);

  const statCards = [
    { label: "Total Changes", value: stats.totalChanges, icon: FileWarning, color: "text-blue-400" },
    { label: "Critical Changes", value: stats.criticalChanges, icon: AlertTriangle, color: "text-red-400" },
    { label: "Modified Files", value: stats.modifiedFiles, icon: FileEdit, color: "text-yellow-400" },
    { label: "New Files", value: stats.newFiles, icon: FilePlus, color: "text-green-400" },
    { label: "Deleted Files", value: stats.deletedFiles, icon: FileX, color: "text-red-400" },
  ];

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">File Integrity Monitoring</h2>
          <p className="text-muted-foreground">
            Track file system changes and detect unauthorized modifications
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm">
            <Download className="mr-2 h-4 w-4" />
            Export Report
          </Button>
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Scan Now
          </Button>
        </div>
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-5">
        {statCards.map((stat) => (
          <Card key={stat.label} className="border-border">
            <CardContent className="flex items-center gap-3 p-4">
              <stat.icon className={`h-8 w-8 ${stat.color}`} />
              <div>
                <p className="text-2xl font-bold text-foreground">{stat.value}</p>
                <p className="text-xs text-muted-foreground">{stat.label}</p>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="events">
            <Clock className="mr-2 h-4 w-4" />
            Change Events
          </TabsTrigger>
          <TabsTrigger value="baselines">
            <Shield className="mr-2 h-4 w-4" />
            Baseline Management
          </TabsTrigger>
        </TabsList>

        <TabsContent value="events" className="space-y-4">
          <FiltersBar
            searchQuery={searchQuery}
            setSearchQuery={setSearchQuery}
            serverFilter={serverFilter}
            setServerFilter={setServerFilter}
            severityFilter={severityFilter}
            setSeverityFilter={setSeverityFilter}
            changeTypeFilter={changeTypeFilter}
            setChangeTypeFilter={setChangeTypeFilter}
          />
          <EventsTable events={filteredEvents} totalCount={mockEvents.length} />
        </TabsContent>

        <TabsContent value="baselines" className="space-y-4">
          <BaselineManagement baselines={mockBaselines} />
        </TabsContent>
      </Tabs>
    </div>
  );
}

// --- Filters Bar ---

function FiltersBar({
  searchQuery,
  setSearchQuery,
  serverFilter,
  setServerFilter,
  severityFilter,
  setSeverityFilter,
  changeTypeFilter,
  setChangeTypeFilter,
}: {
  searchQuery: string;
  setSearchQuery: (v: string) => void;
  serverFilter: string;
  setServerFilter: (v: string) => void;
  severityFilter: string;
  setSeverityFilter: (v: string) => void;
  changeTypeFilter: string;
  setChangeTypeFilter: (v: string) => void;
}) {
  return (
    <Card className="border-border">
      <CardContent className="p-4">
        <div className="flex flex-wrap items-center gap-3">
          <div className="flex items-center gap-2">
            <Filter className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm font-medium text-muted-foreground">Filters:</span>
          </div>
          <div className="relative min-w-[200px] max-w-sm flex-1">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="Search file path..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9"
              aria-label="Search file path"
            />
          </div>
          <Select value={serverFilter} onValueChange={setServerFilter}>
            <SelectTrigger className="w-[160px]" aria-label="Filter by server">
              <SelectValue placeholder="Server" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Servers</SelectItem>
              {servers.map((s) => (
                <SelectItem key={s} value={s}>{s}</SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={severityFilter} onValueChange={setSeverityFilter}>
            <SelectTrigger className="w-[140px]" aria-label="Filter by severity">
              <SelectValue placeholder="Severity" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Severity</SelectItem>
              <SelectItem value="critical">Critical</SelectItem>
              <SelectItem value="high">High</SelectItem>
              <SelectItem value="medium">Medium</SelectItem>
              <SelectItem value="low">Low</SelectItem>
            </SelectContent>
          </Select>
          <Select value={changeTypeFilter} onValueChange={setChangeTypeFilter}>
            <SelectTrigger className="w-[150px]" aria-label="Filter by change type">
              <SelectValue placeholder="Change Type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Types</SelectItem>
              <SelectItem value="created">Created</SelectItem>
              <SelectItem value="modified">Modified</SelectItem>
              <SelectItem value="deleted">Deleted</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </CardContent>
    </Card>
  );
}

// --- Events Table ---

function EventsTable({ events, totalCount }: { events: FileChangeEvent[]; totalCount: number }) {
  return (
    <>
      <Card className="border-border">
        <div className="overflow-x-auto">
          <table className="w-full text-sm" role="table">
            <thead>
              <tr className="border-b border-border bg-muted/50">
                <th className="px-4 py-3 text-left font-medium text-muted-foreground">Timestamp</th>
                <th className="px-4 py-3 text-left font-medium text-muted-foreground">File Path</th>
                <th className="px-4 py-3 text-left font-medium text-muted-foreground">Change Type</th>
                <th className="px-4 py-3 text-left font-medium text-muted-foreground">Hash (Before → After)</th>
                <th className="px-4 py-3 text-left font-medium text-muted-foreground">Severity</th>
                <th className="px-4 py-3 text-left font-medium text-muted-foreground">Server / Agent</th>
              </tr>
            </thead>
            <tbody>
              {events.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-4 py-8 text-center text-muted-foreground">
                    No file change events match your filters.
                  </td>
                </tr>
              ) : (
                events.map((event) => {
                  const ChangeIcon = changeTypeIcons[event.changeType];
                  return (
                    <tr key={event.id} className="border-b border-border transition-colors hover:bg-muted/30">
                      <td className="whitespace-nowrap px-4 py-3 font-mono text-xs text-muted-foreground">
                        {formatTimestamp(event.timestamp)}
                      </td>
                      <td className="px-4 py-3">
                        <code className="rounded bg-muted px-1.5 py-0.5 text-xs text-foreground">
                          {event.filePath}
                        </code>
                      </td>
                      <td className="px-4 py-3">
                        <span className={`inline-flex items-center gap-1.5 text-xs font-medium capitalize ${changeTypeColors[event.changeType]}`}>
                          <ChangeIcon className="h-3.5 w-3.5" />
                          {event.changeType}
                        </span>
                      </td>
                      <td className="whitespace-nowrap px-4 py-3 font-mono text-xs text-muted-foreground">
                        <span>{truncateHash(event.hashBefore)}</span>
                        <span className="mx-1 text-muted-foreground/50">→</span>
                        <span>{truncateHash(event.hashAfter)}</span>
                      </td>
                      <td className="px-4 py-3">
                        <Badge variant={event.severity}>{event.severity}</Badge>
                      </td>
                      <td className="px-4 py-3">
                        <div className="text-xs">
                          <p className="font-medium text-foreground">{event.server}</p>
                          <p className="text-muted-foreground">{event.agent}</p>
                        </div>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>
      </Card>
      <p className="text-xs text-muted-foreground">
        Showing {events.length} of {totalCount} events
      </p>
    </>
  );
}

// --- Baseline Management ---

function BaselineManagement({ baselines }: { baselines: BaselineEntry[] }) {
  return (
    <>
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Manage file integrity baselines for your monitored servers.
        </p>
        <Button size="sm">
          <Shield className="mr-2 h-4 w-4" />
          Create New Baseline
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {baselines.map((baseline) => (
          <Card key={baseline.id} className="border-border transition-colors hover:border-primary/30">
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="text-sm font-medium">{baseline.name}</CardTitle>
                <Badge
                  variant={
                    baseline.status === "active" ? "low" : baseline.status === "outdated" ? "medium" : "default"
                  }
                >
                  {baseline.status}
                </Badge>
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="grid grid-cols-2 gap-2 text-xs">
                <div>
                  <p className="text-muted-foreground">Server</p>
                  <p className="font-medium text-foreground">{baseline.server}</p>
                </div>
                <div>
                  <p className="text-muted-foreground">Files Tracked</p>
                  <p className="font-medium text-foreground">{baseline.fileCount.toLocaleString()}</p>
                </div>
                <div>
                  <p className="text-muted-foreground">Created</p>
                  <p className="font-medium text-foreground">{formatTimestamp(baseline.createdAt)}</p>
                </div>
                <div>
                  <p className="text-muted-foreground">Status</p>
                  <p className="flex items-center gap-1 font-medium text-foreground">
                    {baseline.status === "active" ? (
                      <CheckCircle2 className="h-3 w-3 text-green-400" />
                    ) : (
                      <AlertTriangle className="h-3 w-3 text-yellow-400" />
                    )}
                    {baseline.status === "active" ? "Up to date" : "Needs refresh"}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2 border-t border-border pt-3">
                <Button variant="outline" size="sm" className="flex-1">
                  <RefreshCw className="mr-1.5 h-3.5 w-3.5" />
                  Update
                </Button>
                <Button variant="outline" size="sm" className="flex-1">
                  <Search className="mr-1.5 h-3.5 w-3.5" />
                  Compare
                </Button>
                <Button variant="ghost" size="sm">
                  <Download className="h-3.5 w-3.5" />
                  <span className="sr-only">Download baseline</span>
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </>
  );
}

