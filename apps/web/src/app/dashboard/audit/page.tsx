"use client";

import { ScrollText, CheckCircle2, XCircle, RefreshCw } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DataState } from "@/components/ui/data-state";
import { useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";

interface AuditRow {
  id: string;
  timestamp: string;
  actor_email: string | null;
  action_type: string;
  action_category: string;
  resource_type: string;
  resource_id: string | null;
  risk_level: string;
  integrity_hash: string;
}

export default function AuditPage() {
  const { items, loading, error, refetch } = useApiList<AuditRow>(() => api.audit.list());

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Audit Trail</h2>
          <p className="text-muted-foreground">Complete system activity log</p>
        </div>
        <Button variant="outline" size="sm" onClick={refetch} aria-label="Refresh">
          <RefreshCw className="h-4 w-4" aria-hidden="true" />
        </Button>
      </div>

      <DataState
        loading={loading}
        error={error}
        isEmpty={items.length === 0}
        onRetry={refetch}
        loadingLabel="Loading audit trail"
        emptyTitle="No audit entries"
        emptyDescription="System activity will be logged here."
      >
        <Card className="border-border">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center gap-2 text-base">
              <ScrollText className="h-5 w-5 text-primary" aria-hidden="true" /> Recent Activity
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {items.map((entry) => {
                const ok = entry.risk_level !== "critical" && entry.risk_level !== "high";
                return (
                  <div key={entry.id} className="flex items-start gap-3 rounded-lg border border-border p-3 hover:bg-accent/50 transition-colors">
                    {ok ? (
                      <CheckCircle2 className="h-4 w-4 mt-0.5 text-green-500 shrink-0" aria-hidden="true" />
                    ) : (
                      <XCircle className="h-4 w-4 mt-0.5 text-red-500 shrink-0" aria-hidden="true" />
                    )}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1 flex-wrap">
                        <span className="text-sm font-medium font-mono">{entry.action_type}</span>
                        <Badge variant={ok ? "default" : "destructive"} className="text-[10px]">{entry.risk_level}</Badge>
                        <span className="text-[10px] text-muted-foreground">{entry.action_category}</span>
                      </div>
                      <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground flex-wrap">
                        <span>{entry.actor_email ?? "system"}</span>
                        <span>•</span>
                        <span>{entry.resource_type}{entry.resource_id ? `:${entry.resource_id}` : ""}</span>
                        <span>•</span>
                        <span>{new Date(entry.timestamp).toLocaleString()}</span>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>
      </DataState>
    </div>
  );
}
