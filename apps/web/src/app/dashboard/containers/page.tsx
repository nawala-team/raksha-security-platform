"use client";

import { useState } from "react";
import {
  Box,
  Shield,
  AlertTriangle,
  Search,
  Activity,
  Layers,
  Lock,
  RefreshCw,
} from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { DataState } from "@/components/ui/data-state";
import { useApiData, useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { formatNumber, relativeTime } from "@/lib/utils";

/** Mirrors the portal's `ContainerResponse`. */
interface ContainerRow {
  id: string;
  agent_id: string | null;
  server_id: string | null;
  container_id: string;
  name: string;
  image: string;
  image_tag: string | null;
  runtime: string;
  orchestrator: string | null;
  namespace: string | null;
  pod_name: string | null;
  status: string;
  privileged: boolean;
  root_user: boolean;
  host_network: boolean;
  cpu_usage_pct: number | null;
  memory_mb: number | null;
  critical_vulns: number;
  high_vulns: number;
  medium_vulns: number;
  low_vulns: number;
  compliance_score: number | null;
  started_at: string | null;
  last_scanned_at: string | null;
}

/** Mirrors the portal's `ContainerSummary`. */
interface ContainerSummary {
  total: number;
  running: number;
  stopped: number;
  privileged: number;
  running_as_root: number;
  host_network: number;
  critical_vulns: number;
  high_vulns: number;
}

/** Mirrors the portal's `ImageScanResponse`. */
interface ImageScanRow {
  id: string;
  image: string;
  image_digest: string | null;
  scanner: string;
  status: string;
  critical_count: number;
  high_count: number;
  medium_count: number;
  low_count: number;
  fixable_count: number;
  secrets_found: number;
  misconfigs: number;
  duration_secs: number | null;
  error_message: string | null;
  started_at: string;
  completed_at: string | null;
}

/** Per-container CVE total; the API reports counts per severity, not a sum. */
function containerVulns(container: ContainerRow): number {
  return (
    container.critical_vulns +
    container.high_vulns +
    container.medium_vulns +
    container.low_vulns
  );
}

function scanVulns(scan: ImageScanRow): number {
  return (
    scan.critical_count + scan.high_count + scan.medium_count + scan.low_count
  );
}

/** Badge severity bucket for a CVE count. */
function vulnVariant(count: number): "critical" | "high" | "medium" | "low" {
  if (count >= 10) return "critical";
  if (count >= 5) return "high";
  if (count > 0) return "medium";
  return "low";
}

export default function ContainersPage() {
  const [searchQuery, setSearchQuery] = useState("");

  const summary = useApiData<ContainerSummary>(() => api.containers.summary());
  const containers = useApiList<ContainerRow>(() => api.containers.list());
  const scans = useApiList<ImageScanRow>(() => api.containers.scans());

  const query = searchQuery.trim().toLowerCase();

  const filteredContainers = containers.items.filter(
    (c) =>
      !query ||
      c.name.toLowerCase().includes(query) ||
      c.image.toLowerCase().includes(query)
  );

  const filteredScans = scans.items.filter(
    (s) =>
      !query ||
      s.image.toLowerCase().includes(query) ||
      s.scanner.toLowerCase().includes(query)
  );

  const refreshAll = () => {
    summary.refetch();
    containers.refetch();
    scans.refetch();
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Container Security</h2>
          <p className="text-muted-foreground">Monitor container workloads and image vulnerabilities</p>
        </div>
        <Button variant="outline" onClick={refreshAll}>
          <RefreshCw className="mr-2 h-4 w-4" aria-hidden="true" />
          Refresh
        </Button>
      </div>

      {/* Stats */}
      <DataState
        loading={summary.loading}
        error={summary.error}
        onRetry={summary.refetch}
        loadingLabel="Loading container summary"
      >
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-5">
          <Card className="border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-green-500/10 p-2"><Box className="h-5 w-5 text-green-400" aria-hidden="true" /></div>
                <div><p className="text-2xl font-bold text-foreground">{formatNumber(summary.data?.running)}</p><p className="text-xs text-muted-foreground">Running Containers</p></div>
              </div>
            </CardContent>
          </Card>
          <Card className="border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-blue-500/10 p-2"><Shield className="h-5 w-5 text-blue-400" aria-hidden="true" /></div>
                <div><p className="text-2xl font-bold text-foreground">{formatNumber(scans.total)}</p><p className="text-xs text-muted-foreground">Images Scanned</p></div>
              </div>
            </CardContent>
          </Card>
          <Card className="border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-red-500/10 p-2"><AlertTriangle className="h-5 w-5 text-red-400" aria-hidden="true" /></div>
                <div><p className="text-2xl font-bold text-foreground">{formatNumber((summary.data?.critical_vulns ?? 0) + (summary.data?.high_vulns ?? 0))}</p><p className="text-xs text-muted-foreground">Critical + High Vulns</p></div>
              </div>
            </CardContent>
          </Card>
          <Card className="border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-orange-500/10 p-2"><Lock className="h-5 w-5 text-orange-400" aria-hidden="true" /></div>
                <div><p className="text-2xl font-bold text-foreground">{formatNumber(summary.data?.privileged)}</p><p className="text-xs text-muted-foreground">Privileged Containers</p></div>
              </div>
            </CardContent>
          </Card>
          <Card className="border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-purple-500/10 p-2"><Layers className="h-5 w-5 text-purple-400" aria-hidden="true" /></div>
                <div><p className="text-2xl font-bold text-foreground">{formatNumber(summary.data?.running_as_root)}</p><p className="text-xs text-muted-foreground">Running As Root</p></div>
              </div>
            </CardContent>
          </Card>
        </div>
      </DataState>

      {/* Tabs */}
      <Tabs defaultValue="inventory" className="space-y-4">
        <div className="flex flex-wrap items-center gap-4">
          <TabsList>
            <TabsTrigger value="inventory">Container Inventory</TabsTrigger>
            <TabsTrigger value="images">Image Scans</TabsTrigger>
          </TabsList>
          <div className="relative min-w-[200px]">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
            <Input placeholder="Search..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} className="pl-9" />
          </div>
        </div>


        {/* Container Inventory Tab */}
        <TabsContent value="inventory">
          <Card className="border-border">
            <CardContent className="p-4">
              <DataState
                loading={containers.loading}
                error={containers.error}
                isEmpty={containers.items.length === 0}
                onRetry={containers.refetch}
                loadingLabel="Loading containers"
                emptyTitle="No containers reported"
                emptyDescription="Containers appear here once agents report their workloads."
              >
                <div className="overflow-x-auto">
                  <table className="w-full text-sm">
                    <caption className="sr-only">
                      Container inventory with image, status, vulnerability count and privilege level
                    </caption>
                    <thead>
                      <tr className="border-b border-border text-left">
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">Name</th>
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">Image</th>
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">Status</th>
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">Vulnerabilities</th>
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">Privileged</th>
                        <th scope="col" className="pb-3 font-medium text-muted-foreground">Started</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredContainers.map((container) => {
                        const vulns = containerVulns(container);
                        return (
                          <tr key={container.id} className="border-b border-border/50 hover:bg-muted/30 transition-colors">
                            <td className="py-3 pr-4 font-mono text-xs text-foreground">{container.name}</td>
                            <td className="py-3 pr-4 text-xs text-muted-foreground">
                              {container.image}
                              {container.image_tag ? `:${container.image_tag}` : ""}
                            </td>
                            <td className="py-3 pr-4">
                              <span className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-medium ${
                                container.status === "running" ? "bg-green-500/20 text-green-400 border-green-500/30" :
                                container.status === "errored" ? "bg-red-500/20 text-red-400 border-red-500/30" :
                                "bg-gray-500/20 text-gray-400 border-gray-500/30"
                              }`}>
                                <Activity className="h-3 w-3" aria-hidden="true" />{container.status}
                              </span>
                            </td>
                            <td className="py-3 pr-4">
                              {vulns > 0 ? (
                                <Badge variant={vulnVariant(vulns)}>{vulns} CVEs</Badge>
                              ) : (
                                <Badge variant="low">Clean</Badge>
                              )}
                            </td>
                            <td className="py-3 pr-4">
                              {container.privileged && <span className="inline-flex items-center gap-1 text-xs text-orange-400"><Lock className="h-3 w-3" aria-hidden="true" />Yes</span>}
                              {!container.privileged && <span className="text-xs text-muted-foreground">No</span>}
                            </td>
                            <td className="py-3 text-xs text-muted-foreground">{relativeTime(container.started_at)}</td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              </DataState>
            </CardContent>
          </Card>
        </TabsContent>


        {/* Image Scans Tab */}
        <TabsContent value="images">
          <Card className="border-border">
            <CardContent className="p-4">
              <DataState
                loading={scans.loading}
                error={scans.error}
                isEmpty={scans.items.length === 0}
                onRetry={scans.refetch}
                loadingLabel="Loading image scans"
                emptyTitle="No image scans yet"
                emptyDescription="Scan results appear here once an image scanner reports in."
              >
                <div className="overflow-x-auto">
                  <table className="w-full text-sm">
                    <caption className="sr-only">
                      Container image scans with CVE counts broken down by severity
                    </caption>
                    <thead>
                      <tr className="border-b border-border text-left">
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">Image</th>
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">Scanner</th>
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">Total CVEs</th>
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">Critical</th>
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">High</th>
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">Medium</th>
                        <th scope="col" className="pb-3 pr-4 font-medium text-muted-foreground">Low</th>
                        <th scope="col" className="pb-3 font-medium text-muted-foreground">Scanned</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredScans.map((scan) => {
                        const total = scanVulns(scan);
                        return (
                          <tr key={scan.id} className="border-b border-border/50 hover:bg-muted/30 transition-colors">
                            <td className="py-3 pr-4 font-mono text-xs text-foreground">{scan.image}</td>
                            <td className="py-3 pr-4 text-xs text-muted-foreground">{scan.scanner}</td>
                            <td className="py-3 pr-4"><Badge variant={vulnVariant(total)}>{total}</Badge></td>
                            <td className="py-3 pr-4 text-xs"><span className="text-red-400 font-semibold">{scan.critical_count}</span></td>
                            <td className="py-3 pr-4 text-xs"><span className="text-orange-400 font-semibold">{scan.high_count}</span></td>
                            <td className="py-3 pr-4 text-xs"><span className="text-yellow-400 font-semibold">{scan.medium_count}</span></td>
                            <td className="py-3 pr-4 text-xs"><span className="text-green-400 font-semibold">{scan.low_count}</span></td>
                            <td className="py-3 text-xs text-muted-foreground">{relativeTime(scan.completed_at ?? scan.started_at)}</td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              </DataState>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}

