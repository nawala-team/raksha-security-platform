"use client";

import { useState } from "react";
import {
  ShieldCheck, FileText, Filter, CheckCircle2, XCircle, AlertTriangle, BarChart3,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Progress } from "@/components/ui/progress";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";

interface Risk {
  id: string;
  name: string;
  category: string;
  likelihood: number;
  impact: number;
  score: number;
  owner: string;
  status: "open" | "mitigated" | "accepted";
}

interface Policy {
  id: string;
  name: string;
  status: "Draft" | "Active" | "Archived";
  version: string;
  acknowledgment: number;
  lastUpdated: string;
}

interface Control {
  id: string;
  name: string;
  framework: string;
  status: "Implemented" | "Partial" | "Planned" | "Not Applicable";
  owner: string;
}

const mockRisks: Risk[] = [
  { id: "1", name: "Ransomware Attack", category: "Cyber", likelihood: 4, impact: 5, score: 20, owner: "CISO", status: "open" },
  { id: "2", name: "Data Breach via Insider", category: "Insider", likelihood: 3, impact: 5, score: 15, owner: "DPO", status: "open" },
  { id: "3", name: "Third-party Vendor Compromise", category: "Supply Chain", likelihood: 3, impact: 4, score: 12, owner: "Procurement", status: "mitigated" },
  { id: "4", name: "DDoS on Public Services", category: "Cyber", likelihood: 4, impact: 3, score: 12, owner: "NetOps", status: "open" },
  { id: "5", name: "Physical Facility Breach", category: "Physical", likelihood: 2, impact: 4, score: 8, owner: "Facilities", status: "accepted" },
  { id: "6", name: "Compliance Violation (GDPR)", category: "Regulatory", likelihood: 2, impact: 5, score: 10, owner: "Legal", status: "mitigated" },
];

const mockPolicies: Policy[] = [
  { id: "1", name: "Information Security Policy", status: "Active", version: "3.2", acknowledgment: 94, lastUpdated: "2024-01-10" },
  { id: "2", name: "Acceptable Use Policy", status: "Active", version: "2.1", acknowledgment: 87, lastUpdated: "2024-01-05" },
  { id: "3", name: "Incident Response Plan", status: "Active", version: "4.0", acknowledgment: 78, lastUpdated: "2023-12-20" },
  { id: "4", name: "Data Classification Policy", status: "Draft", version: "1.0", acknowledgment: 0, lastUpdated: "2024-01-12" },
  { id: "5", name: "Remote Work Security", status: "Archived", version: "1.5", acknowledgment: 92, lastUpdated: "2023-06-15" },
];

const mockControls: Control[] = [
  { id: "1", name: "CIS 1.1 - Hardware Asset Inventory", framework: "CIS", status: "Implemented", owner: "IT Ops" },
  { id: "2", name: "CIS 2.1 - Software Asset Inventory", framework: "CIS", status: "Partial", owner: "IT Ops" },
  { id: "3", name: "NIST ID.AM-1 - Asset Management", framework: "NIST", status: "Implemented", owner: "IT Ops" },
  { id: "4", name: "NIST PR.AC-1 - Access Control", framework: "NIST", status: "Implemented", owner: "IAM" },
  { id: "5", name: "PCI 3.4 - Render PAN Unreadable", framework: "PCI", status: "Implemented", owner: "DevOps" },
  { id: "6", name: "PCI 6.1 - Patch Management", framework: "PCI", status: "Partial", owner: "IT Ops" },
  { id: "7", name: "ISO 27001 A.12.6 - Vulnerability Mgmt", framework: "ISO", status: "Planned", owner: "SecOps" },
  { id: "8", name: "NIST DE.CM-1 - Network Monitoring", framework: "NIST", status: "Implemented", owner: "SOC" },
];

const frameworkCoverage = [
  { name: "CIS Controls v8", implemented: 72, total: 100 },
  { name: "NIST CSF", implemented: 85, total: 100 },
  { name: "PCI-DSS 4.0", implemented: 68, total: 100 },
  { name: "ISO 27001:2022", implemented: 54, total: 100 },
];

const heatmapColors: Record<string, string> = {
  "1": "bg-green-900/50", "2": "bg-green-800/50", "3": "bg-yellow-700/50",
  "4": "bg-yellow-600/50", "5": "bg-orange-600/50", "6": "bg-orange-500/50",
  "8": "bg-orange-400/50", "9": "bg-red-600/50", "10": "bg-red-500/50",
  "12": "bg-red-500/60", "15": "bg-red-600/70", "16": "bg-red-700/80",
  "20": "bg-red-800/90", "25": "bg-red-900",
};

function getHeatColor(score: number): string {
  if (score <= 2) return "bg-green-900/50";
  if (score <= 4) return "bg-yellow-700/50";
  if (score <= 8) return "bg-orange-500/50";
  if (score <= 15) return "bg-red-600/70";
  return "bg-red-800/90";
}

const policyStatusColor: Record<string, "default" | "secondary" | "outline"> = {
  Active: "default", Draft: "secondary", Archived: "outline",
};

const controlStatusColor: Record<string, string> = {
  Implemented: "text-green-400", Partial: "text-yellow-400", Planned: "text-blue-400", "Not Applicable": "text-muted-foreground",
};

export default function GRCPage() {
  const [frameworkFilter, setFrameworkFilter] = useState<string>("all");

  const filteredControls = mockControls.filter((c) =>
    frameworkFilter === "all" ? true : c.framework === frameworkFilter
  );

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Governance, Risk & Compliance</h2>
        <p className="text-muted-foreground">Manage risks, policies, and control frameworks</p>
      </div>

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
                  <div className="overflow-x-auto">
                    <table className="w-full text-sm" role="table">
                      <thead>
                        <tr className="border-b border-border bg-muted/30">
                          <th className="px-4 py-3 text-left font-medium text-muted-foreground">Risk</th>
                          <th className="px-4 py-3 text-left font-medium text-muted-foreground">Category</th>
                          <th className="px-4 py-3 text-center font-medium text-muted-foreground">L</th>
                          <th className="px-4 py-3 text-center font-medium text-muted-foreground">I</th>
                          <th className="px-4 py-3 text-center font-medium text-muted-foreground">Score</th>
                          <th className="px-4 py-3 text-left font-medium text-muted-foreground">Owner</th>
                          <th className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                        </tr>
                      </thead>
                      <tbody>
                        {mockRisks.map((risk) => (
                          <tr key={risk.id} className="border-b border-border hover:bg-muted/20">
                            <td className="px-4 py-3 font-medium text-foreground">{risk.name}</td>
                            <td className="px-4 py-3 text-muted-foreground">{risk.category}</td>
                            <td className="px-4 py-3 text-center text-foreground">{risk.likelihood}</td>
                            <td className="px-4 py-3 text-center text-foreground">{risk.impact}</td>
                            <td className="px-4 py-3 text-center">
                              <span className={`inline-flex h-7 w-7 items-center justify-center rounded text-xs font-bold ${getHeatColor(risk.score)} text-foreground`}>{risk.score}</span>
                            </td>
                            <td className="px-4 py-3 text-muted-foreground">{risk.owner}</td>
                            <td className="px-4 py-3 capitalize">
                              <Badge variant={risk.status === "open" ? "high" : risk.status === "mitigated" ? "low" : "medium"}>{risk.status}</Badge>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
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
                        const score = l * i;
                        const hasRisk = mockRisks.some((r) => r.likelihood === l && r.impact === i);
                        return (
                          <div key={`${l}-${i}`} className={`flex-1 aspect-square rounded flex items-center justify-center text-xs font-medium ${getHeatColor(score)} ${hasRisk ? "ring-2 ring-white/50" : ""}`}>
                            {hasRisk ? mockRisks.filter((r) => r.likelihood === l && r.impact === i).length : ""}
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
              <div className="overflow-x-auto">
                <table className="w-full text-sm" role="table">
                  <thead>
                    <tr className="border-b border-border bg-muted/30">
                      <th className="px-4 py-3 text-left font-medium text-muted-foreground">Policy</th>
                      <th className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                      <th className="px-4 py-3 text-left font-medium text-muted-foreground">Version</th>
                      <th className="px-4 py-3 text-left font-medium text-muted-foreground">Acknowledgment</th>
                      <th className="px-4 py-3 text-left font-medium text-muted-foreground">Last Updated</th>
                    </tr>
                  </thead>
                  <tbody>
                    {mockPolicies.map((policy) => (
                      <tr key={policy.id} className="border-b border-border hover:bg-muted/20">
                        <td className="px-4 py-3 font-medium text-foreground">{policy.name}</td>
                        <td className="px-4 py-3"><Badge variant={policyStatusColor[policy.status]}>{policy.status}</Badge></td>
                        <td className="px-4 py-3 text-muted-foreground">v{policy.version}</td>
                        <td className="px-4 py-3">
                          <div className="flex items-center gap-2">
                            <Progress value={policy.acknowledgment} className="h-2 w-20" />
                            <span className="text-xs text-muted-foreground">{policy.acknowledgment}%</span>
                          </div>
                        </td>
                        <td className="px-4 py-3 text-muted-foreground">{policy.lastUpdated}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="controls" className="space-y-4">
          <Card className="border-border">
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="text-lg">Security Controls</CardTitle>
                <Select value={frameworkFilter} onValueChange={setFrameworkFilter}>
                  <SelectTrigger className="w-40"><SelectValue placeholder="Framework" /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Frameworks</SelectItem>
                    <SelectItem value="CIS">CIS</SelectItem>
                    <SelectItem value="NIST">NIST</SelectItem>
                    <SelectItem value="PCI">PCI</SelectItem>
                    <SelectItem value="ISO">ISO</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardHeader>
            <CardContent className="p-0">
              <div className="overflow-x-auto">
                <table className="w-full text-sm" role="table">
                  <thead>
                    <tr className="border-b border-border bg-muted/30">
                      <th className="px-4 py-3 text-left font-medium text-muted-foreground">Control</th>
                      <th className="px-4 py-3 text-left font-medium text-muted-foreground">Framework</th>
                      <th className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                      <th className="px-4 py-3 text-left font-medium text-muted-foreground">Owner</th>
                    </tr>
                  </thead>
                  <tbody>
                    {filteredControls.map((ctrl) => (
                      <tr key={ctrl.id} className="border-b border-border hover:bg-muted/20">
                        <td className="px-4 py-3 font-medium text-foreground">{ctrl.name}</td>
                        <td className="px-4 py-3"><Badge variant="outline">{ctrl.framework}</Badge></td>
                        <td className={`px-4 py-3 font-medium ${controlStatusColor[ctrl.status]}`}>{ctrl.status}</td>
                        <td className="px-4 py-3 text-muted-foreground">{ctrl.owner}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="coverage" className="space-y-4">
          <Card className="border-border">
            <CardHeader className="pb-3">
              <CardTitle className="text-lg">Framework Coverage</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              {frameworkCoverage.map((fw) => (
                <div key={fw.name} className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium text-foreground">{fw.name}</span>
                    <span className="text-sm text-muted-foreground">{fw.implemented}%</span>
                  </div>
                  <Progress value={fw.implemented} className="h-3" />
                </div>
              ))}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
