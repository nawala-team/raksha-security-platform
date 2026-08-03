"use client";

import { useState } from "react";
import {
  Play, Clock, BookOpen, Calendar, CheckCircle2, XCircle,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { DataState } from "@/components/ui/data-state";
import { useApiData, useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { formatNumber, relativeTime } from "@/lib/utils";

/** Mirrors the portal's `HuntingQueryResponse`. */
interface HuntingQuery {
  id: string;
  name: string;
  description: string | null;
  rql: string;
  query_source: string;
  is_scheduled: boolean;
  schedule_interval_mins: number | null;
  alert_on_hits: boolean;
  alert_threshold: number;
  alert_severity: string;
  last_run_at: string | null;
  next_run_at: string | null;
  last_hit_count: number | null;
  run_count: number;
  created_at: string;
}

/** Mirrors the portal's `HuntingRunResponse`. */
interface HuntingRun {
  id: string;
  query_id: string;
  trigger: string;
  status: string;
  total_hits: number | null;
  duration_ms: number | null;
  alert_triggered: boolean;
  alert_id: string | null;
  error_message: string | null;
  started_at: string;
  completed_at: string | null;
}

/** Mirrors the portal's `ValidateResponse` from POST /hunting/validate. */
interface ValidateResult {
  valid: boolean;
  error: string | null;
  source: string | null;
  has_filter: boolean;
  aggregation_count: number;
  limit: number | null;
}

const severityVariants: Record<string, "critical" | "high" | "medium" | "low"> = {
  critical: "critical",
  high: "high",
  medium: "medium",
  low: "low",
};

export default function HuntingPage() {
  const queries = useApiData<HuntingQuery[]>(() => api.hunting.queries());
  const runs = useApiList<HuntingRun>(() => api.hunting.runs());

  const [query, setQuery] = useState("");
  const [validating, setValidating] = useState(false);
  const [validation, setValidation] = useState<ValidateResult | null>(null);
  const [validationError, setValidationError] = useState<string | null>(null);

  const savedQueries = queries.data ?? [];

  // Runs reference their query by id; map ids to names for display.
  const queryNames = new Map(savedQueries.map((q) => [q.id, q.name]));

  // Validation happens against the real RQL parser in the portal, so it needs
  // its own request rather than one of the page-level fetch hooks.
  const handleValidate = async () => {
    setValidating(true);
    setValidation(null);
    setValidationError(null);
    try {
      const result = (await api.hunting.validate(query)) as ValidateResult;
      setValidation(result);
    } catch (err: unknown) {
      setValidationError(err instanceof Error ? err.message : "Validation failed");
    } finally {
      setValidating(false);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Threat Hunting</h2>
        <p className="text-muted-foreground">Query security data with Raksha Query Language (RQL)</p>
      </div>

      {/* Query Editor */}
      <Card className="border-border">
        <CardContent className="p-4 space-y-4">
          <div className="flex items-center gap-2">
            <Select value={query} onValueChange={setQuery}>
              <SelectTrigger className="w-64" aria-label="Load a saved query">
                <SelectValue placeholder="Saved queries..." />
              </SelectTrigger>
              <SelectContent>
                {savedQueries.map((sq) => (
                  <SelectItem key={sq.id} value={sq.rql}>{sq.name}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <textarea
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="w-full h-32 rounded-md border border-border bg-muted/30 p-3 font-mono text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring resize-y"
            placeholder="Enter RQL query..."
            spellCheck={false}
            aria-label="Query editor"
          />
          <div className="flex items-center gap-2">
            <Button onClick={handleValidate} disabled={validating || !query.trim()} className="gap-2">
              <Play className="h-4 w-4" aria-hidden="true" />
              {validating ? "Validating..." : "Validate RQL"}
            </Button>
          </div>

          {validationError && (
            <p className="text-sm text-red-400" role="alert">{validationError}</p>
          )}

          {validation && (
            <div
              className={`rounded-md border p-3 text-sm ${
                validation.valid
                  ? "border-green-500/30 bg-green-500/5"
                  : "border-red-500/30 bg-red-500/5"
              }`}
              role="status"
              aria-live="polite"
            >
              <p className={`flex items-center gap-2 font-medium ${validation.valid ? "text-green-400" : "text-red-400"}`}>
                {validation.valid ? (
                  <CheckCircle2 className="h-4 w-4" aria-hidden="true" />
                ) : (
                  <XCircle className="h-4 w-4" aria-hidden="true" />
                )}
                {validation.valid ? "Valid RQL" : "Invalid RQL"}
              </p>
              {validation.error && (
                <p className="mt-1 font-mono text-xs text-muted-foreground">{validation.error}</p>
              )}
              {validation.valid && (
                <div className="mt-2 flex flex-wrap gap-3 text-xs text-muted-foreground">
                  <span>Source: {validation.source ?? "—"}</span>
                  <span>Filter: {validation.has_filter ? "yes" : "none"}</span>
                  <span>Aggregations: {formatNumber(validation.aggregation_count)}</span>
                  <span>Limit: {validation.limit === null ? "unset" : formatNumber(validation.limit)}</span>
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Run History */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2 text-lg">
              <Clock className="h-5 w-5 text-blue-400" aria-hidden="true" />
              Run History
            </CardTitle>
            <span className="text-xs text-muted-foreground">{formatNumber(runs.total)} runs</span>
          </div>
        </CardHeader>
        <CardContent className="p-0">
          <DataState
            loading={runs.loading}
            error={runs.error}
            isEmpty={runs.items.length === 0}
            onRetry={runs.refetch}
            loadingLabel="Loading run history"
            emptyTitle="No hunting runs yet"
            emptyDescription="Scheduled and manual query runs appear here once they execute."
          >
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <caption className="sr-only">
                  Hunting query run history with trigger, status, hit count and duration.
                </caption>
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Query</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Trigger</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Hits</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Duration</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Started</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Alert</th>
                  </tr>
                </thead>
                <tbody>
                  {runs.items.map((run) => (
                    <tr key={run.id} className="border-b border-border transition-colors hover:bg-muted/20">
                      <td className="px-4 py-3 text-foreground">
                        {queryNames.get(run.query_id) ?? "—"}
                        {run.error_message && (
                          <p className="font-mono text-xs text-red-400">{run.error_message}</p>
                        )}
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">{run.trigger}</td>
                      <td className="px-4 py-3">
                        <Badge variant={run.status === "failed" ? "destructive" : "outline"}>
                          {run.status}
                        </Badge>
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">{formatNumber(run.total_hits)}</td>
                      <td className="px-4 py-3 text-muted-foreground">
                        {run.duration_ms === null ? "—" : `${formatNumber(run.duration_ms)} ms`}
                      </td>
                      <td className="px-4 py-3 text-xs text-muted-foreground">{relativeTime(run.started_at)}</td>
                      <td className="px-4 py-3">
                        {run.alert_triggered ? (
                          <Badge variant="high" className="text-xs">alert raised</Badge>
                        ) : (
                          <span className="text-xs text-muted-foreground">—</span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </DataState>
        </CardContent>
      </Card>


      {/* Saved Queries */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <BookOpen className="h-5 w-5 text-blue-400" aria-hidden="true" /> Saved Queries
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <DataState
            loading={queries.loading}
            error={queries.error}
            isEmpty={savedQueries.length === 0}
            onRetry={queries.refetch}
            loadingLabel="Loading saved queries"
            emptyTitle="No saved queries"
            emptyDescription="Saved RQL hunts appear here with their schedule and run history."
          >
            <div className="space-y-2">
              {savedQueries.map((sq) => (
                <div key={sq.id} className="flex items-center justify-between rounded-lg border border-border px-4 py-3">
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-foreground">{sq.name}</span>
                      {sq.is_scheduled && (
                        <Badge variant="secondary" className="text-xs">
                          <Calendar className="mr-1 h-3 w-3" aria-hidden="true" />
                          {sq.schedule_interval_mins === null
                            ? "Scheduled"
                            : `Every ${formatNumber(sq.schedule_interval_mins)}m`}
                        </Badge>
                      )}
                      {sq.alert_on_hits && severityVariants[sq.alert_severity] && (
                        <Badge variant={severityVariants[sq.alert_severity]} className="text-xs">
                          alerts {sq.alert_severity}
                        </Badge>
                      )}
                    </div>
                    <p className="font-mono text-xs text-muted-foreground truncate max-w-lg">{sq.rql}</p>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="text-xs text-muted-foreground">{formatNumber(sq.run_count)} runs</span>
                    <span className="text-xs text-muted-foreground">Last: {relativeTime(sq.last_run_at)}</span>
                    <Button variant="outline" size="sm" onClick={() => setQuery(sq.rql)}>
                      Load
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </DataState>
        </CardContent>
      </Card>
    </div>
  );
}

