import { CheckCircle2, XCircle, Loader2, Shield } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";

interface SystemCheckResult {
  check: string;
  passed: boolean;
  message: string;
}

export function StepSystemCheck({
  systemChecks,
  isChecking,
  onRun,
}: {
  systemChecks: SystemCheckResult[];
  isChecking: boolean;
  onRun: () => void;
}) {
  return (
    <div className="space-y-4">
      {systemChecks.length === 0 && !isChecking && (
        <div className="text-center py-8">
          <Shield className="h-16 w-16 text-muted-foreground mx-auto mb-4" />
          <p className="text-muted-foreground mb-4">
            Run the system check to verify your environment meets the requirements.
          </p>
          <Button onClick={onRun}>Run System Check</Button>
        </div>
      )}
      {(isChecking || systemChecks.length > 0) && (
        <div className="space-y-2">
          {systemChecks.map((check, i) => (
            <div key={i} className="flex items-center gap-3 rounded-lg border border-border p-3">
              {check.passed ? (
                <CheckCircle2 className="h-5 w-5 text-green-500 shrink-0" />
              ) : (
                <XCircle className="h-5 w-5 text-red-500 shrink-0" />
              )}
              <div className="flex-1">
                <span className="text-sm font-medium">{check.check}</span>
                <p className="text-xs text-muted-foreground">{check.message}</p>
              </div>
            </div>
          ))}
          {isChecking && (
            <div className="flex items-center gap-3 rounded-lg border border-border p-3">
              <Loader2 className="h-5 w-5 text-primary animate-spin shrink-0" />
              <span className="text-sm text-muted-foreground">Checking...</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
