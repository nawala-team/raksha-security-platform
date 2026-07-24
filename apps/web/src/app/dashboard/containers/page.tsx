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

interface Container {
  id: string;
  name: string;
  image: string;
  status: "running" | "stopped" | "errored";
  vulnerabilities: number;
  privileged: boolean;
  started: string;
}

interface ImageScan {
  id: string;
  image: string;
  tag: string;
  totalCves: number;
  critical: number;
  high: number;
  medium: number;
  low: number;
  scannedAt: string;
}

interface RuntimeAlert {
  id: string;
  container: string;
  event: string;
  severity: "critical" | "high" | "medium" | "low";
  timestamp: string;
}

interface K8sPod {
  id: string;
  name: string;
  namespace: string;
  status: "Running" | "Pending" | "CrashLoopBackOff" | "Completed";
  restarts: number;
  age: string;
}

const mockContainers: Container[] = [
  { id: "c1", name: "web-frontend-7d8f9", image: "nginx:1.25", status: "running", vulnerabilities: 3, privileged: false, started: "2024-01-15T06:00:00Z" },
  { id: "c2", name: "api-gateway-4b2c1", image: "node:20-alpine", status: "running", vulnerabilities: 7, privileged: false, started: "2024-01-15T06:00:00Z" },
  { id: "c3", name: "redis-cache-a1b2c", image: "redis:7.2", status: "running", vulnerabilities: 1, privileged: false, started: "2024-01-14T22:00:00Z" },
  { id: "c4", name: "monitoring-agent-x9z", image: "datadog/agent:7", status: "running", vulnerabilities: 0, privileged: true, started: "2024-01-13T08:00:00Z" },
  { id: "c5", name: "postgres-primary-m3n", image: "postgres:16", status: "running", vulnerabilities: 2, privileged: false, started: "2024-01-10T12:00:00Z" },
  { id: "c6", name: "batch-worker-err-1", image: "app/worker:v2.1", status: "errored", vulnerabilities: 12, privileged: true, started: "2024-01-15T09:00:00Z" },
  { id: "c7", name: "log-collector-f4g5", image: "fluentd:v1.16", status: "running", vulnerabilities: 4, privileged: true, started: "2024-01-12T10:00:00Z" },
  { id: "c8", name: "auth-service-h6j7", image: "app/auth:v3.0", status: "stopped", vulnerabilities: 5, privileged: false, started: "2024-01-14T15:00:00Z" },
];

const mockImageScans: ImageScan[] = [
  { id: "i1", image: "nginx", tag: "1.25", totalCves: 3, critical: 0, high: 1, medium: 2, low: 0, scannedAt: "2024-01-15T06:00:00Z" },
  { id: "i2", image: "node", tag: "20-alpine", totalCves: 7, critical: 1, high: 2, medium: 3, low: 1, scannedAt: "2024-01-15T06:00:00Z" },
  { id: "i3", image: "redis", tag: "7.2", totalCves: 1, critical: 0, high: 0, medium: 1, low: 0, scannedAt: "2024-01-15T05:00:00Z" },
  { id: "i4", image: "datadog/agent", tag: "7", totalCves: 0, critical: 0, high: 0, medium: 0, low: 0, scannedAt: "2024-01-14T22:00:00Z" },
  { id: "i5", image: "postgres", tag: "16", totalCves: 2, critical: 0, high: 1, medium: 0, low: 1, scannedAt: "2024-01-15T04:00:00Z" },
  { id: "i6", image: "app/worker", tag: "v2.1", totalCves: 12, critical: 3, high: 5, medium: 3, low: 1, scannedAt: "2024-01-15T09:00:00Z" },
  { id: "i7", image: "fluentd", tag: "v1.16", totalCves: 4, critical: 0, high: 2, medium: 1, low: 1, scannedAt: "2024-01-14T20:00:00Z" },
  { id: "i8", image: "app/auth", tag: "v3.0", totalCves: 5, critical: 1, high: 1, medium: 2, low: 1, scannedAt: "2024-01-14T18:00:00Z" },
];

const mockRuntimeAlerts: RuntimeAlert[] = [
  { id: "r1", container: "batch-worker-err-1", event: "Unexpected process spawn: /bin/sh", severity: "critical", timestamp: "2024-01-15T09:05:00Z" },
  { id: "r2", container: "monitoring-agent-x9z", event: "Privileged container accessing host namespace", severity: "high", timestamp: "2024-01-15T08:30:00Z" },
  { id: "r3", container: "api-gateway-4b2c1", event: "Outbound connection to suspicious IP", severity: "high", timestamp: "2024-01-15T07:45:00Z" },
  { id: "r4", container: "log-collector-f4g5", event: "File write to /etc/passwd", severity: "critical", timestamp: "2024-01-15T07:20:00Z" },
  { id: "r5", container: "web-frontend-7d8f9", event: "Port scan detected from container", severity: "medium", timestamp: "2024-01-15T06:50:00Z" },
];

const mockK8sPods: K8sPod[] = [
  { id: "k1", name: "web-frontend-7d8f9-abc12", namespace: "production", status: "Running", restarts: 0, age: "2d" },
  { id: "k2", name: "api-gateway-4b2c1-def34", namespace: "production", status: "Running", restarts: 1, age: "2d" },
  { id: "k3", name: "redis-cache-a1b2c-ghi56", namespace: "production", status: "Running", restarts: 0, age: "3d" },
  { id: "k4", name: "batch-worker-err-1-jkl78", namespace: "jobs", status: "CrashLoopBackOff", restarts: 15, age: "6h" },
  { id: "k5", name: "postgres-primary-m3n-mno90", namespace: "data", status: "Running", restarts: 0, age: "5d" },
  { id: "k6", name: "migration-job-pqr12", namespace: "jobs", status: "Completed", restarts: 0, age: "1d" },
  { id: "k7", name: "auth-service-h6j7-stu34", namespace: "production", status: "Pending", restarts: 0, age: "10m" },
];

export default function ContainersPage() {
  const [searchQuery, setSearchQuery] = useState("");

  const stats = {
    running: mockContainers.filter((c) => c.status === "running").length,
    imagesScanned: mockImageScans.length,
    totalVulns: mockImageScans.reduce((sum, i) => sum + i.totalCves, 0),
    privileged: mockContainers.filter((c) => c.privileged).length,
    pods: mockK8sPods.length,
  };

  const filteredContainers = mockContainers.filter(
    (c) => !searchQuery || c.name.toLowerCase().includes(searchQuery.toLowerCase()) || c.image.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Container Security</h2>
          <p className="text-muted-foreground">Monitor container workloads and image vulnerabilities</p>
        </div>
        <Button variant="outline">
          <RefreshCw className="mr-2 h-4 w-4" />
          Rescan All
        </Button>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-5">
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-green-500/10 p-2"><Box className="h-5 w-5 text-green-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.running}</p><p className="text-xs text-muted-foreground">Running Containers</p></div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-blue-500/10 p-2"><Shield className="h-5 w-5 text-blue-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.imagesScanned}</p><p className="text-xs text-muted-foreground">Images Scanned</p></div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-red-500/10 p-2"><AlertTriangle className="h-5 w-5 text-red-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.totalVulns}</p><p className="text-xs text-muted-foreground">Vulnerabilities</p></div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-orange-500/10 p-2"><Lock className="h-5 w-5 text-orange-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.privileged}</p><p className="text-xs text-muted-foreground">Privileged Containers</p></div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-purple-500/10 p-2"><Layers className="h-5 w-5 text-purple-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.pods}</p><p className="text-xs text-muted-foreground">K8s Pods</p></div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="inventory" className="space-y-4">
        <div className="flex flex-wrap items-center gap-4">
          <TabsList>
            <TabsTrigger value="inventory">Container Inventory</TabsTrigger>
            <TabsTrigger value="images">Image Scans</TabsTrigger>
            <TabsTrigger value="runtime">Runtime Alerts</TabsTrigger>
            <TabsTrigger value="k8s">Kubernetes</TabsTrigger>
          </TabsList>
          <div className="relative min-w-[200px]">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input placeholder="Search..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} className="pl-9" />
          </div>
        </div>

        {/* Container Inventory Tab */}
        <TabsContent value="inventory">
          <Card className="border-border">
            <CardContent className="p-4">
              <div className="overflow-x-auto">
                <table className="w-full text-sm" role="table">
                  <thead>
                    <tr className="border-b border-border text-left">
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Name</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Image</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Status</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Vulnerabilities</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Privileged</th>
                      <th className="pb-3 font-medium text-muted-foreground">Started</th>
                    </tr>
                  </thead>
                  <tbody>
                    {filteredContainers.map((container) => (
                      <tr key={container.id} className="border-b border-border/50 hover:bg-muted/30 transition-colors">
                        <td className="py-3 pr-4 font-mono text-xs text-foreground">{container.name}</td>
                        <td className="py-3 pr-4 text-xs text-muted-foreground">{container.image}</td>
                        <td className="py-3 pr-4">
                          <span className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-medium ${
                            container.status === "running" ? "bg-green-500/20 text-green-400 border-green-500/30" :
                            container.status === "errored" ? "bg-red-500/20 text-red-400 border-red-500/30" :
                            "bg-gray-500/20 text-gray-400 border-gray-500/30"
                          }`}>
                            <Activity className="h-3 w-3" />{container.status}
                          </span>
                        </td>
                        <td className="py-3 pr-4">
                          {container.vulnerabilities > 0 ? (
                            <Badge variant={container.vulnerabilities >= 10 ? "critical" : container.vulnerabilities >= 5 ? "high" : "medium"}>{container.vulnerabilities} CVEs</Badge>
                          ) : (
                            <Badge variant="low">Clean</Badge>
                          )}
                        </td>
                        <td className="py-3 pr-4">
                          {container.privileged && <span className="inline-flex items-center gap-1 text-xs text-orange-400"><Lock className="h-3 w-3" />Yes</span>}
                          {!container.privileged && <span className="text-xs text-muted-foreground">No</span>}
                        </td>
                        <td className="py-3 text-xs text-muted-foreground">{new Date(container.started).toLocaleString()}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Image Scans Tab */}
        <TabsContent value="images">
          <Card className="border-border">
            <CardContent className="p-4">
              <div className="overflow-x-auto">
                <table className="w-full text-sm" role="table">
                  <thead>
                    <tr className="border-b border-border text-left">
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Image</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Tag</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Total CVEs</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Critical</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">High</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Medium</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Low</th>
                      <th className="pb-3 font-medium text-muted-foreground">Scanned</th>
                    </tr>
                  </thead>
                  <tbody>
                    {mockImageScans.map((scan) => (
                      <tr key={scan.id} className="border-b border-border/50 hover:bg-muted/30 transition-colors">
                        <td className="py-3 pr-4 font-mono text-xs text-foreground">{scan.image}</td>
                        <td className="py-3 pr-4 text-xs text-muted-foreground">{scan.tag}</td>
                        <td className="py-3 pr-4"><Badge variant={scan.totalCves >= 10 ? "critical" : scan.totalCves >= 5 ? "high" : scan.totalCves > 0 ? "medium" : "low"}>{scan.totalCves}</Badge></td>
                        <td className="py-3 pr-4 text-xs"><span className="text-red-400 font-semibold">{scan.critical}</span></td>
                        <td className="py-3 pr-4 text-xs"><span className="text-orange-400 font-semibold">{scan.high}</span></td>
                        <td className="py-3 pr-4 text-xs"><span className="text-yellow-400 font-semibold">{scan.medium}</span></td>
                        <td className="py-3 pr-4 text-xs"><span className="text-green-400 font-semibold">{scan.low}</span></td>
                        <td className="py-3 text-xs text-muted-foreground">{new Date(scan.scannedAt).toLocaleString()}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Runtime Alerts Tab */}
        <TabsContent value="runtime">
          <Card className="border-border">
            <CardContent className="p-4 space-y-3">
              {mockRuntimeAlerts.map((alert) => (
                <div key={alert.id} className="flex items-center gap-4 rounded-lg border border-border/50 p-3 hover:bg-muted/30 transition-colors">
                  <AlertTriangle className={`h-5 w-5 shrink-0 ${alert.severity === "critical" ? "text-red-400" : alert.severity === "high" ? "text-orange-400" : "text-yellow-400"}`} />
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-foreground truncate">{alert.event}</p>
                    <p className="text-xs text-muted-foreground">Container: {alert.container}</p>
                  </div>
                  <Badge variant={alert.severity}>{alert.severity}</Badge>
                  <span className="text-xs text-muted-foreground shrink-0">{new Date(alert.timestamp).toLocaleTimeString()}</span>
                </div>
              ))}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Kubernetes Tab */}
        <TabsContent value="k8s">
          <Card className="border-border">
            <CardContent className="p-4">
              <div className="overflow-x-auto">
                <table className="w-full text-sm" role="table">
                  <thead>
                    <tr className="border-b border-border text-left">
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Pod Name</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Namespace</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Status</th>
                      <th className="pb-3 pr-4 font-medium text-muted-foreground">Restarts</th>
                      <th className="pb-3 font-medium text-muted-foreground">Age</th>
                    </tr>
                  </thead>
                  <tbody>
                    {mockK8sPods.map((pod) => (
                      <tr key={pod.id} className="border-b border-border/50 hover:bg-muted/30 transition-colors">
                        <td className="py-3 pr-4 font-mono text-xs text-foreground">{pod.name}</td>
                        <td className="py-3 pr-4"><Badge variant="secondary" className="text-xs">{pod.namespace}</Badge></td>
                        <td className="py-3 pr-4">
                          <span className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-medium ${
                            pod.status === "Running" ? "bg-green-500/20 text-green-400 border-green-500/30" :
                            pod.status === "CrashLoopBackOff" ? "bg-red-500/20 text-red-400 border-red-500/30" :
                            pod.status === "Pending" ? "bg-yellow-500/20 text-yellow-400 border-yellow-500/30" :
                            "bg-blue-500/20 text-blue-400 border-blue-500/30"
                          }`}>{pod.status}</span>
                        </td>
                        <td className="py-3 pr-4 text-xs">
                          <span className={pod.restarts > 5 ? "text-red-400 font-semibold" : "text-muted-foreground"}>{pod.restarts}</span>
                        </td>
                        <td className="py-3 text-xs text-muted-foreground">{pod.age}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
