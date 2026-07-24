"use client";

import { useState } from "react";
import {
  HardDrive, CheckCircle2, AlertTriangle, XCircle, Clock, Shield, RefreshCw, Database,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";

interface Backup {
  id: string;
  server: string;
  database: string;
  lastBackup: string;
  age: string;
  size: string;
  status: "verified" | "unverified" | "failed";
}

const mockBackups: Backup[] = [
  { id: "1", server: "prod-db-01", database: "users_db", lastBackup: "2024-01-15 06:00", age: "4h", size: "2.3 GB", status: "verified" },
  { id: "2", server: "prod-db-01", database: "orders_db", lastBackup: "2024-01-15 06:00", age: "4h", size: "8.1 GB", status: "verified" },
  { id: "3", server: "prod-db-02", database: "analytics_db", lastBackup: "2024-01-15 02:00", age: "8h", size: "15.4 GB", status: "unverified" },
  { id: "4", server: "prod-web-01", database: "config_store", lastBackup: "2024-01-14 22:00", age: "12h", size: "450 MB", status: "verified" },
  { id: "5", server: "prod-api-01", database: "sessions_db", lastBackup: "2024-01-14 06:00", age: "28h", size: "1.2 GB", status: "unverified" },
  { id: "6", server: "staging-db-01", database: "staging_db", lastBackup: "2024-01-13 06:00", age: "52h", size: "3.8 GB", status: "failed" },
];

const stats = [
  { label: "Total Backups", value: "6", icon: HardDrive, color: "text-blue-400" },
  { label: "Healthy", value: "4", icon: CheckCircle2, color: "text-green-400" },
  { label: "Warning (Aging)", value: "1", icon: AlertTriangle, color: "text-yellow-400" },
  { label: "Critical (Overdue)", value: "1", icon: XCircle, color: "text-red-400" },
];

const statusConfig: Record<string, { color: string; icon: typeof CheckCircle2 }> = {
  verified: { color: "text-green-400", icon: CheckCircle2 },
  unverified: { color: "text-yellow-400", icon: AlertTriangle },
  failed: { color: "text-red-400", icon: XCircle },
};

export default function BackupsPage() {
  const [verifying, setVerifying] = useState<string | null>(null);

  const handleVerify = (id: string) => {
    setVerifying(id);
    setTimeout(() => setVerifying(null), 2000);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Backup Management</h2>
          <p className="text-muted-foreground">Monitor backup health, RPO/RTO compliance</p>
        </div>
        <Badge variant="default" className="text-sm">RPO Compliance: 83%</Badge>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.label} className="border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <stat.icon className={`h-8 w-8 ${stat.color}`} />
                <div>
                  <p className="text-2xl font-bold text-foreground">{stat.value}</p>
                  <p className="text-xs text-muted-foreground">{stat.label}</p>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* RPO/RTO Gauges */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Card className="border-border">
          <CardHeader className="pb-3">
            <CardTitle className="text-lg">RPO (Recovery Point Objective)</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Target: 4 hours</span>
              <span className="text-sm font-medium text-green-400">83% compliant</span>
            </div>
            <Progress value={83} className="h-3" />
            <p className="text-xs text-muted-foreground">5 of 6 backups within RPO window</p>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardHeader className="pb-3">
            <CardTitle className="text-lg">RTO (Recovery Time Objective)</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Target: 1 hour</span>
              <span className="text-sm font-medium text-green-400">100% tested</span>
            </div>
            <Progress value={100} className="h-3" />
            <p className="text-xs text-muted-foreground">All verified backups restore within RTO</p>
          </CardContent>
        </Card>
      </div>

      {/* Backup Inventory */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Database className="h-5 w-5 text-blue-400" />
            Backup Inventory
          </CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full text-sm" role="table">
              <thead>
                <tr className="border-b border-border bg-muted/30">
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Server</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Database</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Last Backup</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Age</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Size</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Action</th>
                </tr>
              </thead>
              <tbody>
                {mockBackups.map((backup) => {
                  const cfg = statusConfig[backup.status];
                  const StatusIcon = cfg.icon;
                  return (
                    <tr key={backup.id} className="border-b border-border hover:bg-muted/20">
                      <td className="px-4 py-3 font-medium text-foreground">{backup.server}</td>
                      <td className="px-4 py-3 text-muted-foreground">{backup.database}</td>
                      <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{backup.lastBackup}</td>
                      <td className="px-4 py-3 text-muted-foreground">{backup.age}</td>
                      <td className="px-4 py-3 text-muted-foreground">{backup.size}</td>
                      <td className="px-4 py-3">
                        <span className={`flex items-center gap-1.5 text-xs font-medium ${cfg.color}`}>
                          <StatusIcon className="h-3.5 w-3.5" />{backup.status}
                        </span>
                      </td>
                      <td className="px-4 py-3">
                        <Button variant="outline" size="sm" disabled={verifying === backup.id} onClick={() => handleVerify(backup.id)} className="gap-1 text-xs">
                          <RefreshCw className={`h-3 w-3 ${verifying === backup.id ? "animate-spin" : ""}`} />
                          {verifying === backup.id ? "Verifying..." : "Verify"}
                        </Button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
