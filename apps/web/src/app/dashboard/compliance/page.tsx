import { ClipboardCheck, CheckCircle2, XCircle, AlertTriangle } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import type { ComplianceFramework } from "@/types";

const mockFrameworks: ComplianceFramework[] = [
  { id: "1", name: "PCI-DSS v4.0", status: "compliant", score: 96, totalControls: 312, passedControls: 300, failedControls: 12, lastAudit: "2024-01-10" },
  { id: "2", name: "SOC 2 Type II", status: "compliant", score: 92, totalControls: 64, passedControls: 59, failedControls: 5, lastAudit: "2024-01-08" },
  { id: "3", name: "HIPAA", status: "partial", score: 78, totalControls: 54, passedControls: 42, failedControls: 12, lastAudit: "2024-01-05" },
  { id: "4", name: "ISO 27001", status: "compliant", score: 94, totalControls: 114, passedControls: 107, failedControls: 7, lastAudit: "2024-01-12" },
  { id: "5", name: "NIST CSF", status: "partial", score: 82, totalControls: 108, passedControls: 89, failedControls: 19, lastAudit: "2024-01-03" },
];

const statusConfig = {
  compliant: { color: "text-green-400", icon: CheckCircle2, badge: "low" as const },
  non_compliant: { color: "text-red-400", icon: XCircle, badge: "critical" as const },
  partial: { color: "text-yellow-400", icon: AlertTriangle, badge: "medium" as const },
  pending: { color: "text-blue-400", icon: ClipboardCheck, badge: "default" as const },
};

export default function CompliancePage() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Compliance</h2>
        <p className="text-muted-foreground">Framework compliance status and audit results</p>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
        {mockFrameworks.map((fw) => {
          const config = statusConfig[fw.status];
          const Icon = config.icon;
          return (
            <Card key={fw.id} className="border-border">
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-sm">{fw.name}</CardTitle>
                  <Badge variant={config.badge} className="capitalize">{fw.status.replace("_", " ")}</Badge>
                </div>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex items-center gap-3">
                  <div className="relative h-16 w-16 flex items-center justify-center">
                    <Icon className={`h-6 w-6 ${config.color}`} />
                  </div>
                  <div className="flex-1">
                    <p className="text-3xl font-bold">{fw.score}%</p>
                    <p className="text-xs text-muted-foreground">Compliance Score</p>
                  </div>
                </div>
                <Progress value={fw.score} className="h-2" />
                <div className="flex justify-between text-xs text-muted-foreground pt-1">
                  <span className="text-green-400">{fw.passedControls} passed</span>
                  <span className="text-red-400">{fw.failedControls} failed</span>
                  <span>{fw.totalControls} total</span>
                </div>
                <p className="text-xs text-muted-foreground border-t border-border pt-2">
                  Last audit: {new Date(fw.lastAudit).toLocaleDateString()}
                </p>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
