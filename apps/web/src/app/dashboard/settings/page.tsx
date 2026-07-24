"use client";

import { Bell, Shield, Globe, Key } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Checkbox } from "@/components/ui/checkbox";

export default function SettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Settings</h2>
        <p className="text-muted-foreground">System configuration and preferences</p>
      </div>

      <Tabs defaultValue="general" className="space-y-4">
        <TabsList>
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="notifications">Notifications</TabsTrigger>
          <TabsTrigger value="security">Security</TabsTrigger>
          <TabsTrigger value="integrations">Integrations</TabsTrigger>
        </TabsList>

        <TabsContent value="general">
          <Card className="border-border">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Globe className="h-5 w-5 text-primary" />General Settings
              </CardTitle>
              <CardDescription>Platform-wide configuration</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="org-name">Organization Name</Label>
                <Input id="org-name" defaultValue="Raksha Security" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="api-url">API Base URL</Label>
                <Input id="api-url" defaultValue="http://localhost:3001/api" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="retention">Data Retention (days)</Label>
                <Input id="retention" type="number" defaultValue="90" />
              </div>
              <Button>Save Changes</Button>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="notifications">
          <Card className="border-border">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Bell className="h-5 w-5 text-primary" />Notifications
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {["Email for critical alerts", "Email for high severity", "Slack notifications", "SMS for critical alerts"].map((item) => (
                <div key={item} className="flex items-center gap-3">
                  <Checkbox id={item} defaultChecked={item.includes("critical")} />
                  <Label htmlFor={item} className="cursor-pointer">{item}</Label>
                </div>
              ))}
              <Button>Save Preferences</Button>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="security">
          <Card className="border-border">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Shield className="h-5 w-5 text-primary" />Security
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="session-timeout">Session Timeout (min)</Label>
                <Input id="session-timeout" type="number" defaultValue="30" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="max-attempts">Max Login Attempts</Label>
                <Input id="max-attempts" type="number" defaultValue="5" />
              </div>
              <div className="flex items-center gap-3">
                <Checkbox id="enforce-mfa" defaultChecked />
                <Label htmlFor="enforce-mfa" className="cursor-pointer">Enforce MFA for all users</Label>
              </div>
              <Button>Save Security Settings</Button>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="integrations">
          <Card className="border-border">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Key className="h-5 w-5 text-primary" />Integrations
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="slack-webhook">Slack Webhook URL</Label>
                <Input id="slack-webhook" type="url" placeholder="https://hooks.slack.com/..." />
              </div>
              <div className="space-y-2">
                <Label htmlFor="vt-key">VirusTotal API Key</Label>
                <Input id="vt-key" type="password" placeholder="Enter API key" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="siem-url">SIEM Endpoint</Label>
                <Input id="siem-url" type="url" placeholder="https://siem.example.com/api" />
              </div>
              <Button>Save Integrations</Button>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
