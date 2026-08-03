"use client";

import { useState } from "react";
import { Globe, Shield, RefreshCw, Plus, X, Search } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { DataState } from "@/components/ui/data-state";
import { useApiData } from "@/hooks/use-api-data";
import { api } from "@/lib/api";

interface FeedRow {
  id: string;
  name: string;
  enabled: boolean;
  last_sync: string | null;
  indicator_count: number;
  status: string;
}

interface IocRow {
  id: string;
  ioc_type: string;
  value: string;
  source: string;
  severity: string;
  confidence: number;
  tags: string[];
  first_seen: string;
  last_seen: string;
}

const typeColor: Record<string, string> = {
  ip: "bg-blue-500/20 text-blue-400",
  domain: "bg-purple-500/20 text-purple-400",
  hash: "bg-orange-500/20 text-orange-400",
  url: "bg-cyan-500/20 text-cyan-400",
};

export default function ThreatIntelPage() {
  const { data: feeds, loading: feedLoading, error: feedError, refetch: refetchFeeds } = useApiData<FeedRow[]>(() => api.threatIntel.feeds());
  const { data: iocs, loading: iocLoading, error: iocError, refetch: refetchIocs } = useApiData<IocRow[]>(() => api.threatIntel.iocs());

  const [search, setSearch] = useState("");
  const [showAdd, setShowAdd] = useState(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [form, setForm] = useState({ ioc_type: "ip", value: "", severity: "medium", tags: "" });

  const feedList = feeds ?? [];
  const iocList = (iocs ?? []).filter((i) =>
    search ? i.value.toLowerCase().includes(search.toLowerCase()) || i.ioc_type.toLowerCase().includes(search.toLowerCase()) : true
  );

  const syncFeeds = async () => {
    setBusy(true);
    try {
      await api.threatIntel.syncFeeds();
      window.alert("Feed sync started");
    } catch (err) {
      window.alert(err instanceof Error ? err.message : "Sync failed");
    } finally {
      setBusy(false);
    }
  };

  const addIoc = async () => {
    setBusy(true);
    setMessage(null);
    try {
      await api.threatIntel.addIoc({
        ioc_type: form.ioc_type,
        value: form.value,
        severity: form.severity,
        tags: form.tags.split(",").map((t) => t.trim()).filter(Boolean),
      });
      setShowAdd(false);
      setForm({ ioc_type: "ip", value: "", severity: "medium", tags: "" });
      refetchIocs();
    } catch (err) {
      setMessage(err instanceof Error ? err.message : "Failed to add IOC");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Threat Intelligence</h2>
          <p className="text-muted-foreground">IOC feeds and threat indicators</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={syncFeeds} disabled={busy}>
            <RefreshCw className="mr-2 h-4 w-4" aria-hidden="true" /> Sync Feeds
          </Button>
          <Button onClick={() => setShowAdd((v) => !v)} className="gap-2">
            <Plus className="h-4 w-4" aria-hidden="true" /> Add IOC
          </Button>
        </div>
      </div>

      {/* Feeds */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <Globe className="h-4 w-4 text-muted-foreground" aria-hidden="true" /> Intelligence Feeds
          </CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <div className="grid grid-cols-1 gap-4 p-4 sm:grid-cols-2 lg:grid-cols-5">
            {feedList.map((f) => (
              <div key={f.id} className="rounded-lg border border-border p-3">
                <p className="text-sm font-medium">{f.name}</p>
                <p className="text-xs text-muted-foreground mb-2">{f.indicator_count.toLocaleString()} indicators</p>
                <Badge variant={f.status === "active" ? "default" : "outline"} className="capitalize">{f.status}</Badge>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {showAdd && (
        <Card className="border-border border-primary/30">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center justify-between text-base">
              Add Threat Indicator
              <Button variant="ghost" size="icon" onClick={() => setShowAdd(false)} aria-label="Close">
                <X className="h-4 w-4" aria-hidden="true" />
              </Button>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-4">
              <div className="space-y-2">
                <Label htmlFor="ioc-type">Type</Label>
                <Select value={form.ioc_type} onValueChange={(v) => setForm({ ...form, ioc_type: v })}>
                  <SelectTrigger id="ioc-type" className="w-full"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="ip">IP</SelectItem>
                    <SelectItem value="domain">Domain</SelectItem>
                    <SelectItem value="hash">Hash</SelectItem>
                    <SelectItem value="url">URL</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="ioc-value">Value</Label>
                <Input id="ioc-value" placeholder="45.33.32.156" value={form.value} onChange={(e) => setForm({ ...form, value: e.target.value })} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="ioc-sev">Severity</Label>
                <Select value={form.severity} onValueChange={(v) => setForm({ ...form, severity: v })}>
                  <SelectTrigger id="ioc-sev" className="w-full"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="critical">Critical</SelectItem>
                    <SelectItem value="high">High</SelectItem>
                    <SelectItem value="medium">Medium</SelectItem>
                    <SelectItem value="low">Low</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="ioc-tags">Tags (comma-separated)</Label>
                <Input id="ioc-tags" placeholder="c2, malware" value={form.tags} onChange={(e) => setForm({ ...form, tags: e.target.value })} />
              </div>
            </div>
            {message && <p className="text-sm text-red-400">{message}</p>}
            <div className="flex gap-2">
              <Button onClick={addIoc} size="sm" disabled={busy || !form.value}>
                {busy ? "Adding..." : "Add IOC"}
              </Button>
              <Button variant="outline" size="sm" onClick={() => setShowAdd(false)}>Cancel</Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Search + IOC list */}
      <div className="relative max-w-sm">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
        <Input placeholder="Search indicators..." value={search} onChange={(e) => setSearch(e.target.value)} className="pl-9" />
      </div>

      <DataState
        loading={iocLoading}
        error={iocError}
        isEmpty={iocList.length === 0}
        onRetry={refetchIocs}
        loadingLabel="Loading indicators"
        emptyTitle="No indicators"
        emptyDescription="Add threat indicators to start tracking them."
      >
        <Card className="border-border">
          <CardContent className="p-0">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-border bg-muted/30 text-left">
                    <th className="px-4 py-3 font-medium text-muted-foreground">Value</th>
                    <th className="px-4 py-3 font-medium text-muted-foreground">Type</th>
                    <th className="px-4 py-3 font-medium text-muted-foreground">Severity</th>
                    <th className="px-4 py-3 font-medium text-muted-foreground">Source</th>
                    <th className="px-4 py-3 font-medium text-muted-foreground">Last Seen</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {iocList.map((ioc) => (
                    <tr key={ioc.id} className="hover:bg-accent/50">
                      <td className="px-4 py-3">
                        <div className="font-mono text-xs">{ioc.value}</div>
                        {ioc.tags.length > 0 && (
                          <div className="flex flex-wrap gap-1 mt-1">
                            {ioc.tags.map((t) => (
                              <span key={t} className="text-[10px] rounded bg-muted px-1.5 py-0.5 text-muted-foreground">{t}</span>
                            ))}
                          </div>
                        )}
                      </td>
                      <td className="px-4 py-3">
                        <span className={`inline-block rounded px-2 py-0.5 text-xs ${typeColor[ioc.ioc_type] ?? "bg-muted text-muted-foreground"}`}>{ioc.ioc_type}</span>
                      </td>
                      <td className="px-4 py-3">
                        <Badge variant={ioc.severity as "critical" | "high" | "medium" | "low"} className="capitalize">{ioc.severity}</Badge>
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">{ioc.source}</td>
                      <td className="px-4 py-3 text-xs text-muted-foreground">{new Date(ioc.last_seen).toLocaleString()}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>
      </DataState>
    </div>
  );
}

