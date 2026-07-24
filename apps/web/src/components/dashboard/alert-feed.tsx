"use client";

import { AlertTriangle, Shield, Info, AlertOctagon } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import type { ThreatLevel } from "@/types";

interface AlertItem {
  id: string;
  title: string;
  severity: ThreatLevel;
  source: string;
  timestamp: string;
}

const mockAlerts: AlertItem[] = [
  {
    id: "1",
    title: "Brute force attempt detected on SSH",
    severity: "critical",
    source: "Network Monitor",
    timestamp: "2 min ago",
  },
  {
    id: "2",
    title: "Unauthorized access attempt to /admin",
    severity: "high",
    source: "WAF",
    timestamp: "5 min ago",
  },
  {
    id: "3",
    title: "Suspicious outbound traffic to known C2 server",
    severity: "critical",
    source: "Threat Intel",
    timestamp: "8 min ago",
  },
  {
    id: "4",
    title: "SSL certificate expiring in 7 days",
    severity: "medium",
    source: "Certificate Monitor",
    timestamp: "15 min ago",
  },
  {
    id: "5",
    title: "New device connected to network segment",
    severity: "low",
    source: "Network Scanner",
    timestamp: "22 min ago",
  },
  {
    id: "6",
    title: "Database query latency above threshold",
    severity: "medium",
    source: "DB Monitor",
    timestamp: "30 min ago",
  },
];

const severityConfig: Record<ThreatLevel, { icon: typeof AlertTriangle; color: string }> = {
  critical: { icon: AlertOctagon, color: "text-red-400" },
  high: { icon: AlertTriangle, color: "text-orange-400" },
  medium: { icon: Shield, color: "text-yellow-400" },
  low: { icon: Info, color: "text-green-400" },
  info: { icon: Info, color: "text-blue-400" },
};

export function AlertFeed() {
  return (
    <Card className="border-border">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-base">
            <AlertTriangle className="h-5 w-5 text-destructive" />
            Live Alert Feed
          </CardTitle>
          <Badge variant="destructive" className="text-xs">
            {mockAlerts.filter((a) => a.severity === "critical").length} Critical
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-3 max-h-[400px] overflow-y-auto pr-2">
          {mockAlerts.map((alert) => {
            const config = severityConfig[alert.severity];
            const Icon = config.icon;

            return (
              <div
                key={alert.id}
                className="flex items-start gap-3 rounded-lg border border-border p-3 transition-colors hover:bg-accent/50"
              >
                <Icon className={cn("h-5 w-5 mt-0.5 shrink-0", config.color)} />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-foreground truncate">
                    {alert.title}
                  </p>
                  <div className="mt-1 flex items-center gap-2">
                    <span className="text-xs text-muted-foreground">
                      {alert.source}
                    </span>
                    <span className="text-xs text-muted-foreground">•</span>
                    <span className="text-xs text-muted-foreground">
                      {alert.timestamp}
                    </span>
                  </div>
                </div>
                <Badge
                  variant={alert.severity as "critical" | "high" | "medium" | "low"}
                  className="shrink-0"
                >
                  {alert.severity}
                </Badge>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}
