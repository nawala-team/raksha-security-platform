"use client";

import { AlertTriangle, Shield, Info, AlertOctagon } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { DataState } from "@/components/ui/data-state";
import { useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { ThreatLevel } from "@/types";

interface AlertItem {
  id: string;
  title: string;
  severity: ThreatLevel;
  source: string;
  created_at?: string;
  createdAt?: string;
}

const severityConfig: Record<ThreatLevel, { icon: typeof AlertTriangle; color: string }> = {
  critical: { icon: AlertOctagon, color: "text-red-400" },
  high: { icon: AlertTriangle, color: "text-orange-400" },
  medium: { icon: Shield, color: "text-yellow-400" },
  low: { icon: Info, color: "text-green-400" },
  info: { icon: Info, color: "text-blue-400" },
};

export function AlertFeed() {
  const { items: alerts, loading, error, refetch } = useApiList<AlertItem>(() =>
    api.alerts.list({ per_page: "10" })
  );

  return (
    <Card className="border-border">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-base">
            <AlertTriangle className="h-5 w-5 text-destructive" />
            Live Alert Feed
          </CardTitle>
          <Badge variant="destructive" className="text-xs">
            {alerts.filter((a) => a.severity === "critical").length} Critical
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        <DataState
          loading={loading}
          error={error}
          isEmpty={alerts.length === 0}
          onRetry={refetch}
          loadingLabel="Loading live alerts"
          emptyTitle="No active alerts"
          emptyDescription="New alerts from agents and detectors will appear here."
        >
          <div className="space-y-3 max-h-[400px] overflow-y-auto pr-2">
            {alerts.map((alert) => {
            const config = severityConfig[alert.severity];
            const Icon = config.icon;
            const ts = alert.created_at ?? alert.createdAt;

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
                      {ts ? new Date(ts).toLocaleString() : "unknown time"}
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
        </DataState>
      </CardContent>
    </Card>
  );
}
