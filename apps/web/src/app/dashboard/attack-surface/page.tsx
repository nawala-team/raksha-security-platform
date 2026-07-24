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
  Play,
  ExternalLink,
} from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";

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

const mockAssets: Asset[] = [
  { id: "1", domain: "api.example.com", type: "subdomain", status: "exposed", risk: "high", lastScan: "2024-01-15T10:00:00Z", details: "Public API - outdated TLS 1.1" },
  { id: "2", domain: "staging.example.com", type: "subdomain", status: "exposed", risk: "critical", lastScan: "2024-01-15T10:00:00Z", details: "No auth on admin panel" },
  { id: "3", domain: "192.168.1.50:3306", type: "port", status: "internal", risk: "medium", lastScan: "2024-01-15T09:30:00Z", details: "MySQL exposed to internal VLAN" },
  { id: "4", domain: "192.168.1.100:22", type: "port", status: "internal", risk: "low", lastScan: "2024-01-15T09:30:00Z", details: "SSH with key-based auth" },
  { id: "5", domain: "10.0.0.5:8080", type: "port", status: "exposed", risk: "high", lastScan: "2024-01-15T09:30:00Z", details: "Debug endpoint publicly reachable" },
  { id: "6", domain: "mail.example.com", type: "service", status: "exposed", risk: "medium", lastScan: "2024-01-15T08:00:00Z", details: "SMTP relay - SPF misconfigured" },
  { id: "7", domain: "vpn.example.com", type: "service", status: "exposed", risk: "low", lastScan: "2024-01-15T08:00:00Z", details: "OpenVPN - up to date" },
  { id: "8", domain: "jenkins.internal.example.com", type: "service", status: "internal", risk: "high", lastScan: "2024-01-14T22:00:00Z", details: "Jenkins with known CVE" },
  { id: "9", domain: "s3://customer-data-prod", type: "cloud", status: "exposed", risk: "critical", lastScan: "2024-01-15T11:00:00Z", details: "Public bucket with PII" },
  { id: "10", domain: "s3://app-logs-staging", type: "cloud", status: "internal", risk: "low", lastScan: "2024-01-15T11:00:00Z", details: "Private, encrypted" },
  { id: "11", domain: "ec2-54-123-45-67.compute-1.amazonaws.com", type: "cloud", status: "exposed", risk: "medium", lastScan: "2024-01-15T11:00:00Z", details: "Overly permissive security group" },
  { id: "12", domain: "dev.example.com", type: "subdomain", status: "exposed", risk: "medium", lastScan: "2024-01-15T10:00:00Z", details: "Development env with test credentials" },
];

const typeIcons: Record<AssetType, typeof Globe> = {
  subdomain: Globe,
  service: Server,
  port: Wifi,
  cloud: Cloud,
};

export default function AttackSurfacePage() {
  const [activeTab, setActiveTab] = useState("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [scanning, setScanning] = useState(false);

  const exposureScore = 72;
  const stats = {
    assets: mockAssets.length,
    openPorts: mockAssets.filter((a) => a.type === "port").length,
    services: mockAssets.filter((a) => a.type === "service").length,
    subdomains: mockAssets.filter((a) => a.type === "subdomain").length,
    critical: mockAssets.filter((a) => a.risk === "critical").length,
  };

  const filtered = mockAssets.filter((asset) => {
    if (activeTab !== "all" && asset.type !== activeTab) return false;
    if (searchQuery && !asset.domain.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });

  function getScoreColor(score: number) {
    if (score >= 80) return "text-red-400";
    if (score >= 60) return "text-orange-400";
    if (score >= 40) return "text-yellow-400";
    return "text-green-400";
  }

  function getScoreBg(score: number) {
    if (score >= 80) return "from-red-500/20 to-red-500/5";
    if (score >= 60) return "from-orange-500/20 to-orange-500/5";
    if (score >= 40) return "from-yellow-500/20 to-yellow-500/5";
    return "from-green-500/20 to-green-500/5";
  }

  function handleScan() {
    setScanning(true);
    setTimeout(() => setScanning(false), 3000);
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Attack Surface</h2>
          <p className="text-muted-foreground">Discover and monitor your external exposure</p>
        </div>
        <Button onClick={handleScan} disabled={scanning}>
          <Play className="mr-2 h-4 w-4" />
          {scanning ? "Scanning..." : "Run Discovery Scan"}
        </Button>
      </div>

      {/* Exposure Score + Stats */}
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-6">
        <Card className={`border-border lg:col-span-2 bg-gradient-to-br ${getScoreBg(exposureScore)}`}>
          <CardContent className="p-6 flex flex-col items-center justify-center">
            <p className="text-xs uppercase tracking-wide text-muted-foreground mb-2">Exposure Score</p>
            <p className={`text-5xl font-bold ${getScoreColor(exposureScore)}`}>{exposureScore}</p>
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

      {/* Tabs + Table */}
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

          {/* Table */}
          <div className="overflow-x-auto">
            <table className="w-full text-sm" role="table">
              <thead>
                <tr className="border-b border-border text-left">
                  <th className="pb-3 pr-4 font-medium text-muted-foreground">Domain / IP</th>
                  <th className="pb-3 pr-4 font-medium text-muted-foreground">Type</th>
                  <th className="pb-3 pr-4 font-medium text-muted-foreground">Status</th>
                  <th className="pb-3 pr-4 font-medium text-muted-foreground">Risk Level</th>
                  <th className="pb-3 pr-4 font-medium text-muted-foreground">Last Scan</th>
                  <th className="pb-3 font-medium text-muted-foreground">Details</th>
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
                          {asset.status === "exposed" && <ExternalLink className="h-3 w-3" />}
                          {asset.status}
                        </span>
                      </td>
                      <td className="py-3 pr-4"><Badge variant={asset.risk}>{asset.risk}</Badge></td>
                      <td className="py-3 pr-4 text-xs text-muted-foreground">{new Date(asset.lastScan).toLocaleString()}</td>
                      <td className="py-3 text-xs text-muted-foreground">{asset.details}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
            {filtered.length === 0 && <p className="py-8 text-center text-muted-foreground">No assets found.</p>}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
