"use client";

import { ClipboardCheck, CheckCircle2, XCircle, AlertTriangle, RefreshCw } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DataState } from "@/components/ui/data-state";
import { useApiData } from "@/hooks/use-api-data";
import { api } from "@/lib/api";

/** Mirrors the portal's `StandardResponse`. */
interface StandardRow {
  id: string;
  name: string;
  version: string;
  description: string | null;
  authority: string | null;
  is_active: boolean;
  created_at: string;
}

export default function CompliancePage() {
  const { data, loading, error, refetch } = useApiData<StandardRow[]>(() =>
    api.compliance.standards()
  );
  const standards = data ?? [];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Compliance</h2>
          <p className="text-muted-foreground">Framework compliance standards and controls</p>
        </div>
        <Button variant="outline" size="sm" onClick={refetch} aria-label="Refresh">
          <RefreshCw className="h-4 w-4" aria-hidden="true" />
        </Button>
      </div>

      <DataState
        loading={loading}
        error={error}
        isEmpty={standards.length === 0}
        onRetry={refetch}
        loadingLabel="Loading compliance standards"
        emptyTitle="No standards configured"
        emptyDescription="Compliance standards will appear here once configured."
      >
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {standards.map((s) => (
            <Card key={s.id} className="border-border">
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-sm">{s.name}</CardTitle>
                  <Badge variant={s.is_active ? "default" : "outline"}>
                    {s.is_active ? "Active" : "Inactive"}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent className="space-y-2">
                <p className="text-xs text-muted-foreground">
                  Version {s.version}
                  {s.authority ? ` • ${s.authority}` : ""}
                </p>
                {s.description && (
                  <p className="text-xs text-muted-foreground">{s.description}</p>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      </DataState>
    </div>
  );
}
