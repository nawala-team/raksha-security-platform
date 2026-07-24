"use client";

import { useState } from "react";
import { Wifi, WifiOff, CheckCircle2, XCircle, Loader2 } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";

type SiemType = "splunk" | "elasticsearch" | "wazuh" | "graylog" | "custom";
type FormatType = "cef" | "leef" | "syslog" | "json";
type ConnectionStatus = "idle" | "testing" | "success" | "error";

interface SiemConfig {
  type: SiemType;
  host: string;
  port: string;
  token: string;
  indexName: string;
  format: FormatType;
  tlsEnabled: boolean;
  enabled: boolean;
  severities: { critical: boolean; high: boolean; medium: boolean; low: boolean; info: boolean };
}

export function SiemSettings() {
  const [config, setConfig] = useState<SiemConfig>({
    type: "splunk",
    host: "",
    port: "8088",
    token: "",
    indexName: "raksha-alerts",
    format: "json",
    tlsEnabled: true,
    enabled: false,
    severities: { critical: true, high: true, medium: true, low: false, info: false },
  });
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>("idle");

  const handleTestConnection = async () => {
    setConnectionStatus("testing");
    // Simulate connection test
    await new Promise((resolve) => setTimeout(resolve, 2000));
    setConnectionStatus(config.host ? "success" : "error");
    setTimeout(() => setConnectionStatus("idle"), 5000);
  };

  const updateConfig = (partial: Partial<SiemConfig>) => {
    setConfig((prev) => ({ ...prev, ...partial }));
  };

  const updateSeverity = (key: keyof SiemConfig["severities"], val: boolean) => {
    setConfig((prev) => ({ ...prev, severities: { ...prev.severities, [key]: val } }));
  };

  const defaultPorts: Record<SiemType, string> = {
    splunk: "8088", elasticsearch: "9200", wazuh: "1514", graylog: "12201", custom: "514",
  };

  return (
    <Card className="border-border">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2 text-base">
              {config.enabled ? <Wifi className="h-5 w-5 text-green-400" /> : <WifiOff className="h-5 w-5 text-muted-foreground" />}
              SIEM Integration
            </CardTitle>
            <CardDescription>Forward security events to your SIEM platform</CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Label htmlFor="siem-enabled" className="text-sm">Enable</Label>
            <button
              id="siem-enabled"
              role="switch"
              aria-checked={config.enabled}
              onClick={() => updateConfig({ enabled: !config.enabled })}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${config.enabled ? "bg-primary" : "bg-muted"}`}
            >
              <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${config.enabled ? "translate-x-6" : "translate-x-1"}`} />
            </button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* SIEM Type */}
        <div className="space-y-2">
          <Label htmlFor="siem-type">SIEM Platform</Label>
          <Select value={config.type} onValueChange={(v: SiemType) => { updateConfig({ type: v, port: defaultPorts[v] }); }}>
            <SelectTrigger id="siem-type" aria-label="Select SIEM platform">
              <SelectValue placeholder="Select SIEM type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="splunk">Splunk</SelectItem>
              <SelectItem value="elasticsearch">Elasticsearch</SelectItem>
              <SelectItem value="wazuh">Wazuh</SelectItem>
              <SelectItem value="graylog">Graylog</SelectItem>
              <SelectItem value="custom">Custom</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* Connection Config */}
        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="siem-host">Host</Label>
            <Input id="siem-host" placeholder="siem.example.com" value={config.host} onChange={(e) => updateConfig({ host: e.target.value })} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="siem-port">Port</Label>
            <Input id="siem-port" type="number" value={config.port} onChange={(e) => updateConfig({ port: e.target.value })} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="siem-token">API Token / Key</Label>
            <Input id="siem-token" type="password" placeholder="Enter token or API key" value={config.token} onChange={(e) => updateConfig({ token: e.target.value })} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="siem-index">Index Name</Label>
            <Input id="siem-index" placeholder="raksha-alerts" value={config.indexName} onChange={(e) => updateConfig({ indexName: e.target.value })} />
          </div>
        </div>

        {/* Format & TLS */}
        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="siem-format">Log Format</Label>
            <Select value={config.format} onValueChange={(v: FormatType) => updateConfig({ format: v })}>
              <SelectTrigger id="siem-format" aria-label="Select log format">
                <SelectValue placeholder="Select format" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="cef">CEF (Common Event Format)</SelectItem>
                <SelectItem value="leef">LEEF (Log Event Extended Format)</SelectItem>
                <SelectItem value="syslog">Syslog</SelectItem>
                <SelectItem value="json">JSON</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex items-end gap-3 pb-1">
            <Checkbox id="siem-tls" checked={config.tlsEnabled} onCheckedChange={(checked) => updateConfig({ tlsEnabled: !!checked })} />
            <Label htmlFor="siem-tls" className="cursor-pointer">Enable TLS encryption</Label>
          </div>
        </div>

        {/* Event Severity Filter */}
        <div className="space-y-3">
          <Label>Event Severity Filter</Label>
          <p className="text-xs text-muted-foreground">Select which alert severities to forward to SIEM</p>
          <div className="flex flex-wrap gap-4">
            {(Object.keys(config.severities) as Array<keyof typeof config.severities>).map((sev) => (
              <div key={sev} className="flex items-center gap-2">
                <Checkbox id={`siem-sev-${sev}`} checked={config.severities[sev]} onCheckedChange={(checked) => updateSeverity(sev, !!checked)} />
                <Label htmlFor={`siem-sev-${sev}`} className="cursor-pointer capitalize text-sm">{sev}</Label>
              </div>
            ))}
          </div>
        </div>

        {/* Test Connection & Save */}
        <div className="flex items-center gap-3 pt-2">
          <Button onClick={handleTestConnection} variant="outline" disabled={connectionStatus === "testing" || !config.host}>
            {connectionStatus === "testing" && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Test Connection
          </Button>
          {connectionStatus === "success" && (
            <Badge variant="low" className="gap-1"><CheckCircle2 className="h-3.5 w-3.5" />Connected</Badge>
          )}
          {connectionStatus === "error" && (
            <Badge variant="critical" className="gap-1"><XCircle className="h-3.5 w-3.5" />Connection Failed</Badge>
          )}
          <Button className="ml-auto">Save SIEM Settings</Button>
        </div>
      </CardContent>
    </Card>
  );
}
