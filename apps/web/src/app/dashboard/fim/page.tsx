"use client";

import { useMemo, useState } from "react";
import { FileWarning, FilePlus, FileX, FileEdit, Search, Download } from "lucide-react";
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
import { DataState } from "@/components/ui/data-state";
import { useApiData, useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";

type ChangeType = "created" | "modified" | "deleted" | "permission_changed";
type Severity = "critical" | "high" | "medium" | "low" | "info";

/** Mirrors the portal's `FimEventResponse`. */
interface FimEvent {
  id: string;
  agent_id: string;
  hostname: string;
  event_type: string;
  severity: string;
  file_path: string;
  file_name: string;
  directory: string;
  detected_at: string;
}

/** Mirrors the portal's `FimSummary`. */
interface FimSummary {
  events_24h: number;
  critical_24h: number;
  created_24h: number;
  modified_24h: number;
  deleted_24h: number;
  permission_changes_24h: number;
  monitored_hosts: number;
  baselines: number;
}

const changeTypeIcons: Record<string, typeof FilePlus> = {
  created: FilePlus,
  modified: FileEdit,
  deleted: FileX,
};
const changeTypeColors: Record<string, string> = {
  created: "text-green-400",
  modified: "text-yellow-400",
  deleted: "text-red-400",
  permission_changed: "text-blue-400",
};

export default function FIMPage() {
  const { items: events, loading, error, refetch } = useApiList<FimEvent>(() => api.fim.events());
  const { data: summary } = useApiData<FimSummary>(() => api.fim.summary());
  const [searchQuery, setSearchQuery] = useState("");
  const [severityFilter, setSeverityFilter] = useState("all");

  const filtered = useMemo(
    () =>
      events.filter((e) => {
        if (severityFilter !== "all" && e.severity !== severityFilter) return false;
        if (searchQuery && !e.file_path.toLowerCase().includes(searchQuery.toLowerCase())) return false;
        return true;
      }),
    [events, severityFilter, searchQuery]
  );

  const exportCsv = () => {
    const header = "id,hostname,event_type,severity,file_path,event_time";
    const rows = filtered.map((e) =>
      [e.id, e.hostname, e.event_type, e.severity, e.file_path, e.detected_at]
        .map((c) => `"${String(c).replace(/"/g, '""')}"`)
        .join(",")
    );
    const blob = new Blob([[header, ...rows].join("\n")], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "raksha-fim-events.csv";
    a.click();
    URL.revokeObjectURL(url);
  };

  const statCards = [
    { label: "Events (24h)", value: summary?.events_24h ?? 0, icon: FileWarning, color: "text-blue-400" },
    { label: "Critical (24h)", value: summary?.critical_24h ?? 0, icon: FileWarning, color: "text-red-400" },
    { label: "Modified", value: summary?.modified_24h ?? 0, icon: FileEdit, color: "text-yellow-400" },
    { label: "Created", value: summary?.created_24h ?? 0, icon: FilePlus, color: "text-green-400" },
    { label: "Deleted", value: summary?.deleted_24h ?? 0, icon: FileX, color: "text-red-400" },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">File Integrity Monitoring</h2>
          <p className="text-muted-foreground">Track file system changes and detect unauthorized modifications</p>
        </div>
        <Button variant="outline" size="sm" onClick={exportCsv} disabled={filtered.length === 0}>
          <Download className="mr-2 h-4 w-4" aria-hidden="true" /> Export CSV
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-5">
        {statCards.map((stat) => (
          <Card key={stat.label} className="border-border">
            <CardContent className="flex items-center gap-3 p-4">
              <stat.icon className={`h-8 w-8 ${stat.color}`} aria-hidden="true" />
              <div>
                <p className="text-2xl font-bold text-foreground">{stat.value}</p>
                <p className="text-xs text-muted-foreground">{stat.label}</p>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Card className="border-border">
        <CardContent className="p-4">
          <div className="flex flex-wrap items-center gap-3">
            <div className="relative min-w-[200px] max-w-sm flex-1">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
              <Input placeholder="Search file path..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} className="pl-9" aria-label="Search file path" />
            </div>
            <Select value={severityFilter} onValueChange={setSeverityFilter}>
              <SelectTrigger className="w-[150px]" aria-label="Filter by severity">
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
          </div>
        </CardContent>
      </Card>

      <DataState
        loading={loading}
        error={error}
        isEmpty={filtered.length === 0}
        onRetry={refetch}
        loadingLabel="Loading FIM events"
        emptyTitle="No file changes detected"
        emptyDescription="File integrity events will appear here once monitored agents report changes."
      >
        <Card className="border-border">
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Change Events</CardTitle>
          </CardHeader>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border bg-muted/30 text-left">
                  <th className="px-4 py-3 font-medium text-muted-foreground">File</th>
                  <th className="px-4 py-3 font-medium text-muted-foreground">Type</th>
                  <th className="px-4 py-3 font-medium text-muted-foreground">Severity</th>
                  <th className="px-4 py-3 font-medium text-muted-foreground">Host</th>
                  <th className="px-4 py-3 font-medium text-muted-foreground">Time</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {filtered.map((e) => {
                  const Icon = changeTypeIcons[e.event_type] ?? FileWarning;
                  return (
                    <tr key={e.id} className="hover:bg-accent/50">
                      <td className="px-4 py-3 font-mono text-xs">{e.file_path}</td>
                      <td className="px-4 py-3">
                        <span className={`inline-flex items-center gap-1.5 text-xs ${changeTypeColors[e.event_type] ?? "text-muted-foreground"}`}>
                          <Icon className="h-3.5 w-3.5" aria-hidden="true" />
                          {e.event_type}
                        </span>
                      </td>
                      <td className="px-4 py-3">
                        <Badge variant={e.severity as "critical" | "high" | "medium" | "low"}>{e.severity}</Badge>
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">{e.hostname}</td>
                      <td className="px-4 py-3 text-muted-foreground">{new Date(e.detected_at).toLocaleString()}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </Card>
      </DataState>
    </div>
  );
}

