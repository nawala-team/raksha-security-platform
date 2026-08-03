"use client";

import { useMemo, useState } from "react";
import {
  Bug, Search, Filter, CheckCircle2, XCircle, AlertTriangle, ShieldCheck,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { DataState } from "@/components/ui/data-state";
import { useApiData, useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { formatNumber, relativeTime } from "@/lib/utils";

/** Mirrors the portal's `ScanResponse`. */
interface Scan {
  id: string;
  agent_id: string;
  scan_type: string;
  scanner: string;
  status: string;
  total_packages: number | null;
  total_vulns: number | null;
  critical_count: number | null;
  high_count: number | null;
  medium_count: number | null;
  low_count: number | null;
  info_count: number | null;
  fixable_count: number | null;
  duration_secs: number | null;
  error_message: string | null;
  started_at: string;
  completed_at: string | null;
}

/** Mirrors the portal's `VulnSummary`. */
interface VulnSummary {
  total_scans: number;
  completed_scans: number;
  failed_scans: number;
  running_scans: number;
  critical_vulns: number;
  high_vulns: number;
  medium_vulns: number;
  low_vulns: number;
  fixable_vulns: number;
  agents_scanned: number;
}

function getStatusIcon(status: string) {
  switch (status) {
    case "completed": return <CheckCircle2 className="h-4 w-4 text-green-400" aria-hidden="true" />;
    case "failed": return <XCircle className="h-4 w-4 text-red-400" aria-hidden="true" />;
    case "running": return <AlertTriangle className="h-4 w-4 text-yellow-400" aria-hidden="true" />;
    default: return <ShieldCheck className="h-4 w-4 text-muted-foreground" aria-hidden="true" />;
  }
}

/** Severity count cell: zero counts stay muted so hot rows stand out. */
function SeverityCell({ count, className }: { count: number | null; className: string }) {
  if (count === null || count === 0) {
    return <span className="text-muted-foreground">{count === null ? "—" : 0}</span>;
  }
  return <span className={className}>{formatNumber(count)}</span>;
}

export default function VulnerabilitiesPage() {
  const summary = useApiData<VulnSummary>(() => api.vulnerabilities.summary());
  const scans = useApiList<Scan>(() => api.vulnerabilities.scans());

  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [scannerFilter, setScannerFilter] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState("");

  const scanners = useMemo(
    () => [...new Set(scans.items.map((s) => s.scanner))],
    [scans.items]
  );

  const filtered = useMemo(() => {
    return scans.items.filter((scan) => {
      if (statusFilter !== "all" && scan.status !== statusFilter) return false;
      if (scannerFilter !== "all" && scan.scanner !== scannerFilter) return false;
      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        if (
          !scan.agent_id.toLowerCase().includes(q) &&
          !scan.scan_type.toLowerCase().includes(q) &&
          !scan.scanner.toLowerCase().includes(q)
        ) {
          return false;
        }
      }
      return true;
    });
  }, [scans.items, statusFilter, scannerFilter, searchQuery]);

  // Severity totals come from the summary: it counts the latest completed scan
  // per agent, so hosts scanned repeatedly are not double counted.
  const critical = summary.data?.critical_vulns ?? 0;
  const high = summary.data?.high_vulns ?? 0;
  const medium = summary.data?.medium_vulns ?? 0;
  const low = summary.data?.low_vulns ?? 0;
  const severityTotal = critical + high + medium + low;

  const filtersActive =
    statusFilter !== "all" || scannerFilter !== "all" || Boolean(searchQuery);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground flex items-center gap-2">
          <Bug className="h-6 w-6 text-primary" aria-hidden="true" />
          Vulnerability Dashboard
        </h2>
        <p className="text-muted-foreground">CVE tracking and vulnerability management across all servers</p>
      </div>

      {/* Summary Stats */}
      <DataState
        loading={summary.loading}
        error={summary.error}
        onRetry={summary.refetch}
        loadingLabel="Loading vulnerability summary"
      >
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6">
          <Card className="border-border"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-foreground">{formatNumber(severityTotal)}</p><p className="text-xs text-muted-foreground">Total CVEs</p></CardContent></Card>
          <Card className="border-red-500/30 bg-red-500/5"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-red-400">{formatNumber(critical)}</p><p className="text-xs text-red-400/80">Critical</p></CardContent></Card>
          <Card className="border-orange-500/30 bg-orange-500/5"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-orange-400">{formatNumber(high)}</p><p className="text-xs text-orange-400/80">High</p></CardContent></Card>
          <Card className="border-yellow-500/30 bg-yellow-500/5"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-yellow-400">{formatNumber(medium)}</p><p className="text-xs text-yellow-400/80">Medium</p></CardContent></Card>
          <Card className="border-green-500/30 bg-green-500/5"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-green-400">{formatNumber(low)}</p><p className="text-xs text-green-400/80">Low</p></CardContent></Card>
          <Card className="border-blue-500/30 bg-blue-500/5"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-blue-400">{formatNumber(summary.data?.fixable_vulns)}</p><p className="text-xs text-blue-400/80">Fixable</p></CardContent></Card>
        </div>
      </DataState>

      {/* Severity Breakdown Bar */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium text-muted-foreground">Severity Breakdown</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex h-4 w-full overflow-hidden rounded-full bg-muted/30">
            {critical > 0 && <div className="bg-red-500" style={{ width: `${(critical / severityTotal) * 100}%` }} title={`Critical: ${critical}`} role="img" aria-label={`Critical: ${critical}`} />}
            {high > 0 && <div className="bg-orange-500" style={{ width: `${(high / severityTotal) * 100}%` }} title={`High: ${high}`} role="img" aria-label={`High: ${high}`} />}
            {medium > 0 && <div className="bg-yellow-500" style={{ width: `${(medium / severityTotal) * 100}%` }} title={`Medium: ${medium}`} role="img" aria-label={`Medium: ${medium}`} />}
            {low > 0 && <div className="bg-green-500" style={{ width: `${(low / severityTotal) * 100}%` }} title={`Low: ${low}`} role="img" aria-label={`Low: ${low}`} />}
          </div>
          <div className="mt-2 flex flex-wrap gap-4 text-xs text-muted-foreground">
            <span className="flex items-center gap-1"><span className="h-2.5 w-2.5 rounded-full bg-red-500" />Critical ({formatNumber(critical)})</span>
            <span className="flex items-center gap-1"><span className="h-2.5 w-2.5 rounded-full bg-orange-500" />High ({formatNumber(high)})</span>
            <span className="flex items-center gap-1"><span className="h-2.5 w-2.5 rounded-full bg-yellow-500" />Medium ({formatNumber(medium)})</span>
            <span className="flex items-center gap-1"><span className="h-2.5 w-2.5 rounded-full bg-green-500" />Low ({formatNumber(low)})</span>
            <span>Across {formatNumber(summary.data?.agents_scanned)} scanned agents</span>
          </div>
        </CardContent>
      </Card>


      {/* Filters */}
      <Card className="border-border">
        <CardContent className="p-4">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex items-center gap-2">
              <Filter className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
              <span className="text-sm font-medium text-muted-foreground">Filters:</span>
            </div>
            <div className="relative">
              <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" aria-hidden="true" />
              <Input placeholder="Search agent, scanner or type..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} className="h-9 w-56 pl-8" aria-label="Search scans by agent, scanner or scan type" />
            </div>
            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger className="h-9 w-36" aria-label="Filter by status"><SelectValue placeholder="Status" /></SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Status</SelectItem>
                <SelectItem value="completed">Completed</SelectItem>
                <SelectItem value="running">Running</SelectItem>
                <SelectItem value="failed">Failed</SelectItem>
              </SelectContent>
            </Select>
            <Select value={scannerFilter} onValueChange={setScannerFilter}>
              <SelectTrigger className="h-9 w-40" aria-label="Filter by scanner"><SelectValue placeholder="Scanner" /></SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Scanners</SelectItem>
                {scanners.map((s) => (<SelectItem key={s} value={s}>{s}</SelectItem>))}
              </SelectContent>
            </Select>
            {filtersActive && (
              <Button variant="ghost" size="sm" onClick={() => { setStatusFilter("all"); setScannerFilter("all"); setSearchQuery(""); }}>Clear filters</Button>
            )}
            <span className="ml-auto text-xs text-muted-foreground">
              {formatNumber(summary.data?.running_scans)} running • {formatNumber(summary.data?.failed_scans)} failed
            </span>
          </div>
        </CardContent>
      </Card>


      {/* Scan Table */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg">Scan History</CardTitle>
            <span className="text-xs text-muted-foreground">{formatNumber(scans.total)} scans</span>
          </div>
        </CardHeader>
        <CardContent className="p-0">
          <DataState
            loading={scans.loading}
            error={scans.error}
            isEmpty={scans.items.length === 0}
            onRetry={scans.refetch}
            loadingLabel="Loading scans"
            emptyTitle="No scans yet"
            emptyDescription="Vulnerability scans reported by agents appear here."
          >
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <caption className="sr-only">
                  Vulnerability scan history per agent with severity counts, fixable count and status.
                </caption>
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Agent</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Scanner</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Packages</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Total</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Critical</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">High</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Medium</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Low</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Fixable</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Started</th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.length === 0 && (
                    <tr><td colSpan={11} className="px-4 py-12 text-center text-muted-foreground">No scans match the current filters.</td></tr>
                  )}
                  {filtered.map((scan) => (
                    <tr key={scan.id} className="border-b border-border transition-colors hover:bg-muted/20">
                      <td className="px-4 py-3">
                        <p className="font-mono text-xs text-foreground">{scan.agent_id}</p>
                        <p className="text-xs text-muted-foreground">{scan.scan_type.replace(/_/g, " ")}</p>
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">{scan.scanner}</td>
                      <td className="px-4 py-3 text-muted-foreground">{formatNumber(scan.total_packages)}</td>
                      <td className="px-4 py-3 font-medium text-foreground">{formatNumber(scan.total_vulns)}</td>
                      <td className="px-4 py-3"><SeverityCell count={scan.critical_count} className="font-bold text-red-400" /></td>
                      <td className="px-4 py-3"><SeverityCell count={scan.high_count} className="font-semibold text-orange-400" /></td>
                      <td className="px-4 py-3"><SeverityCell count={scan.medium_count} className="text-yellow-400" /></td>
                      <td className="px-4 py-3"><SeverityCell count={scan.low_count} className="text-green-400" /></td>
                      <td className="px-4 py-3">
                        {scan.fixable_count === null || scan.fixable_count === 0 ? (
                          <span className="text-muted-foreground">{scan.fixable_count === null ? "—" : 0}</span>
                        ) : (
                          <Badge variant="outline" className="text-xs">{formatNumber(scan.fixable_count)}</Badge>
                        )}
                      </td>
                      <td className="px-4 py-3">
                        <span className="flex items-center gap-1.5 capitalize">
                          {getStatusIcon(scan.status)}{scan.status}
                        </span>
                        {scan.error_message && (
                          <p className="font-mono text-xs text-red-400">{scan.error_message}</p>
                        )}
                      </td>
                      <td className="px-4 py-3 text-xs text-muted-foreground">
                        {relativeTime(scan.started_at)}
                        {scan.duration_secs !== null && (
                          <p className="text-xs text-muted-foreground">{formatNumber(scan.duration_secs)}s</p>
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

      <p className="text-xs text-muted-foreground">
        Showing {formatNumber(filtered.length)} of {formatNumber(scans.total)} scans
      </p>
    </div>
  );
}

