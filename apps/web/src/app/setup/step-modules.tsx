import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";

export function StepModules({
  modules,
  setModules,
}: {
  modules: Record<string, boolean>;
  setModules: (m: Record<string, boolean>) => void;
}) {
  const moduleList = [
    { key: "serverMonitoring", label: "Server Monitoring", desc: "Monitor server health, resources, and performance" },
    { key: "networkSecurity", label: "Network Security", desc: "Firewall management, IDS/IPS, traffic analysis" },
    { key: "databaseMonitoring", label: "Database Monitoring", desc: "Query monitoring, connection tracking, backup status" },
    { key: "complianceEngine", label: "Compliance Engine", desc: "PCI-DSS, HIPAA, SOC2 compliance automation" },
    { key: "threatIntelligence", label: "Threat Intelligence", desc: "Real-time threat feeds and IOC matching" },
    { key: "auditTrail", label: "Audit Trail", desc: "Complete audit logging of all system activities" },
  ];

  return (
    <div className="space-y-3">
      {moduleList.map((mod) => (
        <div key={mod.key} className="flex items-start gap-3 rounded-lg border border-border p-4">
          <Checkbox
            id={mod.key}
            checked={modules[mod.key]}
            onCheckedChange={(checked) => setModules({ ...modules, [mod.key]: !!checked })}
          />
          <div className="flex-1">
            <Label htmlFor={mod.key} className="text-sm font-medium cursor-pointer">{mod.label}</Label>
            <p className="text-xs text-muted-foreground mt-0.5">{mod.desc}</p>
          </div>
        </div>
      ))}
    </div>
  );
}

export function StepFeeds({
  feeds,
  setFeeds,
}: {
  feeds: { otx: boolean; abuseIpDb: boolean; virusTotal: boolean; customFeeds: string };
  setFeeds: (f: typeof feeds) => void;
}) {
  const feedList = [
    { key: "otx", label: "AlienVault OTX", desc: "Open Threat Exchange community feeds" },
    { key: "abuseIpDb", label: "AbuseIPDB", desc: "IP address abuse reports and blacklists" },
    { key: "virusTotal", label: "VirusTotal", desc: "File and URL malware scanning (requires API key)" },
  ];

  return (
    <div className="space-y-4">
      <div className="space-y-3">
        {feedList.map((feed) => (
          <div key={feed.key} className="flex items-start gap-3 rounded-lg border border-border p-4">
            <Checkbox
              id={feed.key}
              checked={feeds[feed.key as keyof typeof feeds] as boolean}
              onCheckedChange={(checked) => setFeeds({ ...feeds, [feed.key]: !!checked })}
            />
            <div className="flex-1">
              <Label htmlFor={feed.key} className="text-sm font-medium cursor-pointer">{feed.label}</Label>
              <p className="text-xs text-muted-foreground mt-0.5">{feed.desc}</p>
            </div>
          </div>
        ))}
      </div>
      <div className="space-y-2">
        <Label htmlFor="custom-feeds">Custom Feed URLs (one per line)</Label>
        <textarea
          id="custom-feeds"
          className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          value={feeds.customFeeds}
          onChange={(e) => setFeeds({ ...feeds, customFeeds: e.target.value })}
          placeholder={"https://feeds.example.com/stix\nhttps://feeds.example.com/taxii"}
        />
      </div>
    </div>
  );
}
