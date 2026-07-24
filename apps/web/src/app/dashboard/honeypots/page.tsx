"use client";

import { useState } from "react";
import {
  Bug, Wifi, Server, Mail, Database, Plus, Activity, Users, AlertTriangle, Clock,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

interface Honeypot {
  id: string;
  name: string;
  type: "SSH" | "HTTP" | "SMTP" | "MySQL";
  port: number;
  status: "Active" | "Stopped";
  interactions: number;
}

interface Interaction {
  id: string;
  time: string;
  sourceIp: string;
  honeypot: string;
  action: string;
  dataCaptured: string;
}

const mockHoneypots: Honeypot[] = [
  { id: "1", name: "ssh-trap-01", type: "SSH", port: 2222, status: "Active", interactions: 342 },
  { id: "2", name: "web-decoy-01", type: "HTTP", port: 8080, status: "Active", interactions: 128 },
  { id: "3", name: "smtp-lure-01", type: "SMTP", port: 2525, status: "Active", interactions: 56 },
  { id: "4", name: "mysql-pot-01", type: "MySQL", port: 3307, status: "Stopped", interactions: 89 },
  { id: "5", name: "ssh-trap-02", type: "SSH", port: 2223, status: "Active", interactions: 215 },
];

const mockInteractions: Interaction[] = [
  { id: "1", time: "2024-01-15 10:32:14", sourceIp: "45.33.32.156", honeypot: "ssh-trap-01", action: "Login attempt", dataCaptured: "root:admin123" },
  { id: "2", time: "2024-01-15 10:30:45", sourceIp: "198.51.100.23", honeypot: "web-decoy-01", action: "HTTP request", dataCaptured: "GET /wp-admin/" },
  { id: "3", time: "2024-01-15 10:28:11", sourceIp: "203.0.113.50", honeypot: "smtp-lure-01", action: "Relay attempt", dataCaptured: "RCPT TO: spam@test.com" },
  { id: "4", time: "2024-01-15 10:25:03", sourceIp: "45.33.32.156", honeypot: "ssh-trap-01", action: "Login attempt", dataCaptured: "admin:password" },
  { id: "5", time: "2024-01-15 10:22:58", sourceIp: "192.0.2.100", honeypot: "ssh-trap-02", action: "Login attempt", dataCaptured: "ubuntu:ubuntu" },
  { id: "6", time: "2024-01-15 10:20:30", sourceIp: "198.51.100.23", honeypot: "web-decoy-01", action: "HTTP request", dataCaptured: "POST /xmlrpc.php" },
];

const typeIcons = { SSH: Wifi, HTTP: Server, SMTP: Mail, MySQL: Database };

const stats = [
  { label: "Active Honeypots", value: "4", icon: Bug, color: "text-green-400" },
  { label: "Interactions Today", value: "87", icon: Activity, color: "text-blue-400" },
  { label: "Unique Attackers", value: "23", icon: Users, color: "text-purple-400" },
  { label: "Critical Alerts", value: "5", icon: AlertTriangle, color: "text-red-400" },
];

export default function HoneypotsPage() {
  const [showDeploy, setShowDeploy] = useState(false);
  const [deployType, setDeployType] = useState("SSH");
  const [deployPort, setDeployPort] = useState("");

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Honeypots</h2>
          <p className="text-muted-foreground">Deploy and monitor deception technology</p>
        </div>
        <Button onClick={() => setShowDeploy(!showDeploy)} className="gap-2">
          <Plus className="h-4 w-4" /> Deploy New Honeypot
        </Button>
      </div>

      {showDeploy && (
        <Card className="border-border border-primary/30">
          <CardContent className="p-4">
            <div className="flex flex-wrap items-end gap-4">
              <div className="space-y-1">
                <label className="text-xs font-medium text-muted-foreground">Type</label>
                <Select value={deployType} onValueChange={setDeployType}>
                  <SelectTrigger className="w-32"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="SSH">SSH</SelectItem>
                    <SelectItem value="HTTP">HTTP</SelectItem>
                    <SelectItem value="SMTP">SMTP</SelectItem>
                    <SelectItem value="MySQL">MySQL</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <label className="text-xs font-medium text-muted-foreground">Port</label>
                <Input placeholder="e.g. 2222" value={deployPort} onChange={(e) => setDeployPort(e.target.value)} className="w-28" />
              </div>
              <Button size="sm">Deploy</Button>
            </div>
          </CardContent>
        </Card>
      )}

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
          <CardTitle className="text-lg">Honeypot Inventory</CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full text-sm" role="table">
              <thead>
                <tr className="border-b border-border bg-muted/30">
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Name</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Type</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Port</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Interactions</th>
                </tr>
              </thead>
              <tbody>
                {mockHoneypots.map((hp) => {
                  const Icon = typeIcons[hp.type];
                  return (
                    <tr key={hp.id} className="border-b border-border hover:bg-muted/20">
                      <td className="px-4 py-3 font-medium text-foreground">{hp.name}</td>
                      <td className="px-4 py-3"><span className="flex items-center gap-1.5 text-muted-foreground"><Icon className="h-4 w-4" />{hp.type}</span></td>
                      <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{hp.port}</td>
                      <td className="px-4 py-3">
                        <span className={`inline-flex items-center gap-1 text-xs font-medium ${hp.status === "Active" ? "text-green-400" : "text-muted-foreground"}`}>
                          <span className={`h-2 w-2 rounded-full ${hp.status === "Active" ? "bg-green-400" : "bg-muted-foreground"}`} />{hp.status}
                        </span>
                      </td>
                      <td className="px-4 py-3 text-foreground">{hp.interactions}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Clock className="h-5 w-5 text-blue-400" />
            Recent Interactions
          </CardTitle>
        </CardHeader>
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full text-sm" role="table">
              <thead>
                <tr className="border-b border-border bg-muted/30">
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Time</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Source IP</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Honeypot</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Action</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Data Captured</th>
                </tr>
              </thead>
              <tbody>
                {mockInteractions.map((ix) => (
                  <tr key={ix.id} className="border-b border-border hover:bg-muted/20">
                    <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{ix.time}</td>
                    <td className="px-4 py-3 font-mono text-xs text-foreground">{ix.sourceIp}</td>
                    <td className="px-4 py-3 text-muted-foreground">{ix.honeypot}</td>
                    <td className="px-4 py-3 text-muted-foreground">{ix.action}</td>
                    <td className="px-4 py-3 font-mono text-xs text-foreground">{ix.dataCaptured}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
