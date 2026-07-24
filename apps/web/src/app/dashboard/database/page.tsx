import { Database, Activity } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import type { DatabaseInstance } from "@/types";

const mockDatabases: DatabaseInstance[] = [
  { id: "1", name: "raksha-primary", type: "postgresql", status: "online", connections: 45, maxConnections: 100, queryRate: 1250, replicationLag: 0, size: "24.5 GB" },
  { id: "2", name: "raksha-replica", type: "postgresql", status: "online", connections: 32, maxConnections: 100, queryRate: 890, replicationLag: 12, size: "24.3 GB" },
  { id: "3", name: "session-store", type: "redis", status: "online", connections: 128, maxConnections: 500, queryRate: 5400, size: "2.1 GB" },
  { id: "4", name: "analytics-db", type: "mongodb", status: "degraded", connections: 67, maxConnections: 200, queryRate: 340, size: "156 GB" },
];

const typeColors = { postgresql: "text-blue-400", mysql: "text-orange-400", mongodb: "text-green-400", redis: "text-red-400" };
const statusBadge = { online: "default" as const, offline: "destructive" as const, degraded: "high" as const, maintenance: "secondary" as const };

export default function DatabasePage() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Database Monitoring</h2>
        <p className="text-muted-foreground">Monitor database health and performance</p>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {mockDatabases.map((db) => (
          <Card key={db.id} className="border-border">
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center gap-2 text-sm">
                  <Database className={`h-4 w-4 ${typeColors[db.type]}`} />
                  {db.name}
                </CardTitle>
                <Badge variant={statusBadge[db.status]}>{db.status}</Badge>
              </div>
              <p className="text-xs text-muted-foreground capitalize">{db.type} • {db.size}</p>
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
                <span className="flex items-center gap-1"><Activity className="h-3 w-3" />{db.queryRate}/s</span>
              </div>
              {db.replicationLag !== undefined && (
                <div className="flex justify-between text-xs">
                  <span className="text-muted-foreground">Replication Lag</span>
                  <span className={db.replicationLag > 50 ? "text-red-400" : "text-green-400"}>{db.replicationLag}ms</span>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
