"use client";

import { AlertTriangle, Server, ShieldCheck, Activity } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { DataState } from "@/components/ui/data-state";
import { SecurityScore } from "@/components/dashboard/security-score";
import { AlertFeed } from "@/components/dashboard/alert-feed";
import { useApiData } from "@/hooks/use-api-data";
import { api } from "@/lib/api";

interface DashboardStats {
  active_alerts: number;
  critical_alerts: number;
  alerts_last_24h: number;
  agents_total: number;
  agents_online: number;
  threats_blocked: number;
  threats_blocked_today: number;
  open_incidents: number;
  compliance_score: number;
  generated_at: string;
}

interface DashboardSecurityScore {
  score: number;
  grade: string;
  components: Array<{ name: string; score: number; weight: number; detail: string }>;
  generated_at: string;
}

export default function DashboardPage() {
  const stats = useApiData<DashboardStats>(() => api.dashboard.stats());
  const securityScore = useApiData<DashboardSecurityScore>(() => api.dashboard.securityScore());
  const s = stats.data;

  const statCards = [
    {
      title: "Active Alerts",
      value: s ? String(s.active_alerts) : "—",
      change: s ? `${s.alerts_last_24h} in the last 24h` : "Loading",
      icon: AlertTriangle,
      color: "text-red-400",
      bgColor: "bg-red-400/10",
    },
    {
      title: "Agents Online",
      value: s ? `${s.agents_online}/${s.agents_total}` : "—",
      change: s && s.agents_total > 0 ? `${Math.round((s.agents_online / s.agents_total) * 100)}% online` : "No agents enrolled",
      icon: Server,
      color: "text-green-400",
      bgColor: "bg-green-400/10",
    },
    {
      title: "Threats Blocked",
      value: s ? String(s.threats_blocked) : "—",
      change: s ? `+${s.threats_blocked_today} today` : "Loading",
      icon: ShieldCheck,
      color: "text-blue-400",
      bgColor: "bg-blue-400/10",
    },
    {
      title: "Compliance Score",
      value: s ? `${Math.round(s.compliance_score)}%` : "—",
      change: s ? `${s.open_incidents} open incidents` : "Loading",
      icon: Activity,
      color: "text-emerald-400",
      bgColor: "bg-emerald-400/10",
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Dashboard</h2>
        <p className="text-muted-foreground">
          Security overview and real-time monitoring
        </p>
      </div>

      <DataState
        loading={stats.loading}
        error={stats.error}
        onRetry={stats.refetch}
        loadingLabel="Loading dashboard stats"
      >
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {statCards.map((stat) => (
            <Card key={stat.title} className="border-border">
              <CardContent className="p-4">
                <div className="flex items-center gap-3">
                  <div className={`rounded-lg p-2.5 ${stat.bgColor}`}>
                    <stat.icon className={`h-5 w-5 ${stat.color}`} />
                  </div>
                  <div>
                    <p className="text-sm text-muted-foreground">{stat.title}</p>
                    <p className="text-2xl font-bold text-foreground">{stat.value}</p>
                    <p className="text-xs text-muted-foreground">{stat.change}</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </DataState>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <DataState
          loading={securityScore.loading}
          error={securityScore.error}
          onRetry={securityScore.refetch}
          loadingLabel="Loading security score"
        >
          <SecurityScore
            score={securityScore.data?.score ?? 0}
            grade={securityScore.data?.grade}
            components={securityScore.data?.components ?? []}
          />
        </DataState>
        <AlertFeed />
      </div>

      <Card className="border-border">
        <CardHeader>
          <CardTitle className="text-base">Threat Activity</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <div className="rounded-lg border border-border p-4">
              <p className="text-xs text-muted-foreground">Critical Alerts</p>
              <p className="mt-1 text-2xl font-bold text-red-400">{s?.critical_alerts ?? "—"}</p>
            </div>
            <div className="rounded-lg border border-border p-4">
              <p className="text-xs text-muted-foreground">Open Incidents</p>
              <p className="mt-1 text-2xl font-bold text-orange-400">{s?.open_incidents ?? "—"}</p>
            </div>
            <div className="rounded-lg border border-border p-4">
              <p className="text-xs text-muted-foreground">Last Updated</p>
              <p className="mt-1 text-sm font-medium text-foreground">
                {s?.generated_at ? new Date(s.generated_at).toLocaleString() : "—"}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}