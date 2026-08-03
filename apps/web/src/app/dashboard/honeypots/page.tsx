"use client";

import {
  Bug, Wifi, Server, Mail, Database, Activity, Users, AlertTriangle, Clock, RefreshCw, Globe,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DataState } from "@/components/ui/data-state";
import { useApiData, useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { formatNumber, relativeTime } from "@/lib/utils";

/** Mirrors the portal's `HoneypotResponse`. */
interface Honeypot {
  id: string;
  name: string;
  description: string | null;
  honeypot_type: string;
  status: string;
  listen_ip: string | null;
  listen_port: number;
  server_id: string | null;
  emulated_banner: string | null;
  interaction_count: number;
  unique_attackers: number;
  last_interaction_at: string | null;
  created_at: string;
}

/** Mirrors the portal's `HoneypotSummary`. */
interface HoneypotSummary {
  total: number;
  running: number;
  stopped: number;
  interactions_24h: number;
  unique_attackers_24h: number;
  exploit_attempts_24h: number;
  critical_interactions_24h: number;
}

/**
 * Mirrors the portal's `InteractionResponse`. The portal deliberately omits the
 * attempted password, so only the username is available here.
 */
interface Interaction {
  id: string;
  honeypot_id: string;
  source_ip: string;
  source_port: number | null;
  country_code: string | null;
  asn: string | null;
  interaction_type: string;
  username_tried: string | null;
  severity: string;
  occurred_at: string;
}

/** Mirrors the portal's `TopAttacker`. */
interface TopAttacker {
  source_ip: string;
  country_code: string | null;
  interaction_count: number;
  exploit_attempts: number;
  last_seen: string | null;
}

/** Icon per honeypot_type; the API sends free-form lowercase protocol names. */
const typeIcons: Record<string, typeof Wifi> = {
  ssh: Wifi,
  http: Server,
  https: Server,
  smtp: Mail,
  mysql: Database,
  postgres: Database,
  redis: Database,
};

const severityVariants: Record<string, "critical" | "high" | "medium" | "low"> = {
  critical: "critical",
  high: "high",
  medium: "medium",
  low: "low",
};

export default function HoneypotsPage() {
  const summary = useApiData<HoneypotSummary>(() => api.honeypots.summary());
  const honeypots = useApiData<Honeypot[]>(() => api.honeypots.list());
  const interactions = useApiList<Interaction>(() =>
    api.honeypots.interactions()
  );
  const attackers = useApiData<TopAttacker[]>(() => api.honeypots.topAttackers());

  const pots = honeypots.data ?? [];
  const topAttackers = attackers.data ?? [];

  // Interactions reference their honeypot by id, so map ids to names for display.
  const potNames = new Map(pots.map((hp) => [hp.id, hp.name]));

  const stats = [
    {
      label: "Active Honeypots",
      value: formatNumber(summary.data?.running),
      icon: Bug,
      color: "text-green-400",
    },
    {
      label: "Interactions (24h)",
      value: formatNumber(summary.data?.interactions_24h),
      icon: Activity,
      color: "text-blue-400",
    },
    {
      label: "Unique Attackers (24h)",
      value: formatNumber(summary.data?.unique_attackers_24h),
      icon: Users,
      color: "text-purple-400",
    },
    {
      label: "Critical Interactions (24h)",
      value: formatNumber(summary.data?.critical_interactions_24h),
      icon: AlertTriangle,
      color: "text-red-400",
    },
  ];

  const refreshAll = () => {
    summary.refetch();
    honeypots.refetch();
    interactions.refetch();
    attackers.refetch();
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Honeypots</h2>
          <p className="text-muted-foreground">Deploy and monitor deception technology</p>
        </div>
        <Button onClick={refreshAll} variant="outline" className="gap-2">
          <RefreshCw className="h-4 w-4" aria-hidden="true" /> Refresh
        </Button>
      </div>

      <DataState
        loading={summary.loading}
        error={summary.error}
        onRetry={summary.refetch}
        loadingLabel="Loading honeypot summary"
      >
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {stats.map((stat) => (
            <Card key={stat.label} className="border-border">
              <CardContent className="p-4">
                <div className="flex items-center gap-3">
                  <stat.icon className={`h-8 w-8 ${stat.color}`} aria-hidden="true" />
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


      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="text-lg">Honeypot Inventory</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <DataState
            loading={honeypots.loading}
            error={honeypots.error}
            isEmpty={pots.length === 0}
            onRetry={honeypots.refetch}
            loadingLabel="Loading honeypots"
            emptyTitle="No honeypots deployed"
            emptyDescription="Deployed decoys appear here once they register with the portal."
          >
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <caption className="sr-only">
                  Deployed honeypots with type, listening port, status and interaction totals
                </caption>
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Name</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Type</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Port</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Interactions</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Last Seen</th>
                  </tr>
                </thead>
                <tbody>
                  {pots.map((hp) => {
                    const Icon = typeIcons[hp.honeypot_type.toLowerCase()] ?? Bug;
                    const running = hp.status === "running";
                    return (
                      <tr key={hp.id} className="border-b border-border hover:bg-muted/20">
                        <td className="px-4 py-3 font-medium text-foreground">{hp.name}</td>
                        <td className="px-4 py-3"><span className="flex items-center gap-1.5 uppercase text-muted-foreground"><Icon className="h-4 w-4" aria-hidden="true" />{hp.honeypot_type}</span></td>
                        <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{hp.listen_port}</td>
                        <td className="px-4 py-3">
                          <span className={`inline-flex items-center gap-1 text-xs font-medium capitalize ${running ? "text-green-400" : "text-muted-foreground"}`}>
                            <span className={`h-2 w-2 rounded-full ${running ? "bg-green-400" : "bg-muted-foreground"}`} aria-hidden="true" />{hp.status}
                          </span>
                        </td>
                        <td className="px-4 py-3 text-foreground">{formatNumber(hp.interaction_count)}</td>
                        <td className="px-4 py-3 text-xs text-muted-foreground">{relativeTime(hp.last_interaction_at)}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </DataState>
        </CardContent>
      </Card>


      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Clock className="h-5 w-5 text-blue-400" aria-hidden="true" />
            Recent Interactions
          </CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <DataState
            loading={interactions.loading}
            error={interactions.error}
            isEmpty={interactions.items.length === 0}
            onRetry={interactions.refetch}
            loadingLabel="Loading interactions"
            emptyTitle="No interactions captured"
            emptyDescription="Attacker activity against your decoys appears here."
          >
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <caption className="sr-only">
                  Recent honeypot interactions with source, type, attempted username and severity
                </caption>
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Time</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Source IP</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Honeypot</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Action</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Username Tried</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Severity</th>
                  </tr>
                </thead>
                <tbody>
                  {interactions.items.map((ix) => (
                    <tr key={ix.id} className="border-b border-border hover:bg-muted/20">
                      <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{relativeTime(ix.occurred_at)}</td>
                      <td className="px-4 py-3 font-mono text-xs text-foreground">
                        {ix.source_ip}
                        {ix.country_code ? ` (${ix.country_code})` : ""}
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">{potNames.get(ix.honeypot_id) ?? "—"}</td>
                      <td className="px-4 py-3 text-muted-foreground">{ix.interaction_type.replace(/_/g, " ")}</td>
                      <td className="px-4 py-3 font-mono text-xs text-foreground">{ix.username_tried ?? "—"}</td>
                      <td className="px-4 py-3">
                        {severityVariants[ix.severity] ? (
                          <Badge variant={severityVariants[ix.severity]}>{ix.severity}</Badge>
                        ) : (
                          <span className="text-xs text-muted-foreground">{ix.severity}</span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </DataState>
        </CardContent>
      </Card>


      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Globe className="h-5 w-5 text-purple-400" aria-hidden="true" />
            Top Attackers
          </CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <DataState
            loading={attackers.loading}
            error={attackers.error}
            isEmpty={topAttackers.length === 0}
            onRetry={attackers.refetch}
            loadingLabel="Loading top attackers"
            emptyTitle="No attackers recorded"
            emptyDescription="Sources appear here once they interact with a honeypot."
          >
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <caption className="sr-only">
                  Most active attacker source IPs with interaction and exploit attempt counts
                </caption>
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Source IP</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Country</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Interactions</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Exploit Attempts</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Last Seen</th>
                  </tr>
                </thead>
                <tbody>
                  {topAttackers.map((attacker) => (
                    <tr key={attacker.source_ip} className="border-b border-border hover:bg-muted/20">
                      <td className="px-4 py-3 font-mono text-xs text-foreground">{attacker.source_ip}</td>
                      <td className="px-4 py-3">
                        <Badge variant="outline" className="text-xs">{attacker.country_code ?? "unknown"}</Badge>
                      </td>
                      <td className="px-4 py-3 text-foreground">{formatNumber(attacker.interaction_count)}</td>
                      <td className="px-4 py-3">
                        <span className={attacker.exploit_attempts > 0 ? "font-semibold text-red-400" : "text-muted-foreground"}>
                          {formatNumber(attacker.exploit_attempts)}
                        </span>
                      </td>
                      <td className="px-4 py-3 text-xs text-muted-foreground">{relativeTime(attacker.last_seen)}</td>
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

