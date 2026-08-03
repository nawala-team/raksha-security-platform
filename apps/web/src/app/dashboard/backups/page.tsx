"use client";

import {
  HardDrive, CheckCircle2, AlertTriangle, XCircle, Shield, RefreshCw, Database,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { DataState } from "@/components/ui/data-state";
import { useApiData, useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { formatBytes, formatNumber, relativeTime } from "@/lib/utils";

/** Mirrors the portal's `BackupJobResponse`. */
interface BackupJob {
  id: string;
  name: string;
  description: string | null;
  backup_type: string;
  target_kind: string;
  source_ref: string;
  destination: string;
  destination_path: string | null;
  server_id: string | null;
  is_enabled: boolean;
  schedule_interval_mins: number | null;
  retention_days: number;
  encryption_enabled: boolean;
  encryption_algo: string | null;
  verify_after_backup: boolean;
  last_status: string | null;
  last_run_at: string | null;
  next_run_at: string | null;
  last_size_bytes: number | null;
  success_count: number;
  failure_count: number;
  created_at: string;
}

/** Mirrors the portal's `BackupRunResponse`. */
interface BackupRun {
  id: string;
  job_id: string;
  trigger: string;
  status: string;
  size_bytes: number | null;
  compressed_bytes: number | null;
  file_count: number | null;
  duration_secs: number | null;
  checksum: string | null;
  verified: boolean;
  verified_at: string | null;
  restore_tested: boolean;
  error_message: string | null;
  expires_at: string | null;
  started_at: string;
  completed_at: string | null;
}

/** Mirrors the portal's `BackupSummary`. */
interface BackupSummary {
  total_jobs: number;
  enabled_jobs: number;
  failing_jobs: number;
  never_run_jobs: number;
  unencrypted_jobs: number;
  total_backup_bytes: number;
  unverified_runs: number;
}

const statusConfig: Record<string, { color: string; icon: typeof CheckCircle2 }> = {
  success: { color: "text-green-400", icon: CheckCircle2 },
  running: { color: "text-blue-400", icon: RefreshCw },
  failed: { color: "text-red-400", icon: XCircle },
};

/** Percentage of `part` out of `whole`, clamped for the Progress component. */
function percent(part: number, whole: number): number {
  if (whole <= 0) return 0;
  return Math.min(100, Math.round((part / whole) * 100));
}

export default function BackupsPage() {
  const summary = useApiData<BackupSummary>(() => api.backups.summary());
  const jobs = useApiData<BackupJob[]>(() => api.backups.jobs());
  const runs = useApiList<BackupRun>(() => api.backups.runs());

  const jobList = jobs.data ?? [];

  // Runs reference their job by id; map ids to names for display.
  const jobNames = new Map(jobList.map((job) => [job.id, job.name]));

  const totalJobs = summary.data?.total_jobs ?? 0;
  const failingJobs = summary.data?.failing_jobs ?? 0;
  const neverRunJobs = summary.data?.never_run_jobs ?? 0;
  const unencryptedJobs = summary.data?.unencrypted_jobs ?? 0;

  // The backend exposes no RPO/RTO targets, so the gauges track what it does
  // report: jobs currently succeeding, and how much of the estate is encrypted.
  const healthyJobs = Math.max(totalJobs - failingJobs - neverRunJobs, 0);
  const healthPct = percent(healthyJobs, totalJobs);
  const encryptedJobs = Math.max(totalJobs - unencryptedJobs, 0);
  const encryptedPct = percent(encryptedJobs, totalJobs);

  const verifiedRuns = runs.items.filter((run) => run.verified).length;

  const stats = [
    {
      label: "Total Jobs",
      value: formatNumber(summary.data?.total_jobs),
      icon: HardDrive,
      color: "text-blue-400",
    },
    {
      label: "Enabled",
      value: formatNumber(summary.data?.enabled_jobs),
      icon: CheckCircle2,
      color: "text-green-400",
    },
    {
      label: "Never Run",
      value: formatNumber(summary.data?.never_run_jobs),
      icon: AlertTriangle,
      color: "text-yellow-400",
    },
    {
      label: "Failing",
      value: formatNumber(summary.data?.failing_jobs),
      icon: XCircle,
      color: "text-red-400",
    },
  ];

  const refreshAll = () => {
    summary.refetch();
    jobs.refetch();
    runs.refetch();
  };

  const refreshing = summary.loading || jobs.loading || runs.loading;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Backup Management</h2>
          <p className="text-muted-foreground">Monitor backup health, retention and verification</p>
        </div>
        <div className="flex items-center gap-3">
          <Badge variant="default" className="text-sm">
            Stored: {formatBytes(summary.data?.total_backup_bytes)}
          </Badge>
          <Button onClick={refreshAll} disabled={refreshing} className="gap-2">
            <RefreshCw className={`h-4 w-4 ${refreshing ? "animate-spin" : ""}`} aria-hidden="true" />
            {refreshing ? "Refreshing..." : "Refresh"}
          </Button>
        </div>
      </div>

      <DataState
        loading={summary.loading}
        error={summary.error}
        onRetry={summary.refetch}
        loadingLabel="Loading backup summary"
      >
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {stats.map((stat) => (
            <Card key={stat.label} className="border-border">
              <CardContent className="p-4">
                <div className="flex items-center gap-3">
                  <stat.icon className={`h-8 w-8 ${stat.color}`} aria-hidden="true" />
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

      {/* Job Health / Encryption Coverage */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Card className="border-border">
          <CardHeader className="pb-3">
            <CardTitle className="text-lg">Job Health</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">
                {formatNumber(totalJobs)} configured jobs
              </span>
              <span className="text-sm font-medium text-green-400">{healthPct}% healthy</span>
            </div>
            <Progress value={healthPct} className="h-3" />
            <p className="text-xs text-muted-foreground">
              {formatNumber(healthyJobs)} of {formatNumber(totalJobs)} jobs last completed without failure
            </p>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center gap-2 text-lg">
              <Shield className="h-5 w-5 text-blue-400" aria-hidden="true" />
              Encryption Coverage
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">
                {formatNumber(unencryptedJobs)} unencrypted
              </span>
              <span className="text-sm font-medium text-green-400">{encryptedPct}% encrypted</span>
            </div>
            <Progress value={encryptedPct} className="h-3" />
            <p className="text-xs text-muted-foreground">
              {formatNumber(summary.data?.unverified_runs)} successful runs still awaiting verification
            </p>
          </CardContent>
        </Card>

      {/* Backup Jobs */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Database className="h-5 w-5 text-blue-400" aria-hidden="true" />
            Backup Jobs
          </CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <DataState
            loading={jobs.loading}
            error={jobs.error}
            isEmpty={jobList.length === 0}
            onRetry={jobs.refetch}
            loadingLabel="Loading backup jobs"
            emptyTitle="No backup jobs configured"
            emptyDescription="Jobs defining what gets backed up, where and how often appear here."
          >
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <caption className="sr-only">
                  Configured backup jobs with source, destination, retention and last run status.
                </caption>
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Job</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Source</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Destination</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Retention</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Last Run</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Last Size</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                  </tr>
                </thead>
                <tbody>
                  {jobList.map((job) => {
                    const cfg = job.last_status ? statusConfig[job.last_status] : undefined;
                    const StatusIcon = cfg?.icon;
                    return (
                      <tr key={job.id} className="border-b border-border transition-colors hover:bg-muted/20">
                        <td className="px-4 py-3">
                          <p className="font-medium text-foreground">{job.name}</p>
                          <p className="text-xs text-muted-foreground">
                            {job.backup_type} • {job.target_kind}
                            {job.encryption_enabled
                              ? ` • ${job.encryption_algo ?? "encrypted"}`
                              : " • unencrypted"}
                          </p>
                        </td>
                        <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{job.source_ref}</td>
                        <td className="px-4 py-3 text-muted-foreground">
                          {job.destination}
                          {job.destination_path && (
                            <p className="font-mono text-xs text-muted-foreground">{job.destination_path}</p>
                          )}
                        </td>
                        <td className="px-4 py-3 text-muted-foreground">
                          {formatNumber(job.retention_days)}d
                        </td>
                        <td className="px-4 py-3 text-xs text-muted-foreground">
                          {relativeTime(job.last_run_at)}
                        </td>
                        <td className="px-4 py-3 text-muted-foreground">{formatBytes(job.last_size_bytes)}</td>
                        <td className="px-4 py-3">
                          {job.last_status ? (
                            <span className={`flex items-center gap-1.5 text-xs font-medium ${cfg?.color ?? "text-muted-foreground"}`}>
                              {StatusIcon && <StatusIcon className="h-3.5 w-3.5" aria-hidden="true" />}
                              {job.last_status}
                            </span>
                          ) : (
                            <span className="text-xs text-muted-foreground">never run</span>
                          )}
                          {!job.is_enabled && (
                            <Badge variant="secondary" className="mt-1 text-xs">paused</Badge>
                          )}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </DataState>
        </CardContent>
      </Card>

      </div>


      {/* Recent Runs */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2 text-lg">
              <HardDrive className="h-5 w-5 text-blue-400" aria-hidden="true" />
              Recent Runs
            </CardTitle>
            <span className="text-xs text-muted-foreground">
              {formatNumber(verifiedRuns)} of {formatNumber(runs.items.length)} shown runs verified
            </span>
          </div>
        </CardHeader>
        <CardContent className="p-0">
          <DataState
            loading={runs.loading}
            error={runs.error}
            isEmpty={runs.items.length === 0}
            onRetry={runs.refetch}
            loadingLabel="Loading backup runs"
            emptyTitle="No backup runs yet"
            emptyDescription="Completed and in-flight backup runs appear here."
          >
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <caption className="sr-only">
                  Backup run history with size, compressed size, duration and verification state.
                </caption>
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Job</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Trigger</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Started</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Size</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Compressed</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Duration</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Verified</th>
                  </tr>
                </thead>
                <tbody>
                  {runs.items.map((run) => {
                    const cfg = statusConfig[run.status];
                    const StatusIcon = cfg?.icon;
                    return (
                      <tr key={run.id} className="border-b border-border transition-colors hover:bg-muted/20">
                        <td className="px-4 py-3 font-medium text-foreground">
                          {jobNames.get(run.job_id) ?? "—"}
                          {run.error_message && (
                            <p className="font-mono text-xs text-red-400">{run.error_message}</p>
                          )}
                        </td>
                        <td className="px-4 py-3 text-muted-foreground">{run.trigger}</td>
                        <td className="px-4 py-3 text-xs text-muted-foreground">{relativeTime(run.started_at)}</td>
                        <td className="px-4 py-3 text-muted-foreground">{formatBytes(run.size_bytes)}</td>
                        <td className="px-4 py-3 text-muted-foreground">{formatBytes(run.compressed_bytes)}</td>
                        <td className="px-4 py-3 text-muted-foreground">
                          {run.duration_secs === null ? "—" : `${formatNumber(run.duration_secs)}s`}
                        </td>
                        <td className="px-4 py-3">
                          <span className={`flex items-center gap-1.5 text-xs font-medium ${cfg?.color ?? "text-muted-foreground"}`}>
                            {StatusIcon && <StatusIcon className="h-3.5 w-3.5" aria-hidden="true" />}
                            {run.status}
                          </span>
                        </td>
                        <td className="px-4 py-3">
                          {run.verified ? (
                            <Badge variant="outline" className="text-xs">
                              {relativeTime(run.verified_at)}
                            </Badge>
                          ) : (
                            <Badge variant="secondary" className="text-xs">unverified</Badge>
                          )}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </DataState>
        </CardContent>
      </Card>
    </div>
  );
}

