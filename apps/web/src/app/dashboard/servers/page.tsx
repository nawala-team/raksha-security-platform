"use client";

import { Server } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { DataState } from "@/components/ui/data-state";
import { useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { relativeTime } from "@/lib/utils";

/** Mirrors the portal's `ServerResponse`. */
interface ServerRow {
  id: string;
  hostname: string;
  display_name: string | null;
  environment: string;
  role: string | null;
  provider: string | null;
  region: string | null;
  ip_address: string | null;
  os_family: string | null;
  os_version: string | null;
  status: string;
  cpu_usage_pct: number | null;
  memory_usage_pct: number | null;
  disk_usage_pct: number | null;
  last_seen_at: string | null;
}

const statusColors: Record<string, string> = {
  online: "bg-green-500",
  offline: "bg-red-500",
  degraded: "bg-yellow-500",
  maintenance: "bg-blue-500",
};

/** Coloured usage meter; renders an em dash when the agent reported nothing. */
function UsageBar({ value, label }: { value: number | null; label: string }) {
  const pct = Math.round(value ?? 0);
  const color = pct > 85 ? "text-red-400" : pct > 70 ? "text-yellow-400" : "text-green-400";
  return (
    <div className="space-y-1">
      <div className="flex justify-between text-xs">
        <span className="text-muted-foreground">{label}</span>
        <span className={color}>{value === null ? "—" : `${pct}%`}</span>
      </div>
      <Progress value={pct} className="h-1.5" />
    </div>
  );
}

export default function ServersPage() {
  const { items, loading, error, refetch } = useApiList<ServerRow>(() =>
    api.servers.list()
  );

  const online = items.filter((s) => s.status === "online").length;
  const offline = items.filter((s) => s.status === "offline").length;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Servers</h2>
          <p className="text-muted-foreground">Monitor server health and performance</p>
        </div>
        {!loading && !error && (
          <div className="flex items-center gap-2">
            <Badge variant="default">{online} Online</Badge>
            <Badge variant="destructive">{offline} Offline</Badge>
          </div>
        )}
      </div>

      <DataState
        loading={loading}
        error={error}
        isEmpty={items.length === 0}
        onRetry={refetch}
        loadingLabel="Loading servers"
        emptyTitle="No servers registered"
        emptyDescription="Servers appear here once an agent checks in."
      >
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
          {items.map((server) => (
            <Card
              key={server.id}
              className="border-border hover:border-primary/30 transition-colors"
            >
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <CardTitle className="flex items-center gap-2 text-sm">
                    <Server className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
                    {server.display_name || server.hostname}
                  </CardTitle>
                  <div className="flex items-center gap-2">
                    <span
                      className={`h-2.5 w-2.5 rounded-full ${
                        statusColors[server.status] ?? "bg-muted"
                      }`}
                      aria-hidden="true"
                    />
                    <span className="text-xs capitalize text-muted-foreground">
                      {server.status}
                    </span>
                  </div>
                </div>
                <p className="text-xs text-muted-foreground">
                  {server.ip_address ?? "no IP"} • {server.os_family ?? "unknown OS"}
                  {server.os_version ? ` ${server.os_version}` : ""} •{" "}
                  {relativeTime(server.last_seen_at)}
                </p>
              </CardHeader>
              <CardContent className="space-y-3">
                <UsageBar value={server.cpu_usage_pct} label="CPU" />
                <UsageBar value={server.memory_usage_pct} label="Memory" />
                <UsageBar value={server.disk_usage_pct} label="Disk" />
                <div className="flex flex-wrap gap-1.5 pt-2 border-t border-border">
                  <Badge variant="outline" className="text-xs capitalize">
                    {server.environment}
                  </Badge>
                  {server.role && (
                    <Badge variant="outline" className="text-xs">
                      {server.role}
                    </Badge>
                  )}
                  {server.provider && (
                    <Badge variant="outline" className="text-xs">
                      {server.provider}
                      {server.region ? ` / ${server.region}` : ""}
                    </Badge>
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </DataState>
    </div>
  );
}
