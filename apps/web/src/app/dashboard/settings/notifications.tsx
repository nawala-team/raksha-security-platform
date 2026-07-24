"use client";

import { useState } from "react";
import { Mail, MessageSquare, Hash, Send, Loader2, CheckCircle2, XCircle } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";

type TestStatus = "idle" | "testing" | "success" | "error";

interface EmailConfig {
  enabled: boolean;
  host: string;
  port: string;
  username: string;
  password: string;
  from: string;
  to: string;
  tlsEnabled: boolean;
  minSeverity: string;
}

interface TelegramConfig {
  enabled: boolean;
  botToken: string;
  chatIds: string;
  parseMode: string;
  minSeverity: string;
}

interface SlackConfig {
  enabled: boolean;
  webhookUrl: string;
  minSeverity: string;
}

export function NotificationSettings() {
  const [email, setEmail] = useState<EmailConfig>({
    enabled: true, host: "", port: "587", username: "", password: "",
    from: "", to: "", tlsEnabled: true, minSeverity: "high",
  });
  const [telegram, setTelegram] = useState<TelegramConfig>({
    enabled: false, botToken: "", chatIds: "", parseMode: "HTML", minSeverity: "critical",
  });
  const [slack, setSlack] = useState<SlackConfig>({
    enabled: false, webhookUrl: "", minSeverity: "medium",
  });
  const [testStatus, setTestStatus] = useState<Record<string, TestStatus>>({
    email: "idle", telegram: "idle", slack: "idle",
  });

  const handleTest = async (channel: string) => {
    setTestStatus((prev) => ({ ...prev, [channel]: "testing" }));
    await new Promise((resolve) => setTimeout(resolve, 2000));
    const success = channel === "email" ? !!email.host : channel === "telegram" ? !!telegram.botToken : !!slack.webhookUrl;
    setTestStatus((prev) => ({ ...prev, [channel]: success ? "success" : "error" }));
    setTimeout(() => setTestStatus((prev) => ({ ...prev, [channel]: "idle" })), 5000);
  };

  const TestButton = ({ channel }: { channel: string }) => (
    <div className="flex items-center gap-2">
      <Button variant="outline" size="sm" onClick={() => handleTest(channel)} disabled={testStatus[channel] === "testing"}>
        {testStatus[channel] === "testing" ? <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" /> : <Send className="mr-1.5 h-3.5 w-3.5" />}
        Test
      </Button>
      {testStatus[channel] === "success" && <Badge variant="low" className="gap-1"><CheckCircle2 className="h-3 w-3" />Sent</Badge>}
      {testStatus[channel] === "error" && <Badge variant="critical" className="gap-1"><XCircle className="h-3 w-3" />Failed</Badge>}
    </div>
  );

  const SeveritySelect = ({ value, onChange }: { value: string; onChange: (v: string) => void }) => (
    <div className="space-y-2">
      <Label>Min Severity to Notify</Label>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger className="h-9 w-40" aria-label="Minimum severity"><SelectValue /></SelectTrigger>
        <SelectContent>
          <SelectItem value="critical">Critical</SelectItem>
          <SelectItem value="high">High</SelectItem>
          <SelectItem value="medium">Medium</SelectItem>
          <SelectItem value="low">Low</SelectItem>
          <SelectItem value="info">Info</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );

  const Toggle = ({ enabled, onToggle, id }: { enabled: boolean; onToggle: () => void; id: string }) => (
    <button
      id={id}
      role="switch"
      aria-checked={enabled}
      onClick={onToggle}
      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${enabled ? "bg-primary" : "bg-muted"}`}
    >
      <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${enabled ? "translate-x-6" : "translate-x-1"}`} />
    </button>
  );

  return (
    <div className="space-y-6">
      {/* Email SMTP */}
      <Card className="border-border">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2 text-base">
                <Mail className="h-5 w-5 text-primary" />Email (SMTP)
              </CardTitle>
              <CardDescription>Send alert notifications via email</CardDescription>
            </div>
            <div className="flex items-center gap-2">
              <Label htmlFor="email-toggle" className="text-sm">Enable</Label>
              <Toggle id="email-toggle" enabled={email.enabled} onToggle={() => setEmail((p) => ({ ...p, enabled: !p.enabled }))} />
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="smtp-host">SMTP Host</Label>
              <Input id="smtp-host" placeholder="smtp.example.com" value={email.host} onChange={(e) => setEmail((p) => ({ ...p, host: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="smtp-port">Port</Label>
              <Input id="smtp-port" type="number" value={email.port} onChange={(e) => setEmail((p) => ({ ...p, port: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="smtp-user">Username</Label>
              <Input id="smtp-user" placeholder="user@example.com" value={email.username} onChange={(e) => setEmail((p) => ({ ...p, username: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="smtp-pass">Password</Label>
              <Input id="smtp-pass" type="password" placeholder="••••••••" value={email.password} onChange={(e) => setEmail((p) => ({ ...p, password: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="smtp-from">From Address</Label>
              <Input id="smtp-from" type="email" placeholder="alerts@raksha.io" value={email.from} onChange={(e) => setEmail((p) => ({ ...p, from: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="smtp-to">To Address(es)</Label>
              <Input id="smtp-to" placeholder="admin@example.com, soc@example.com" value={email.to} onChange={(e) => setEmail((p) => ({ ...p, to: e.target.value }))} />
            </div>
          </div>
          <div className="flex items-center gap-3">
            <Checkbox id="smtp-tls" checked={email.tlsEnabled} onCheckedChange={(checked) => setEmail((p) => ({ ...p, tlsEnabled: !!checked }))} />
            <Label htmlFor="smtp-tls" className="cursor-pointer">Enable TLS</Label>
          </div>
          <div className="flex items-center justify-between">
            <SeveritySelect value={email.minSeverity} onChange={(v) => setEmail((p) => ({ ...p, minSeverity: v }))} />
            <TestButton channel="email" />
          </div>
        </CardContent>
      </Card>

      {/* Telegram */}
      <Card className="border-border">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2 text-base">
                <MessageSquare className="h-5 w-5 text-primary" />Telegram
              </CardTitle>
              <CardDescription>Send alerts to Telegram channels or groups</CardDescription>
            </div>
            <div className="flex items-center gap-2">
              <Label htmlFor="tg-toggle" className="text-sm">Enable</Label>
              <Toggle id="tg-toggle" enabled={telegram.enabled} onToggle={() => setTelegram((p) => ({ ...p, enabled: !p.enabled }))} />
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="tg-token">Bot Token</Label>
              <Input id="tg-token" type="password" placeholder="123456:ABC-DEF..." value={telegram.botToken} onChange={(e) => setTelegram((p) => ({ ...p, botToken: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="tg-chats">Chat IDs</Label>
              <Input id="tg-chats" placeholder="-1001234567890, -1009876543210" value={telegram.chatIds} onChange={(e) => setTelegram((p) => ({ ...p, chatIds: e.target.value }))} />
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="tg-parse">Parse Mode</Label>
            <Select value={telegram.parseMode} onValueChange={(v) => setTelegram((p) => ({ ...p, parseMode: v }))}>
              <SelectTrigger id="tg-parse" className="w-40" aria-label="Parse mode"><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="HTML">HTML</SelectItem>
                <SelectItem value="Markdown">Markdown</SelectItem>
                <SelectItem value="MarkdownV2">MarkdownV2</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex items-center justify-between">
            <SeveritySelect value={telegram.minSeverity} onChange={(v) => setTelegram((p) => ({ ...p, minSeverity: v }))} />
            <TestButton channel="telegram" />
          </div>
        </CardContent>
      </Card>

      {/* Slack */}
      <Card className="border-border">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2 text-base">
                <Hash className="h-5 w-5 text-primary" />Slack
              </CardTitle>
              <CardDescription>Send alerts to Slack via incoming webhook</CardDescription>
            </div>
            <div className="flex items-center gap-2">
              <Label htmlFor="slack-toggle" className="text-sm">Enable</Label>
              <Toggle id="slack-toggle" enabled={slack.enabled} onToggle={() => setSlack((p) => ({ ...p, enabled: !p.enabled }))} />
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="slack-webhook">Webhook URL</Label>
            <Input id="slack-webhook" type="url" placeholder="https://hooks.slack.com/services/T.../B.../..." value={slack.webhookUrl} onChange={(e) => setSlack((p) => ({ ...p, webhookUrl: e.target.value }))} />
          </div>
          <div className="flex items-center justify-between">
            <SeveritySelect value={slack.minSeverity} onChange={(v) => setSlack((p) => ({ ...p, minSeverity: v }))} />
            <TestButton channel="slack" />
          </div>
        </CardContent>
      </Card>

      {/* Save */}
      <div className="flex justify-end">
        <Button>Save Notification Settings</Button>
      </div>
    </div>
  );
}
