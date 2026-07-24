import { Database, Activity, Shield, AlertTriangle, Users, HardDrive } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import type { DatabaseInstance, ServerStatus } from "@/types";

const mockDatabases: (DatabaseInstance & { alerts: number; encrypted: boolean })[] = [
  { id: "1", name: "primary-postgres", type: "postgresql", status: "online", connections: 45, maxConnections: 100, queryRate: 1250, replicationLag: 0, size: "128 GB", alerts: 0, encrypted: true },
  { id: "2", name: "replica-postgres-01", type: "postgresql", status: "online", connections: 32, maxConnections: 100, queryRate: 890, replicationLag: 120, size: "128 GB", alerts: 0, encrypted: true },
  { id: "3", name: "cache-redis", type: "redis", status: "degraded", connections: 89, maxConnections: 128, queryRate: 5400, size: "16 GB", alerts: 2, encrypted: false },
  { id: "4", name: "analytics-mongo", type: "mongodb", status: "online", connections: 18, maxConnections: 50, queryRate: 340, size: "256 GB", alerts: 1, encrypted: true },
  { id: "5", name: "sessions-redis", type: "redis", status: "online", connections: 52, maxConnections: 128, queryRate: 3200, size: "8 GB", alerts: 0, encrypted: true },
];

const typeIcons: Record<string, string> = { postgresql: "🐘", mysql: "🐬", mongodb: "🍃", redis: "⚡" };
const statusColors: Record<ServerStatus, string> = { online: "bg-green-500", offline: "bg-red-500", degraded: "bg-yellow-500", maintenance: "bg-blue-500" };

export default function DatabasePage() {
  const stats = [
    { label: "Total Instances", value: mockDatabases.length, icon: Database },
    { label: "Active Connections", value: mockDatabases.reduce((a, d) => a + d.connections, 0), icon: Users },
    { label: "Queries/sec", value: mockDatabases.reduce((a, d) => a + d.queryRate, 0).toLocaleString(), icon: Activity },
    { label: "Encrypted", value: `${mockDatabases.filter((d) => d.encrypted).length}/${mockDatabases.length}`, icon: Shield },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Database Security</h2>
          <p className="text-muted-foreground">Monitor database instances and security posture</p>
        </div>
        <div className="flex items-center gap-2">
          <Badge variant="default">{mockDatabases.filter((d) => d.status === "online").length} Healthy</Badge>
          {mockDatabases.some((d) => d.alerts > 0) && (
            <Badge variant="destructive">{mockDatabases.reduce((a, d) => a + d.alerts, 0)} Alerts</Badge>
          )}
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.label} className="border-border">
            <CardContent className="p-4 flex items-center gap-3">
              <stat.icon className="h-8 w-8 text-primary/60" />
              <div>
                <p className="text-2xl font-bold text-foreground">{stat.value}</p>
                <p className="text-xs text-muted-foreground">{stat.label}</p>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
        {mockDatabases.map((db) => (
          <Card key={db.id} className="border-border hover:border-primary/30 transition-colors">
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2 text-sm">
                  <span className="text-lg">{typeIcons[db.type]}</span>
                  {db.name}
                </CardTitle>
                <div className="flex items-center gap-2">
                  <span className={`h-2.5 w-2.5 rounded-full ${statusColors[db.status]}`} />
                  <span className="text-xs capitalize text-muted-foreground">{db.status}</span>
                </div>
              </div>
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <span className="capitalize">{db.type}</span>
                <span>•</span>
                <HardDrive className="h-3 w-3" />
                <span>{db.size}</span>
                {db.encrypted && (
                  <>
                    <span>•</span>
                    <Shield className="h-3 w-3 text-green-400" />
                    <span className="text-green-400">Encrypted</span>
                  </>
                )}
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="space-y-1">
                <div className="flex justify-between text-xs">
                  <span className="text-muted-foreground">Connections</span>
                  <span>{db.connections}/{db.maxConnections}</span>
                </div>
                <Progress value={(db.connections / db.maxConnections) * 100} className="h-1.5" />
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-muted-foreground">Query Rate</span>
                <span className="font-mono">{db.queryRate.toLocaleString()} q/s</span>
              </div>
              {db.replicationLag !== undefined && (
                <div className="flex justify-between text-xs">
                  <span className="text-muted-foreground">Replication Lag</span>
                  <span className={db.replicationLag > 500 ? "text-red-400" : "text-green-400"}>{db.replicationLag}ms</span>
                </div>
              )}
              {db.alerts > 0 && (
                <div className="pt-2 border-t border-border flex items-center gap-2">
                  <AlertTriangle className="h-3.5 w-3.5 text-destructive" />
                  <span className="text-xs text-destructive">{db.alerts} active alert{db.alerts > 1 ? "s" : ""}</span>
                </div>
              )}
              {!db.encrypted && (
                <div className="pt-2 border-t border-border flex items-center gap-2">
                  <Shield className="h-3.5 w-3.5 text-yellow-400" />
                  <span className="text-xs text-yellow-400">Encryption not enabled</span>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
