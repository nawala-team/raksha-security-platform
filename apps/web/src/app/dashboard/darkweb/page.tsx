"use client";

import { useState } from "react";
import {
  Globe, Search, Trash2, Plus, RefreshCw, Eye, ShieldAlert, Clock, AlertTriangle,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

interface Leak {
  id: string;
  email: string;
  source: string;
  dateFound: string;
  dataTypes: string[];
  severity: "critical" | "high" | "medium" | "low";
}

interface WatchlistItem {
  id: string;
  value: string;
  type: "domain" | "email";
  lastChecked: string;
}

const mockLeaks: Leak[] = [
  { id: "1", email: "admin@company.com", source: "BreachDB-2024", dateFound: "2024-01-14", dataTypes: ["password", "email"], severity: "critical" },
  { id: "2", email: "dev@company.com", source: "DarkForum Paste", dateFound: "2024-01-12", dataTypes: ["email", "username"], severity: "high" },
  { id: "3", email: "*.company.com", source: "Stealer Logs", dateFound: "2024-01-10", dataTypes: ["cookies", "credentials"], severity: "critical" },
  { id: "4", email: "hr@company.com", source: "Combo List v7", dateFound: "2024-01-08", dataTypes: ["password"], severity: "medium" },
  { id: "5", email: "support@company.com", source: "Telegram Channel", dateFound: "2024-01-05", dataTypes: ["email"], severity: "low" },
  { id: "6", email: "ceo@company.com", source: "RaidForums Archive", dateFound: "2024-01-03", dataTypes: ["password", "phone", "address"], severity: "critical" },
];

const mockWatchlist: WatchlistItem[] = [
  { id: "1", value: "company.com", type: "domain", lastChecked: "2 min ago" },
  { id: "2", value: "admin@company.com", type: "email", lastChecked: "2 min ago" },
  { id: "3", value: "ceo@company.com", type: "email", lastChecked: "5 min ago" },
  { id: "4", value: "partner.io", type: "domain", lastChecked: "1 hour ago" },
];

const stats = [
  { label: "Domains Monitored", value: "4", icon: Globe, color: "text-blue-400" },
  { label: "Leaks Found", value: "23", icon: ShieldAlert, color: "text-red-400" },
  { label: "Credentials Exposed", value: "12", icon: Eye, color: "text-orange-400" },
  { label: "Last Scan", value: "2 min ago", icon: Clock, color: "text-green-400" },
];

export default function DarkWebPage() {
  const [watchlist, setWatchlist] = useState(mockWatchlist);
  const [newEntry, setNewEntry] = useState("");
  const [scanning, setScanning] = useState(false);

  const handleAddEntry = () => {
    if (!newEntry.trim()) return;
    const isEmail = newEntry.includes("@");
    setWatchlist((prev) => [
      ...prev,
      { id: Date.now().toString(), value: newEntry.trim(), type: isEmail ? "email" : "domain", lastChecked: "Never" },
    ]);
    setNewEntry("");
  };

  const handleRemove = (id: string) => {
    setWatchlist((prev) => prev.filter((item) => item.id !== id));
  };

  const handleScan = () => {
    setScanning(true);
    setTimeout(() => setScanning(false), 2000);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Dark Web Monitoring</h2>
          <p className="text-muted-foreground">Monitor for leaked credentials and data exposure</p>
        </div>
        <Button onClick={handleScan} disabled={scanning} className="gap-2">
          <RefreshCw className={`h-4 w-4 ${scanning ? "animate-spin" : ""}`} />
          {scanning ? "Scanning..." : "Check Now"}
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.label} className="border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <stat.icon className={`h-8 w-8 ${stat.color}`} />
                <div>
                  <p className="text-2xl font-bold text-foreground">{stat.value}</p>
                  <p className="text-xs text-muted-foreground">{stat.label}</p>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <AlertTriangle className="h-5 w-5 text-red-400" />
            Discovered Leaks
          </CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full text-sm" role="table">
              <thead>
                <tr className="border-b border-border bg-muted/30">
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Email / Domain</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Source</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Date Found</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Data Types</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Severity</th>
                </tr>
              </thead>
              <tbody>
                {mockLeaks.map((leak) => (
                  <tr key={leak.id} className="border-b border-border transition-colors hover:bg-muted/20">
                    <td className="px-4 py-3 font-mono text-xs text-foreground">{leak.email}</td>
                    <td className="px-4 py-3 text-muted-foreground">{leak.source}</td>
                    <td className="px-4 py-3 text-muted-foreground">{leak.dateFound}</td>
                    <td className="px-4 py-3">
                      <div className="flex flex-wrap gap-1">
                        {leak.dataTypes.map((t) => (
                          <span key={t} className="rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">{t}</span>
                        ))}
                      </div>
                    </td>
                    <td className="px-4 py-3"><Badge variant={leak.severity}>{leak.severity}</Badge></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Search className="h-5 w-5 text-blue-400" />
            Watchlist
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <Input placeholder="Add domain or email to monitor..." value={newEntry} onChange={(e) => setNewEntry(e.target.value)} onKeyDown={(e) => e.key === "Enter" && handleAddEntry()} className="flex-1" />
            <Button onClick={handleAddEntry} size="sm" className="gap-1"><Plus className="h-4 w-4" /> Add</Button>
          </div>
          <div className="space-y-2">
            {watchlist.map((item) => (
              <div key={item.id} className="flex items-center justify-between rounded-lg border border-border px-4 py-2">
                <div className="flex items-center gap-3">
                  <Badge variant="outline" className="text-xs">{item.type}</Badge>
                  <span className="font-mono text-sm text-foreground">{item.value}</span>
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-xs text-muted-foreground">Checked: {item.lastChecked}</span>
                  <Button variant="outline" size="sm" onClick={() => handleRemove(item.id)} aria-label={`Remove ${item.value}`} className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive">
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
