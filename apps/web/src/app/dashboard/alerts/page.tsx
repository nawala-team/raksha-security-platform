"use client";

import { useState } from "react";
import { AlertTriangle, Search, CheckCircle2, Clock, XCircle } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import type { Alert, ThreatLevel, AlertStatus } from "@/types";

const mockAlerts: Alert[] = [
  { id: "1", title: "Brute force attack on SSH port 22", description: "Multiple failed login attempts from 192.168.1.105", severity: "critical", status: "active", source: "IDS", timestamp: "2024-01-15T10:30:00Z", tags: ["brute-force", "ssh"] },
  { id: "2", title: "Unauthorized API access attempt", description: "Invalid bearer token on /api/admin", severity: "high", status: "active", source: "WAF", timestamp: "2024-01-15T10:25:00Z", tags: ["api", "auth"] },
  { id: "3", title: "Suspicious outbound connection to C2", description: "Host web-03 connecting to known malicious IP", severity: "critical", status: "active", source: "Threat Intel", timestamp: "2024-01-15T10:20:00Z", tags: ["c2", "malware"] },
  { id: "4", title: "SSL certificate expiring soon", description: "Certificate for api.example.com expires in 7 days", severity: "medium", status: "acknowledged", source: "Cert Monitor", timestamp: "2024-01-15T09:00:00Z", tags: ["ssl"] },
  { id: "5", title: "Unusual database query pattern", description: "Potential SQL injection attempt detected", severity: "high", status: "active", source: "DB Monitor", timestamp: "2024-01-15T08:45:00Z", tags: ["sql-injection"] },
  { id: "6", title: "New device on restricted network", description: "Unknown MAC on VLAN 10", severity: "medium", status: "acknowledged", source: "Scanner", timestamp: "2024-01-15T08:30:00Z", tags: ["network"] },
  { id: "7", title: "Failed compliance check - PCI-DSS", description: "Weak password policy", severity: "low", status: "resolved", source: "Compliance", timestamp: "2024-01-15T07:00:00Z", tags: ["pci-dss"] },
];

const statusIcons = { active: AlertTriangle, acknowledged: Clock, resolved: CheckCircle2, false_positive: XCircle };

export default function AlertsPage() {
  const [searchQuery, setSearchQuery] = useState("");
  const [severityFilter, setSeverityFilter] = useState<ThreatLevel | "all">("all");

  const filtered = mockAlerts.filter((alert) => {
    if (severityFilter !== "all" && alert.severity !== severityFilter) return false;
    if (searchQuery && !alert.title.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Alerts</h2>
          <p className="text-muted-foreground">Monitor and manage security alerts</p>
        </div>
        <Badge variant="destructive">{mockAlerts.filter((a) => a.status === "active").length} Active</Badge>
      </div>

      <Card className="border-border">
        <CardContent className="p-4">
          <div className="flex flex-wrap items-center gap-3">
            <div className="relative flex-1 min-w-[200px]">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input placeholder="Search alerts..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} className="pl-9" />
            </div>
            <div className="flex gap-2">
              {(["all", "critical", "high", "medium", "low"] as const).map((level) => (
                <Button key={level} variant={severityFilter === level ? "default" : "outline"} size="sm" onClick={() => setSeverityFilter(level)} className="capitalize">{level}</Button>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      <div className="space-y-3">
        {filtered.map((alert) => {
          const StatusIcon = statusIcons[alert.status];
          const statusColor = alert.status === "active" ? "text-red-400" : alert.status === "acknowledged" ? "text-yellow-400" : "text-green-400";
          return (
            <Card key={alert.id} className="border-border hover:border-primary/30 transition-colors">
              <CardContent className="p-4">
                <div className="flex items-start gap-4">
                  <StatusIcon className={`h-5 w-5 mt-0.5 shrink-0 ${statusColor}`} />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <h3 className="text-sm font-medium truncate">{alert.title}</h3>
                      <Badge variant={alert.severity as "critical" | "high" | "medium" | "low"}>{alert.severity}</Badge>
                    </div>
                    <p className="text-xs text-muted-foreground mb-2">{alert.description}</p>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      <span>{alert.source}</span><span>•</span>
                      <span>{new Date(alert.timestamp).toLocaleString()}</span>
                    </div>
                  </div>
                  <Button variant="outline" size="sm">View</Button>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
