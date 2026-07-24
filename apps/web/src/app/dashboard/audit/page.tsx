import { ScrollText, CheckCircle2, XCircle } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { AuditEntry } from "@/types";

const mockAuditEntries: AuditEntry[] = [
  { id: "1", action: "user.login", actor: "admin@raksha.io", resource: "auth-service", timestamp: "2024-01-15T10:30:00Z", ipAddress: "10.0.1.50", details: "Successful MFA login", result: "success" },
  { id: "2", action: "firewall.rule.update", actor: "ops@raksha.io", resource: "fw-rule-042", timestamp: "2024-01-15T10:28:00Z", ipAddress: "10.0.1.51", details: "Updated inbound rule for port 8080", result: "success" },
  { id: "3", action: "user.login", actor: "unknown@external.com", resource: "auth-service", timestamp: "2024-01-15T10:25:00Z", ipAddress: "203.0.113.45", details: "Failed login attempt - invalid credentials", result: "failure" },
  { id: "4", action: "server.restart", actor: "admin@raksha.io", resource: "web-02", timestamp: "2024-01-15T10:20:00Z", ipAddress: "10.0.1.50", details: "Scheduled maintenance restart", result: "success" },
  { id: "5", action: "database.backup", actor: "system", resource: "raksha-primary", timestamp: "2024-01-15T10:00:00Z", ipAddress: "10.0.2.10", details: "Automated daily backup completed", result: "success" },
  { id: "6", action: "user.permission.change", actor: "admin@raksha.io", resource: "user-analyst-01", timestamp: "2024-01-15T09:45:00Z", ipAddress: "10.0.1.50", details: "Role changed from viewer to analyst", result: "success" },
  { id: "7", action: "api.key.revoke", actor: "ops@raksha.io", resource: "api-key-legacy", timestamp: "2024-01-15T09:30:00Z", ipAddress: "10.0.1.51", details: "Revoked expired API key", result: "success" },
];

export default function AuditPage() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Audit Trail</h2>
        <p className="text-muted-foreground">Complete system activity log</p>
      </div>

      <Card className="border-border">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <ScrollText className="h-5 w-5 text-primary" />
            Recent Activity
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {mockAuditEntries.map((entry) => (
              <div key={entry.id} className="flex items-start gap-3 rounded-lg border border-border p-3 hover:bg-accent/50 transition-colors">
                {entry.result === "success" ? (
                  <CheckCircle2 className="h-4 w-4 mt-0.5 text-green-500 shrink-0" />
                ) : (
                  <XCircle className="h-4 w-4 mt-0.5 text-red-500 shrink-0" />
                )}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-sm font-medium font-mono">{entry.action}</span>
                    <Badge variant={entry.result === "success" ? "default" : "destructive"} className="text-[10px]">{entry.result}</Badge>
                  </div>
                  <p className="text-xs text-muted-foreground">{entry.details}</p>
                  <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
                    <span>{entry.actor}</span>
                    <span>•</span>
                    <span>{entry.resource}</span>
                    <span>•</span>
                    <span>{entry.ipAddress}</span>
                    <span>•</span>
                    <span>{new Date(entry.timestamp).toLocaleString()}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
