"use client";

import { useState } from "react";
import {
  ShieldCheck, FileText, CheckCircle2, AlertTriangle, BarChart3,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Progress } from "@/components/ui/progress";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { DataState } from "@/components/ui/data-state";
import { useApiData } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { formatNumber, relativeTime } from "@/lib/utils";

/** Mirrors the portal's `RiskResponse`. */
interface RiskRecord {
  id: string;
  title: string;
  description: string;
  category: string;
  likelihood: number;
  impact: number;
  risk_score: number;
  owner: string;
  status: string;
  mitigation_plan: string | null;
  review_date: string;
  created_at: string;
  updated_at: string;
}

/** Mirrors the portal's `PolicyResponse` (list form, without the body). */
interface PolicyRecord {
  id: string;
  title: string;
  version: string;
  status: string;
  approved_by: string | null;
  effective_date: string | null;
  review_cycle_days: number;
  created_at: string;
  updated_at: string;
}

/** Mirrors the portal's `ControlResponse`. */
interface ControlRecord {
  id: string;
  title: string;
  description: string;
  framework: string;
  control_ref: string;
  status: string;
  evidence: string | null;
  last_assessed: string | null;
  created_at: string;
}

/** Mirrors the portal's `GrcSummary`. */
interface GrcSummary {
  total_risks: number;
  high_risks: number;
  open_risks: number;
  risks_due_review: number;
  total_policies: number;
  published_policies: number;
  total_controls: number;
  implemented_controls: number;
}

/** `risk_score` is likelihood × impact, so it runs 1–25. */
function getHeatColor(score: number): string {
  if (score <= 4) return "bg-green-900/50";
  if (score <= 8) return "bg-yellow-700/50";
  if (score <= 12) return "bg-orange-500/50";
  if (score <= 19) return "bg-red-600/70";
  return "bg-red-800/90";
}

const riskStatusVariants: Record<string, "critical" | "high" | "medium" | "low" | "outline"> = {
  open: "high",
  in_progress: "medium",
  mitigated: "low",
  accepted: "medium",
  closed: "low",
};

const policyStatusVariants: Record<string, "default" | "secondary" | "outline"> = {
  published: "default",
  active: "default",
  draft: "secondary",
  archived: "outline",
  retired: "outline",
};

const controlStatusColors: Record<string, string> = {
  implemented: "text-green-400",
  partial: "text-yellow-400",
  planned: "text-blue-400",
  not_applicable: "text-muted-foreground",
  gap: "text-red-400",
};

/** Absolute date for review/effective columns, where "in 3 months" is vaguer. */
function formatDate(iso: string | null): string {
  if (!iso) return "—";
  const ms = new Date(iso).getTime();
  if (Number.isNaN(ms)) return "—";
  return new Date(ms).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

/** Coverage is derived from the control list: no dedicated endpoint exists. */
function frameworkCoverage(controls: ControlRecord[]) {
  const byFramework = new Map<string, { implemented: number; total: number }>();
  for (const control of controls) {
    const entry = byFramework.get(control.framework) ?? { implemented: 0, total: 0 };
    entry.total += 1;
    if (control.status === "implemented") entry.implemented += 1;
    byFramework.set(control.framework, entry);
  }
  return Array.from(byFramework.entries())
    .map(([name, counts]) => ({
      name,
      implemented: counts.implemented,
      total: counts.total,
      percent: counts.total === 0 ? 0 : Math.round((counts.implemented / counts.total) * 100),
    }))
    .sort((a, b) => a.name.localeCompare(b.name));
}

export default function GRCPage() {
  const [frameworkFilter, setFrameworkFilter] = useState<string>("all");

  const summary = useApiData<GrcSummary>(() => api.grc.summary());
  const risks = useApiData<RiskRecord[]>(() => api.grc.risks());
  const policies = useApiData<PolicyRecord[]>(() => api.grc.policies());
  const controls = useApiData<ControlRecord[]>(() => api.grc.controls());

  const riskRows = risks.data ?? [];
  const policyRows = policies.data ?? [];
  const controlRows = controls.data ?? [];

  const frameworks = Array.from(new Set(controlRows.map((c) => c.framework))).sort();
  const filteredControls = controlRows.filter((c) =>
    frameworkFilter === "all" ? true : c.framework === frameworkFilter
  );
  const coverage = frameworkCoverage(controlRows);

  const stats = [
    { label: "Open Risks", value: formatNumber(summary.data?.open_risks), icon: AlertTriangle, color: "text-red-400" },
    { label: "High Risks", value: formatNumber(summary.data?.high_risks), icon: BarChart3, color: "text-orange-400" },
    { label: "Due for Review", value: formatNumber(summary.data?.risks_due_review), icon: AlertTriangle, color: "text-yellow-400" },
    { label: "Published Policies", value: formatNumber(summary.data?.published_policies), icon: FileText, color: "text-blue-400" },
    { label: "Implemented Controls", value: formatNumber(summary.data?.implemented_controls), icon: CheckCircle2, color: "text-green-400" },
    { label: "Total Controls", value: formatNumber(summary.data?.total_controls), icon: ShieldCheck, color: "text-muted-foreground" },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Governance, Risk &amp; Compliance</h2>
        <p className="text-muted-foreground">Manage risks, policies, and control frameworks</p>
      </div>

      <DataState
        loading={summary.loading}
        error={summary.error}
        onRetry={summary.refetch}
        loadingLabel="Loading GRC summary"
      >
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6">
          {stats.map((stat) => (
            <Card key={stat.label} className="border-border">
              <CardContent className="p-4">
                <div className="flex items-center gap-3">
                  <stat.icon className={`h-7 w-7 shrink-0 ${stat.color}`} aria-hidden="true" />
                  <div>
                    <p className="text-2xl font-bold text-foreground">{stat.value}</p>
                    <p className="text-xs text-muted-foreground">{stat.label}</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </DataState>

      <Tabs defaultValue="risks" className="space-y-4">
        <TabsList>
          <TabsTrigger value="risks">Risk Register</TabsTrigger>
          <TabsTrigger value="policies">Policies</TabsTrigger>
          <TabsTrigger value="controls">Controls</TabsTrigger>
          <TabsTrigger value="coverage">Coverage</TabsTrigger>
        </TabsList>


        <TabsContent value="risks" className="space-y-4">
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
            <div className="lg:col-span-2">
              <Card className="border-border">
                <CardHeader className="pb-3">
                  <CardTitle className="text-lg">Risk Register</CardTitle>
                </CardHeader>
                <CardContent className="p-0">
                  <DataState
                    loading={risks.loading}
                    error={risks.error}
                    isEmpty={riskRows.length === 0}
                    onRetry={risks.refetch}
                    loadingLabel="Loading risk register"
                    emptyTitle="No risks recorded"
                    emptyDescription="Risks added to the register appear here with their likelihood and impact scores."
                  >
                    <div className="overflow-x-auto">
                      <table className="w-full text-sm">
                        <caption className="sr-only">
                          Risk register with category, likelihood, impact, score, review date and status.
                        </caption>
                        <thead>
                          <tr className="border-b border-border bg-muted/30">
                            <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Risk</th>
                            <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Category</th>
                            <th scope="col" className="px-4 py-3 text-center font-medium text-muted-foreground">L</th>
                            <th scope="col" className="px-4 py-3 text-center font-medium text-muted-foreground">I</th>
                            <th scope="col" className="px-4 py-3 text-center font-medium text-muted-foreground">Score</th>
                            <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Review Due</th>
                            <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                          </tr>
                        </thead>
                        <tbody>
                          {riskRows.map((risk) => (
                            <tr key={risk.id} className="border-b border-border hover:bg-muted/20">
                              <td className="px-4 py-3 font-medium text-foreground">{risk.title}</td>
                              <td className="px-4 py-3 text-muted-foreground">{risk.category}</td>
                              <td className="px-4 py-3 text-center text-foreground">{risk.likelihood}</td>
                              <td className="px-4 py-3 text-center text-foreground">{risk.impact}</td>
                              <td className="px-4 py-3 text-center">
                                <span className={`inline-flex h-7 w-7 items-center justify-center rounded text-xs font-bold ${getHeatColor(risk.risk_score)} text-foreground`}>{risk.risk_score}</span>
                              </td>
                              <td className="px-4 py-3 text-muted-foreground">{formatDate(risk.review_date)}</td>
                              <td className="px-4 py-3 capitalize">
                                <Badge variant={riskStatusVariants[risk.status] ?? "outline"}>{risk.status.replace(/_/g, " ")}</Badge>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </DataState>
                </CardContent>
              </Card>
            </div>

            <Card className="border-border">
              <CardHeader className="pb-3">
                <CardTitle className="text-lg">Risk Heatmap</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-1">
                  <div className="flex items-center gap-1">
                    <span className="w-8 text-xs text-muted-foreground text-right">L/I</span>
                    {[1, 2, 3, 4, 5].map((i) => (
                      <span key={i} className="flex-1 text-center text-xs text-muted-foreground">{i}</span>
                    ))}
                  </div>
                  {[5, 4, 3, 2, 1].map((l) => (
                    <div key={l} className="flex items-center gap-1">
                      <span className="w-8 text-xs text-muted-foreground text-right">{l}</span>
                      {[1, 2, 3, 4, 5].map((i) => {
                        const cellRisks = riskRows.filter((r) => r.likelihood === l && r.impact === i);
                        return (
                          <div key={`${l}-${i}`} className={`flex-1 aspect-square rounded flex items-center justify-center text-xs font-medium ${getHeatColor(l * i)} ${cellRisks.length > 0 ? "ring-2 ring-white/50" : ""}`}>
                            {cellRisks.length > 0 ? cellRisks.length : ""}
                          </div>
                        );
                      })}
                    </div>
                  ))}
                  <div className="mt-2 flex justify-between text-xs text-muted-foreground">
                    <span>Low Impact →</span><span>High Impact</span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>


        <TabsContent value="policies" className="space-y-4">
          <Card className="border-border">
            <CardHeader className="pb-3">
              <CardTitle className="text-lg">Policy Management</CardTitle>
            </CardHeader>
            <CardContent className="p-0">
              <DataState
                loading={policies.loading}
                error={policies.error}
                isEmpty={policyRows.length === 0}
                onRetry={policies.refetch}
                loadingLabel="Loading policies"
                emptyTitle="No policies yet"
                emptyDescription="Policies published to the platform appear here."
              >
                <div className="overflow-x-auto">
                  <table className="w-full text-sm">
                    <caption className="sr-only">
                      Policies with status, version, effective date, review cycle and last update.
                    </caption>
                    <thead>
                      <tr className="border-b border-border bg-muted/30">
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Policy</th>
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Version</th>
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Effective</th>
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Review Cycle</th>
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Last Updated</th>
                      </tr>
                    </thead>
                    <tbody>
                      {policyRows.map((policy) => (
                        <tr key={policy.id} className="border-b border-border hover:bg-muted/20">
                          <td className="px-4 py-3 font-medium text-foreground">{policy.title}</td>
                          <td className="px-4 py-3">
                            <Badge variant={policyStatusVariants[policy.status] ?? "outline"}>{policy.status.replace(/_/g, " ")}</Badge>
                          </td>
                          <td className="px-4 py-3 text-muted-foreground">v{policy.version}</td>
                          <td className="px-4 py-3 text-muted-foreground">{formatDate(policy.effective_date)}</td>
                          <td className="px-4 py-3 text-muted-foreground">{formatNumber(policy.review_cycle_days)} days</td>
                          <td className="px-4 py-3 text-xs text-muted-foreground">{relativeTime(policy.updated_at)}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </DataState>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="controls" className="space-y-4">
          <Card className="border-border">
            <CardHeader className="pb-3">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <CardTitle className="text-lg">Control Framework Mappings</CardTitle>
                <Select value={frameworkFilter} onValueChange={setFrameworkFilter}>
                  <SelectTrigger className="w-[200px]">
                    <SelectValue placeholder="All frameworks" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All frameworks</SelectItem>
                    {frameworks.map((framework) => (
                      <SelectItem key={framework} value={framework}>
                        {framework}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </CardHeader>
            <CardContent className="p-0">
              <DataState
                loading={controls.loading}
                error={controls.error}
                isEmpty={filteredControls.length === 0}
                onRetry={controls.refetch}
                loadingLabel="Loading controls"
                emptyTitle="No controls found"
                emptyDescription="Controls mapped to a framework appear here."
              >
                <div className="overflow-x-auto">
                  <table className="w-full text-sm">
                    <caption className="sr-only">
                      Framework controls with reference, status, evidence and last assessment date.
                    </caption>
                    <thead>
                      <tr className="border-b border-border bg-muted/30">
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Framework</th>
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Ref</th>
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Control</th>
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Evidence</th>
                        <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Last Assessed</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredControls.map((control) => (
                        <tr key={control.id} className="border-b border-border hover:bg-muted/20">
                          <td className="px-4 py-3 text-muted-foreground">{control.framework}</td>
                          <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{control.control_ref}</td>
                          <td className="px-4 py-3 font-medium text-foreground">{control.title}</td>
                          <td className={`px-4 py-3 text-xs font-medium capitalize ${controlStatusColors[control.status] ?? "text-muted-foreground"}`}>
                            {control.status.replace(/_/g, " ")}
                          </td>
                          <td className="px-4 py-3 text-xs text-muted-foreground">
                            {control.evidence ? "Attached" : "None"}
                          </td>
                          <td className="px-4 py-3 text-xs text-muted-foreground">
                            {control.last_assessed ? relativeTime(control.last_assessed) : "never"}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </DataState>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="coverage" className="space-y-4">
          <Card className="border-border">
            <CardHeader className="pb-3">
              <CardTitle className="text-lg">Framework Coverage</CardTitle>
            </CardHeader>
            <CardContent>
              <DataState
                loading={controls.loading}
                error={controls.error}
                isEmpty={coverage.length === 0}
                onRetry={controls.refetch}
                loadingLabel="Loading coverage"
                emptyTitle="No framework data"
                emptyDescription="Coverage is calculated from mapped controls."
              >
                <div className="space-y-4">
                  {coverage.map((framework) => (
                    <div key={framework.name} className="space-y-1.5">
                      <div className="flex items-center justify-between text-sm">
                        <span className="font-medium text-foreground">{framework.name}</span>
                        <span className="text-muted-foreground">
                          {formatNumber(framework.implemented)} / {formatNumber(framework.total)} implemented ({framework.percent}%)
                        </span>
                      </div>
                      <Progress value={framework.percent} className="h-2" />
                    </div>
                  ))}
                </div>
              </DataState>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}


