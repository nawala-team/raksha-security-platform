"use client";

import { useState } from "react";
import {
  Radar,
  Globe,
  Server,
  Wifi,
  Cloud,
  AlertTriangle,
  Search,
  Plus,
  RefreshCw,
} from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DataState } from "@/components/ui/data-state";
import { useApiData } from "@/hooks/use-api-data";
import { api } from "@/lib/api";

type AssetType = "subdomain" | "service" | "port" | "cloud";
type AssetStatus = "exposed" | "internal";
type RiskLevel = "critical" | "high" | "medium" | "low";

interface Asset {
  id: string;
  domain: string;
  type: AssetType;
  status: AssetStatus;
  risk: RiskLevel;
  lastScan: string;
  details?: string;
}

// Backend field names; maps asset_type -> type, last_scan_at -> lastScan.
interface AssetBackend {
  id: string;
  domain: string;
  asset_type: string;
  status: string;
  risk: string;
  details: string | null;
  last_scan_at: string | null;
  created_at: string;
}

const typeIcons: Record<AssetType, typeof Globe> = {
  subdomain: Globe,
  service: Server,
  port: Wifi,
  cloud: Cloud,
};

export default function AttackSurfacePage() {
  const [activeTab, setActiveTab] = useState("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [showAdd, setShowAdd] = useState(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [form, setForm] = useState({ domain: "", asset_type: "subdomain", risk: "low" });

  const { data: backend, loading, error, refetch } = useApiData<AssetBackend[]>(() => api.attackSurface.list());
  const assets: Asset[] = (backend ?? []).map((b) => ({
    id: b.id,
    domain: b.domain,
    type: (["subdomain", "service", "port", "cloud"].includes(b.asset_type) ? b.asset_type : "subdomain") as AssetType,
    status: (b.status === "internal" ? "internal" : "exposed") as AssetStatus,
    risk: (b.risk as RiskLevel) || "low",
    lastScan: b.last_scan_at ?? b.created_at,
    details: b.details ?? undefined,
  }));

  const stats = {
    assets: assets.length,
    openPorts: assets.filter((a) => a.type === "port").length,
    services: assets.filter((a) => a.type === "service").length,
    subdomains: assets.filter((a) => a.type === "subdomain").length,
    critical: assets.filter((a) => a.risk === "critical").length,
  };
  const exposureScore = Math.min(100, stats.critical * 20 + stats.openPorts * 5);

  const filtered = assets.filter((asset) => {
    if (activeTab !== "all" && asset.type !== activeTab) return false;
    if (searchQuery && !asset.domain.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });

  const addAsset = async () => {
    setBusy(true);
    setMessage(null);
    try {
      await api.attackSurface.add({
        domain: form.domain,
        asset_type: form.asset_type,
        risk: form.risk,
      });
      setShowAdd(false);
      setForm({ domain: "", asset_type: "subdomain", risk: "low" });
      refetch();
    } catch (err) {
      setMessage(err instanceof Error ? err.message : "Failed to add asset");
    } finally {
      setBusy(false);
    }
  };

  const removeAsset = async (asset: Asset) => {
    if (!window.confirm(`Remove asset "${asset.domain}"?`)) return;
    try {
      await api.attackSurface.remove(asset.id);
      refetch();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : "Failed to remove asset");
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Attack Surface</h2>
          <p className="text-muted-foreground">Discover and monitor your external exposure</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={refetch} aria-label="Refresh">
            <RefreshCw className="h-4 w-4" aria-hidden="true" />
          </Button>
          <Button onClick={() => setShowAdd((v) => !v)} className="gap-2">
            <Plus className="h-4 w-4" aria-hidden="true" /> Add Asset
          </Button>
        </div>
      </div>

      {/* Exposure Score + Stats */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-6">
        <Card className={`border-border lg:col-span-2 bg-gradient-to-br ${exposureScore >= 80 ? "from-red-500/20 to-red-500/5" : exposureScore >= 60 ? "from-orange-500/20 to-orange-500/5" : exposureScore >= 40 ? "from-yellow-500/20 to-yellow-500/5" : "from-green-500/20 to-green-500/5"}`}>
          <CardContent className="p-6 flex flex-col items-center justify-center">
            <p className="text-xs uppercase tracking-wide text-muted-foreground mb-2">Exposure Score</p>
            <p className={`text-5xl font-bold ${exposureScore >= 80 ? "text-red-400" : exposureScore >= 60 ? "text-orange-400" : exposureScore >= 40 ? "text-yellow-400" : "text-green-400"}`}>{exposureScore}</p>
            <p className="text-xs text-muted-foreground mt-1">out of 100</p>
            <div className="w-full mt-3 h-2 rounded-full bg-muted overflow-hidden">
              <div className={`h-full rounded-full ${exposureScore >= 60 ? "bg-orange-500" : "bg-green-500"}`} style={{ width: `${exposureScore}%` }} />
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-blue-500/10 p-2"><Radar className="h-5 w-5 text-blue-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.assets}</p><p className="text-xs text-muted-foreground">Discovered Assets</p></div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-purple-500/10 p-2"><Wifi className="h-5 w-5 text-purple-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.openPorts}</p><p className="text-xs text-muted-foreground">Open Ports</p></div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-cyan-500/10 p-2"><Globe className="h-5 w-5 text-cyan-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.subdomains}</p><p className="text-xs text-muted-foreground">Subdomains</p></div>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className="rounded-lg bg-red-500/10 p-2"><AlertTriangle className="h-5 w-5 text-red-400" /></div>
              <div><p className="text-2xl font-bold text-foreground">{stats.critical}</p><p className="text-xs text-muted-foreground">Critical Findings</p></div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Add asset form */}
      {showAdd && (
        <Card className="border-border border-primary/30">
          <CardContent className="space-y-4 p-4">
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <div className="space-y-2">
                <label className="text-sm font-medium text-muted-foreground" htmlFor="as-domain">Domain / IP</label>
                <Input id="as-domain" placeholder="api.example.com" value={form.domain} onChange={(e) => setForm({ ...form, domain: e.target.value })} />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium text-muted-foreground" htmlFor="as-type">Type</label>
                <Tabs value={form.asset_type} onValueChange={(v) => setForm({ ...form, asset_type: v })}>
                  <TabsList>
                    <TabsTrigger value="subdomain">Subdomain</TabsTrigger>
                    <TabsTrigger value="port">Port</TabsTrigger>
                    <TabsTrigger value="service">Service</TabsTrigger>
                    <TabsTrigger value="cloud">Cloud</TabsTrigger>
                  </TabsList>
                </Tabs>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium text-muted-foreground" htmlFor="as-risk">Risk</label>
                <Tabs value={form.risk} onValueChange={(v) => setForm({ ...form, risk: v })}>
                  <TabsList>
                    <TabsTrigger value="low">Low</TabsTrigger>
                    <TabsTrigger value="medium">Medium</TabsTrigger>
                    <TabsTrigger value="high">High</TabsTrigger>
                    <TabsTrigger value="critical">Critical</TabsTrigger>
                  </TabsList>
                </Tabs>
              </div>
            </div>
            {message && <p className="text-sm text-red-400">{message}</p>}
            <div className="flex gap-2">
              <Button onClick={addAsset} disabled={busy || !form.domain}>
                {busy ? "Adding..." : "Add Asset"}
              </Button>
              <Button variant="outline" onClick={() => setShowAdd(false)}>Cancel</Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Tabs + Table */}
      <DataState
        loading={loading}
        error={error}
        isEmpty={filtered.length === 0}
        onRetry={refetch}
        loadingLabel="Loading assets"
        emptyTitle="No assets discovered"
        emptyDescription="Add assets or run a discovery scan to populate the attack surface."
      >
        <Card className="border-border">
          <CardContent className="p-4">
            <div className="flex flex-wrap items-center gap-4 mb-4">
              <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1">
                <TabsList>
                  <TabsTrigger value="all">All Assets</TabsTrigger>
                  <TabsTrigger value="subdomain">Subdomains</TabsTrigger>
                  <TabsTrigger value="port">Open Ports</TabsTrigger>
                  <TabsTrigger value="service">Services</TabsTrigger>
                  <TabsTrigger value="cloud">Cloud Assets</TabsTrigger>
                </TabsList>
              </Tabs>
              <div className="relative min-w-[200px]">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input placeholder="Search assets..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} className="pl-9" />
              </div>
            </div>

            <div className="overflow-x-auto">
              <table className="w-full text-sm" role="table">
                <thead>
                  <tr className="border-b border-border text-left">
                    <th className="pb-3 pr-4 font-medium text-muted-foreground">Domain / IP</th>
                    <th className="pb-3 pr-4 font-medium text-muted-foreground">Type</th>
                    <th className="pb-3 pr-4 font-medium text-muted-foreground">Status</th>
                    <th className="pb-3 pr-4 font-medium text-muted-foreground">Risk Level</th>
                    <th className="pb-3 pr-4 font-medium text-muted-foreground">Last Scan</th>
                    <th className="pb-3 pr-4 font-medium text-muted-foreground">Details</th>
                    <th className="pb-3 font-medium text-muted-foreground">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((asset) => {
                    const TypeIcon = typeIcons[asset.type];
                    return (
                      <tr key={asset.id} className="border-b border-border/50 hover:bg-muted/30 transition-colors">
                        <td className="py-3 pr-4">
                          <div className="flex items-center gap-2">
                            <TypeIcon className="h-4 w-4 text-muted-foreground" />
                            <span className="font-mono text-xs text-foreground">{asset.domain}</span>
                          </div>
                        </td>
                        <td className="py-3 pr-4"><Badge variant="secondary" className="capitalize text-xs">{asset.type}</Badge></td>
                        <td className="py-3 pr-4">
                          <span className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-medium ${asset.status === "exposed" ? "bg-red-500/20 text-red-400 border-red-500/30" : "bg-green-500/20 text-green-400 border-green-500/30"}`}>
                            {asset.status}
                          </span>
                        </td>
                        <td className="py-3 pr-4"><Badge variant={asset.risk}>{asset.risk}</Badge></td>
                        <td className="py-3 pr-4 text-xs text-muted-foreground">{new Date(asset.lastScan).toLocaleString()}</td>
                        <td className="py-3 pr-4 text-xs text-muted-foreground">{asset.details}</td>
                        <td className="py-3">
                          <Button variant="ghost" size="sm" onClick={() => removeAsset(asset)} className="text-red-400 hover:text-red-300" aria-label={`Remove ${asset.domain}`}>
                            <Plus className="h-4 w-4 rotate-45" aria-hidden="true" />
                          </Button>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>
      </DataState>
    </div>
  );
}
