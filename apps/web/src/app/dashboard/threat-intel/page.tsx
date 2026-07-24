"use client";

import { useState } from "react";
import { Globe, Shield, RefreshCw } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

const mockFeeds = [
  { id: "1", name: "AlienVault OTX", provider: "AT&T", lastSync: "5 min ago", indicators: 45230, status: "active" },
  { id: "2", name: "Abuse.ch URLhaus", provider: "abuse.ch", lastSync: "12 min ago", indicators: 18920, status: "active" },
  { id: "3", name: "CISA KEV", provider: "CISA", lastSync: "1 hour ago", indicators: 1124, status: "active" },
  { id: "4", name: "MITRE ATT&CK", provider: "MITRE", lastSync: "6 hours ago", indicators: 890, status: "active" },
  { id: "5", name: "NVD", provider: "NIST", lastSync: "2 days ago", indicators: 224500, status: "stale" },
];

const mockIOCs = [
  { id: "1", type: "ip", value: "45.33.32.156", source: "OTX", severity: "critical", lastSeen: "2026-07-24", tags: ["c2", "cobalt-strike"] },
  { id: "2", type: "domain", value: "evil-payload.example.net", source: "URLhaus", severity: "high", lastSeen: "2026-07-24", tags: ["malware"] },
  { id: "3", type: "hash", value: "a1b2c3d4...deadbeef", source: "OTX", severity: "critical", lastSeen: "2026-07-23", tags: ["ransomware"] },
  { id: "4", type: "ip", value: "198.51.100.23", source: "Abuse.ch", severity: "high", lastSeen: "2026-07-24", tags: ["brute-force"] },
  { id: "5", type: "url", value: "http://203.0.113.50/payload.exe", source: "URLhaus", severity: "medium", lastSeen: "2026-07-22", tags: ["dropper"] },
];

const statusColor: Record<string, string> = { active: "text-green-400", stale: "text-yellow-400", error: "text-red-400" };
const typeColor: Record<string, string> = { ip: "bg-blue-500/20 text-blue-400", domain: "bg-purple-500/20 text-purple-400", hash: "bg-orange-500/20 text-orange-400", url: "bg-cyan-500/20 text-cyan-400" };

export default function ThreatIntelPage() {
  const [search, setSearch] = useState("");
  const filtered = mockIOCs.filter((ioc) =>
    search ? ioc.value.includes(search) || ioc.tags.some((t) => t.includes(search)) : true
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Threat Intelligence</h2>
          <p className="text-muted-foreground">IOC feeds and threat indicators</p>
        </div>
        <Button variant="outline" size="sm" className="gap-2">
          <RefreshCw className="h-4 w-4" /> Sync Feeds
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-5">
        {mockFeeds.map((feed) => (
          <Card key={feed.id} className="border-border">
            <CardContent className="p-4">
              <div className="flex items-center justify-between mb-2">
                <Globe className="h-4 w-4 text-primary" />
                <span className={`text-xs font-medium ${statusColor[feed.status]}`}>
                  {feed.status}
                </span>
              </div>
              <p className="text-sm font-medium truncate">{feed.name}</p>
              <p className="text-xs text-muted-foreground">{feed.provider}</p>
              <div className="mt-2 flex justify-between text-xs text-muted-foreground">
                <span>{feed.indicators.toLocaleString()} IOCs</span>
                <span>{feed.lastSync}</span>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Card className="border-border">
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2 text-base">
              <Shield className="h-5 w-5 text-primary" /> Indicators of Compromise
            </CardTitle>
            <Input
              placeholder="Search IOCs..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-64"
            />
          </div>
        </CardHeader>
        <CardContent>
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border text-left">
                <th className="pb-2 text-muted-foreground">Type</th>
                <th className="pb-2 text-muted-foreground">Value</th>
                <th className="pb-2 text-muted-foreground">Source</th>
                <th className="pb-2 text-muted-foreground">Severity</th>
                <th className="pb-2 text-muted-foreground">Tags</th>
                <th className="pb-2 text-muted-foreground">Last Seen</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {filtered.map((ioc) => (
                <tr key={ioc.id} className="hover:bg-accent/50">
                  <td className="py-2">
                    <span className={`px-2 py-0.5 rounded text-xs ${typeColor[ioc.type]}`}>
                      {ioc.type.toUpperCase()}
                    </span>
                  </td>
                  <td className="py-2 font-mono text-xs">{ioc.value}</td>
                  <td className="py-2 text-xs text-muted-foreground">{ioc.source}</td>
                  <td className="py-2">
                    <Badge variant={ioc.severity === "critical" ? "destructive" : "default"}>
                      {ioc.severity}
                    </Badge>
                  </td>
                  <td className="py-2">
                    <div className="flex gap-1">
                      {ioc.tags.map((t) => (
                        <span key={t} className="px-1.5 py-0.5 rounded bg-muted text-[10px]">
                          {t}
                        </span>
                      ))}
                    </div>
                  </td>
                  <td className="py-2 text-xs text-muted-foreground">{ioc.lastSeen}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </CardContent>
      </Card>
    </div>
  );
}

