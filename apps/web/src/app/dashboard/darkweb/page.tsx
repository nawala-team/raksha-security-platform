"use client";

import {
  Globe, Search, RefreshCw, Eye, ShieldAlert, AlertTriangle,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DataState } from "@/components/ui/data-state";
import { useApiData, useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { formatNumber, relativeTime } from "@/lib/utils";

/** Mirrors the portal's `MonitorResponse`. */
interface Monitor {
  id: string;
  name: string;
  monitor_type: string;
  keyword: string;
  is_enabled: boolean;
  severity_floor: string;
  finding_count: number;
  new_finding_count: number;
  last_scanned_at: string | null;
  next_scan_at: string | null;
  scan_interval_mins: number;
  created_at: string;
}

/**
 * Mirrors the portal's `FindingResponse`. Only `excerpt_redacted` is available:
 * the portal redacts excerpts at ingest and never returns raw leaked secrets.
 */
interface Finding {
  id: string;
  monitor_id: string;
  title: string;
  description: string | null;
  finding_type: string;
  severity: string;
  status: string;
  source_name: string | null;
  source_type: string | null;
  excerpt_redacted: string | null;
  record_count: number | null;
  confidence: number | null;
  alert_id: string | null;
  incident_id: string | null;
  discovered_at: string;
}

/** Mirrors the portal's `DarkwebSummary`. */
interface DarkwebSummary {
  active_monitors: number;
  total_findings: number;
  new_findings: number;
  critical_findings: number;
  credential_leaks: number;
  exposed_records: number;
}

const severityVariants: Record<string, "critical" | "high" | "medium" | "low"> = {
  critical: "critical",
  high: "high",
  medium: "medium",
  low: "low",
};

export default function DarkWebPage() {
  const summary = useApiData<DarkwebSummary>(() => api.darkweb.summary());
  const monitors = useApiData<Monitor[]>(() => api.darkweb.monitors());
  const findings = useApiList<Finding>(() => api.darkweb.findings());

  const watchlist = monitors.data ?? [];

  // Findings reference their monitor by id; map ids to names for display.
  const monitorNames = new Map(watchlist.map((m) => [m.id, m.name]));

  // The summary has no "last scan" field, so derive it from the monitors.
  const lastScanned = watchlist
    .map((m) => m.last_scanned_at)
    .filter((v): v is string => Boolean(v))
    .sort()
    .pop();

  const stats = [
    {
      label: "Active Monitors",
      value: formatNumber(summary.data?.active_monitors),
      icon: Globe,
      color: "text-blue-400",
    },
    {
      label: "Findings",
      value: formatNumber(summary.data?.total_findings),
      icon: ShieldAlert,
      color: "text-red-400",
    },
    {
      label: "Credential Leaks",
      value: formatNumber(summary.data?.credential_leaks),
      icon: Eye,
      color: "text-orange-400",
    },
    {
      label: "Last Scan",
      value: relativeTime(lastScanned),
      icon: Search,
      color: "text-green-400",
    },
  ];

  const refreshAll = () => {
    summary.refetch();
    monitors.refetch();
    findings.refetch();
  };

  const scanning = summary.loading || monitors.loading || findings.loading;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Dark Web Monitoring</h2>
          <p className="text-muted-foreground">Monitor for leaked credentials and data exposure</p>
        </div>
        <Button onClick={refreshAll} disabled={scanning} className="gap-2">
          <RefreshCw className={`h-4 w-4 ${scanning ? "animate-spin" : ""}`} aria-hidden="true" />
          {scanning ? "Refreshing..." : "Refresh"}
        </Button>
      </div>

      <DataState
        loading={summary.loading}
        error={summary.error}
        onRetry={summary.refetch}
        loadingLabel="Loading dark web summary"
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
          <CardTitle className="flex items-center gap-2 text-lg">
            <AlertTriangle className="h-5 w-5 text-red-400" aria-hidden="true" />
            Discovered Findings
          </CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <DataState
            loading={findings.loading}
            error={findings.error}
            isEmpty={findings.items.length === 0}
            onRetry={findings.refetch}
            loadingLabel="Loading findings"
            emptyTitle="No findings yet"
            emptyDescription="Matches for your monitored keywords appear here."
          >
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <caption className="sr-only">
                  Dark web findings with source, redacted excerpt and severity. Excerpts are
                  redacted by the portal; raw leaked credentials are never shown.
                </caption>
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Finding</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Monitor</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Source</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Discovered</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Redacted Excerpt</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Severity</th>
                  </tr>
                </thead>
                <tbody>
                  {findings.items.map((finding) => (
                    <tr key={finding.id} className="border-b border-border transition-colors hover:bg-muted/20">
                      <td className="px-4 py-3">
                        <p className="text-foreground">{finding.title}</p>
                        <p className="text-xs text-muted-foreground">
                          {finding.finding_type.replace(/_/g, " ")}
                          {finding.record_count !== null
                            ? ` • ${formatNumber(finding.record_count)} records`
                            : ""}
                        </p>
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">
                        {monitorNames.get(finding.monitor_id) ?? "—"}
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">
                        {finding.source_name ?? "unknown source"}
                      </td>
                      <td className="px-4 py-3 text-xs text-muted-foreground">
                        {relativeTime(finding.discovered_at)}
                      </td>
                      <td className="px-4 py-3 font-mono text-xs text-muted-foreground">
                        {finding.excerpt_redacted ?? "—"}
                      </td>
                      <td className="px-4 py-3">
                        {severityVariants[finding.severity] ? (
                          <Badge variant={severityVariants[finding.severity]}>{finding.severity}</Badge>
                        ) : (
                          <span className="text-xs text-muted-foreground">{finding.severity}</span>
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
            <Search className="h-5 w-5 text-blue-400" aria-hidden="true" />
            Watchlist
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <DataState
            loading={monitors.loading}
            error={monitors.error}
            isEmpty={watchlist.length === 0}
            onRetry={monitors.refetch}
            loadingLabel="Loading monitors"
            emptyTitle="No monitors configured"
            emptyDescription="Keywords, domains and emails being watched appear here."
          >
            <div className="space-y-2">
              {watchlist.map((monitor) => (
                <div key={monitor.id} className="flex items-center justify-between rounded-lg border border-border px-4 py-2">
                  <div className="flex items-center gap-3">
                    <Badge variant="outline" className="text-xs">{monitor.monitor_type}</Badge>
                    <span className="font-mono text-sm text-foreground">{monitor.keyword}</span>
                    {!monitor.is_enabled && (
                      <Badge variant="secondary" className="text-xs">paused</Badge>
                    )}
                    {monitor.new_finding_count > 0 && (
                      <Badge variant="high" className="text-xs">
                        {formatNumber(monitor.new_finding_count)} new
                      </Badge>
                    )}
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="text-xs text-muted-foreground">
                      {formatNumber(monitor.finding_count)} findings
                    </span>
                    <span className="text-xs text-muted-foreground">
                      Checked: {relativeTime(monitor.last_scanned_at)}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </DataState>
        </CardContent>
      </Card>
    </div>
  );
}

