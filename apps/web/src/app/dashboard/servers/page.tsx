import { Server, Cpu, HardDrive, MemoryStick, Activity } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import type { ServerStatus } from "@/types";

const mockServers = [
  { id: "1", hostname: "web-01", ipAddress: "10.0.1.10", status: "online" as ServerStatus, cpuUsage: 45, memoryUsage: 62, diskUsage: 38, os: "Ubuntu 22.04", lastHeartbeat: "10s ago", alerts: 0 },
  { id: "2", hostname: "web-02", ipAddress: "10.0.1.11", status: "online" as ServerStatus, cpuUsage: 72, memoryUsage: 81, diskUsage: 55, os: "Ubuntu 22.04", lastHeartbeat: "8s ago", alerts: 1 },
  { id: "3", hostname: "db-primary", ipAddress: "10.0.2.10", status: "online" as ServerStatus, cpuUsage: 58, memoryUsage: 74, diskUsage: 67, os: "RHEL 9", lastHeartbeat: "5s ago", alerts: 0 },
  { id: "4", hostname: "db-replica", ipAddress: "10.0.2.11", status: "online" as ServerStatus, cpuUsage: 32, memoryUsage: 45, diskUsage: 67, os: "RHEL 9", lastHeartbeat: "12s ago", alerts: 0 },
  { id: "5", hostname: "cache-01", ipAddress: "10.0.3.10", status: "degraded" as ServerStatus, cpuUsage: 89, memoryUsage: 92, diskUsage: 41, os: "Debian 12", lastHeartbeat: "3s ago", alerts: 2 },
  { id: "6", hostname: "monitor-01", ipAddress: "10.0.4.10", status: "online" as ServerStatus, cpuUsage: 25, memoryUsage: 38, diskUsage: 22, os: "Ubuntu 22.04", lastHeartbeat: "6s ago", alerts: 0 },
  { id: "7", hostname: "api-gateway", ipAddress: "10.0.1.20", status: "offline" as ServerStatus, cpuUsage: 0, memoryUsage: 0, diskUsage: 45, os: "Alpine 3.19", lastHeartbeat: "5m ago", alerts: 3 },
];

const statusColors: Record<ServerStatus, string> = {
  online: "bg-green-500",
  offline: "bg-red-500",
  degraded: "bg-yellow-500",
  maintenance: "bg-blue-500",
};

function UsageBar({ value, label }: { value: number; label: string }) {
  const color = value > 85 ? "text-red-400" : value > 70 ? "text-yellow-400" : "text-green-400";
  return (
    <div className="space-y-1">
      <div className="flex justify-between text-xs">
        <span className="text-muted-foreground">{label}</span>
        <span className={color}>{value}%</span>
      </div>
      <Progress value={value} className="h-1.5" />
    </div>
  );
}

export default function ServersPage() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Servers</h2>
          <p className="text-muted-foreground">Monitor server health and performance</p>
        </div>
        <div className="flex items-center gap-2">
          <Badge variant="default">{mockServers.filter((s) => s.status === "online").length} Online</Badge>
          <Badge variant="destructive">{mockServers.filter((s) => s.status === "offline").length} Offline</Badge>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
        {mockServers.map((server) => (
          <Card key={server.id} className="border-border hover:border-primary/30 transition-colors">
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2 text-sm">
                  <Server className="h-4 w-4 text-muted-foreground" />
                  {server.hostname}
                </CardTitle>
                <div className="flex items-center gap-2">
                  <span className={`h-2.5 w-2.5 rounded-full ${statusColors[server.status]}`} />
                  <span className="text-xs capitalize text-muted-foreground">{server.status}</span>
                </div>
              </div>
              <p className="text-xs text-muted-foreground">{server.ipAddress} • {server.os} • {server.lastHeartbeat}</p>
            </CardHeader>
            <CardContent className="space-y-3">
              <UsageBar value={server.cpuUsage} label="CPU" />
              <UsageBar value={server.memoryUsage} label="Memory" />
              <UsageBar value={server.diskUsage} label="Disk" />
              {server.alerts > 0 && (
                <div className="pt-2 border-t border-border">
                  <Badge variant="destructive" className="text-xs">{server.alerts} alert{server.alerts > 1 ? "s" : ""}</Badge>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
