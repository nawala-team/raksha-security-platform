"use client";

import { useState } from "react";
import { AlertTriangle, Search, Clock, CheckCircle2, XCircle } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { DataState } from "@/components/ui/data-state";
import { useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";

type ThreatLevel = "critical" | "high" | "medium" | "low" | "info";
type AlertStatus = "active" | "acknowledged" | "resolved" | "false_positive";

interface AlertRow {
  id: string;
  title: string;
  description: string;
  severity: ThreatLevel;
  status: AlertStatus;
  source: string;
  created_at: string;
}

const statusIcons = { active: AlertTriangle, acknowledged: Clock, resolved: CheckCircle2, false_positive: XCircle };

export default function AlertsPage() {
  const { items, loading, error, refetch } = useApiList<AlertRow>(() => api.alerts.list());
  const [searchQuery, setSearchQuery] = useState("");
  const [severityFilter, setSeverityFilter] = useState<ThreatLevel | "all">("all");
  const [busyId, setBusyId] = useState<string | null>(null);

  const filtered = items.filter((a) => {
    if (severityFilter !== "all" && a.severity !== severityFilter) return false;
    if (searchQuery && !a.title.toLowerCase().includes(searchQuery.toLowerCase()) && !a.source.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });
  const activeCount = items.filter((a) => a.status === "active").length;

  const changeStatus = async (row: AlertRow, status: AlertStatus) => {
    setBusyId(row.id);
    try {
      if (status === "acknowledged") await api.alerts.acknowledge(row.id);
      else if (status === "resolved") await api.alerts.resolve(row.id);
      refetch();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : "Failed to update alert");
    } finally {
      setBusyId(null);
    }
  };

  const statusColor = (s: AlertStatus) =>
    s === "active" ? "text-red-400" : s === "acknowledged" ? "text-yellow-400" : "text-green-400";

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Alerts</h2>
          <p className="text-muted-foreground">Monitor and manage security alerts</p>
        </div>
        <Badge variant="destructive">{activeCount} Active</Badge>
      </div>

      <Card className="border-border">
        <CardContent className="p-4">
          <div className="flex flex-wrap items-center gap-3">
            <div className="relative flex-1 min-w-[200px]">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input placeholder="Search alerts..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} className="pl-9" />
            </div>
            <div className="flex gap-2">
              {(["all", "critical", "high", "medium", "low"] as const).map((level) => (
                <Button key={level} variant={severityFilter === level ? "default" : "outline"} size="sm" onClick={() => setSeverityFilter(level)} className="capitalize">{level}</Button>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      <DataState loading={loading} error={error} isEmpty={filtered.length === 0} onRetry={refetch} loadingLabel="Loading alerts" emptyTitle="No alerts" emptyDescription="Alerts from agents and detectors will appear here.">
        <div className="space-y-3">
          {filtered.map((alert) => {
            const StatusIcon = statusIcons[alert.status] ?? Clock;
            return (
              <Card key={alert.id} className="border-border hover:border-primary/30 transition-colors">
                <CardContent className="p-4">
                  <div className="flex items-start gap-4">
                    <StatusIcon className={`h-5 w-5 mt-0.5 shrink-0 ${statusColor(alert.status)}`} />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1 flex-wrap">
                        <h3 className="text-sm font-medium truncate">{alert.title}</h3>
                        <Badge variant={alert.severity as "critical" | "high" | "medium" | "low"}>{alert.severity}</Badge>
                        <span className="text-xs capitalize text-muted-foreground">{alert.status}</span>
                      </div>
                      <p className="text-xs text-muted-foreground mb-2">{alert.description}</p>
                      <div className="flex items-center gap-2 text-xs text-muted-foreground">
                        <span>{alert.source}</span><span>•</span>
                        <span>{new Date(alert.created_at).toLocaleString()}</span>
                      </div>
                    </div>
                    <div className="flex flex-col gap-2">
                      {alert.status === "active" && (
                        <Button variant="outline" size="sm" disabled={busyId === alert.id} onClick={() => changeStatus(alert, "acknowledged")}>Acknowledge</Button>
                      )}
                      {(alert.status === "active" || alert.status === "acknowledged") && (
                        <Button variant="outline" size="sm" disabled={busyId === alert.id} onClick={() => changeStatus(alert, "resolved")} className="text-green-400 hover:text-green-300">Resolve</Button>
                      )}
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      </DataState>
    </div>
  );
}
