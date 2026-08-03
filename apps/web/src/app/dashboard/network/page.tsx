"use client";

import { ArrowUpDown } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { DataState } from "@/components/ui/data-state";
import { useApiData, useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { formatBytes, formatNumber } from "@/lib/utils";

/** Mirrors the portal's `NetworkEventResponse`. */
interface NetworkEvent {
  id: string;
  event_type: string;
  severity: string;
  protocol: string | null;
  source_ip: string | null;
  source_port: number | null;
  dest_ip: string | null;
  dest_port: number | null;
  direction: string | null;
  action: string | null;
  bytes_sent: number | null;
  bytes_received: number | null;
  process_name: string | null;
  country_code: string | null;
  is_threat: boolean;
  occurred_at: string;
}

/** Mirrors the portal's `NetworkSummary`. */
interface NetworkSummary {
  events_24h: number;
  blocked_24h: number;
  threats_24h: number;
  port_scans_24h: number;
  bytes_in_24h: number;
  bytes_out_24h: number;
  active_rules: number;
}

const actionColors: Record<string, string> = {
  allow: "text-green-400",
  accept: "text-green-400",
  block: "text-red-400",
  drop: "text-red-400",
  reject: "text-red-400",
  monitor: "text-yellow-400",
  log: "text-yellow-400",
};

const severityVariants: Record<string, "critical" | "high" | "medium" | "low"> = {
  critical: "critical",
  high: "high",
  medium: "medium",
  low: "low",
};

export default function NetworkPage() {
  const summary = useApiData<NetworkSummary>(() => api.network.summary());
  const events = useApiList<NetworkEvent>(() => api.network.events());

  const stats = [
    {
      label: "Traffic In",
      value: formatBytes(summary.data?.bytes_in_24h),
      sub: "Last 24h",
    },
    {
      label: "Traffic Out",
      value: formatBytes(summary.data?.bytes_out_24h),
      sub: "Last 24h",
    },
    {
      label: "Blocked",
      value: formatNumber(summary.data?.blocked_24h),
      sub: `${formatNumber(summary.data?.threats_24h)} flagged as threats`,
    },
    {
      label: "Firewall Rules",
      value: formatNumber(summary.data?.active_rules),
      sub: `${formatNumber(summary.data?.port_scans_24h)} port scans seen`,
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Network Security</h2>
        <p className="text-muted-foreground">
          Network traffic monitoring and firewall management
        </p>
      </div>

      <DataState
        loading={summary.loading}
        error={summary.error}
        onRetry={summary.refetch}
        loadingLabel="Loading network summary"
      >
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {stats.map((stat) => (
            <Card key={stat.label} className="border-border">
              <CardContent className="p-4">
                <p className="text-sm text-muted-foreground">{stat.label}</p>
                <p className="text-2xl font-bold text-foreground">{stat.value}</p>
                <p className="text-xs text-muted-foreground">{stat.sub}</p>
              </CardContent>
            </Card>
          ))}
        </div>
      </DataState>

      <Card className="border-border">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <ArrowUpDown className="h-5 w-5 text-primary" aria-hidden="true" />
            Recent Network Events
          </CardTitle>
        </CardHeader>
        <CardContent>
          <DataState
            loading={events.loading}
            error={events.error}
            isEmpty={events.items.length === 0}
            onRetry={events.refetch}
            loadingLabel="Loading network events"
            emptyTitle="No network events recorded"
            emptyDescription="Traffic events appear here once agents start reporting."
          >
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <caption className="sr-only">
                  Recent network traffic events with source, destination and action
                </caption>
                <thead>
                  <tr className="border-b border-border text-left">
                    <th scope="col" className="pb-2 font-medium text-muted-foreground">Source</th>
                    <th scope="col" className="pb-2 font-medium text-muted-foreground">Destination</th>
                    <th scope="col" className="pb-2 font-medium text-muted-foreground">Protocol</th>
                    <th scope="col" className="pb-2 font-medium text-muted-foreground">Action</th>
                    <th scope="col" className="pb-2 font-medium text-muted-foreground">Severity</th>
                    <th scope="col" className="pb-2 font-medium text-muted-foreground">Bytes</th>
                    <th scope="col" className="pb-2 font-medium text-muted-foreground">Time</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {events.items.map((event) => (
                    <tr key={event.id} className="hover:bg-accent/50">
                      <td className="py-2 font-mono text-xs">
                        {event.source_ip ?? "—"}
                        {event.source_port ? `:${event.source_port}` : ""}
                      </td>
                      <td className="py-2 font-mono text-xs">
                        {event.dest_ip ?? "—"}
                        {event.dest_port ? `:${event.dest_port}` : ""}
                      </td>
                      <td className="py-2 text-xs uppercase">{event.protocol ?? "—"}</td>
                      <td
                        className={`py-2 text-xs font-medium capitalize ${
                          actionColors[event.action ?? ""] ?? "text-muted-foreground"
                        }`}
                      >
                        {event.action ?? "—"}
                      </td>
                      <td className="py-2">
                        {severityVariants[event.severity] ? (
                          <Badge variant={severityVariants[event.severity]}>
                            {event.severity}
                          </Badge>
                        ) : (
                          <span className="text-xs text-muted-foreground">
                            {event.severity}
                          </span>
                        )}
                      </td>
                      <td className="py-2 text-xs text-muted-foreground">
                        {formatBytes(
                          (event.bytes_sent ?? 0) + (event.bytes_received ?? 0)
                        )}
                      </td>
                      <td className="py-2 text-xs text-muted-foreground">
                        {new Date(event.occurred_at).toLocaleTimeString()}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </DataState>
        </CardContent>
      </Card>
    </div>
  );
}
