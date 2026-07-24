"use client";

import { useState } from "react";
import {
  Shield,
  Database,
  UserCog,
  Puzzle,
  Radio,
  CheckCircle2,
  XCircle,
  Loader2,
  ArrowRight,
  ArrowLeft,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { StepSystemCheck } from "./steps";
import { StepDatabase, StepAdmin } from "./step-forms";
import { StepModules, StepFeeds } from "./step-modules";
import { cn } from "@/lib/utils";

const steps = [
  { id: 1, title: "System Check", icon: Shield },
  { id: 2, title: "Database", icon: Database },
  { id: 3, title: "Admin Account", icon: UserCog },
  { id: 4, title: "Modules", icon: Puzzle },
  { id: 5, title: "Intelligence Feeds", icon: Radio },
];

interface SystemCheckResult {
  check: string;
  passed: boolean;
  message: string;
}

export default function SetupPage() {
  const [currentStep, setCurrentStep] = useState(1);
  const [isChecking, setIsChecking] = useState(false);
  const [systemChecks, setSystemChecks] = useState<SystemCheckResult[]>([]);
  const [dbConfig, setDbConfig] = useState({
    host: "localhost", port: "5432", name: "raksha",
    username: "raksha_admin", password: "", type: "postgresql",
  });
  const [adminConfig, setAdminConfig] = useState({
    name: "", email: "", password: "", confirmPassword: "",
  });
  const [modules, setModules] = useState({
    serverMonitoring: true, networkSecurity: true, databaseMonitoring: true,
    complianceEngine: true, threatIntelligence: true, auditTrail: true,
  });
  const [feeds, setFeeds] = useState({
    otx: true, abuseIpDb: false, virusTotal: false, customFeeds: "",
  });

  const runSystemCheck = async () => {
    setIsChecking(true);
    setSystemChecks([]);
    const checks: SystemCheckResult[] = [
      { check: "Node.js version", passed: true, message: "v20.11.0 detected" },
      { check: "Available memory", passed: true, message: "16GB available (min: 4GB)" },
      { check: "Disk space", passed: true, message: "120GB free (min: 20GB)" },
      { check: "Network connectivity", passed: true, message: "Internet access confirmed" },
      { check: "Port 443 availability", passed: true, message: "Port available" },
      { check: "Port 3001 availability", passed: true, message: "Port available" },
      { check: "OpenSSL version", passed: true, message: "OpenSSL 3.0.11" },
    ];
    for (let i = 0; i < checks.length; i++) {
      await new Promise((resolve) => setTimeout(resolve, 500));
      setSystemChecks((prev) => [...prev, checks[i]]);
    }
    setIsChecking(false);
  };

  const progress = (currentStep / steps.length) * 100;

  return (
    <div className="min-h-screen bg-background flex flex-col items-center justify-center p-4">
      <div className="mb-8 text-center">
        <div className="flex items-center justify-center gap-2 mb-2">
          <Shield className="h-10 w-10 text-primary" />
          <h1 className="text-3xl font-bold text-foreground">Raksha Setup</h1>
        </div>
        <p className="text-muted-foreground">Configure your security platform</p>
      </div>

      <div className="w-full max-w-2xl mb-6">
        <div className="flex items-center justify-between mb-2">
          {steps.map((step) => {
            const Icon = step.icon;
            return (
              <div key={step.id} className={cn(
                "flex items-center gap-1.5",
                currentStep >= step.id ? "text-primary" : "text-muted-foreground"
              )}>
                <Icon className="h-4 w-4" />
                <span className="text-xs font-medium hidden sm:inline">{step.title}</span>
              </div>
            );
          })}
        </div>
        <Progress value={progress} className="h-2" />
      </div>

      <Card className="w-full max-w-2xl border-border">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            {(() => { const StepIcon = steps[currentStep - 1].icon; return <StepIcon className="h-5 w-5 text-primary" />; })()}
            Step {currentStep}: {steps[currentStep - 1].title}
          </CardTitle>
          <CardDescription>
            {currentStep === 1 && "Verifying system requirements and compatibility"}
            {currentStep === 2 && "Configure your database connection"}
            {currentStep === 3 && "Create the administrator account"}
            {currentStep === 4 && "Select which security modules to enable"}
            {currentStep === 5 && "Configure threat intelligence feed sources"}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {currentStep === 1 && (
            <StepSystemCheck systemChecks={systemChecks} isChecking={isChecking} onRun={runSystemCheck} />
          )}
          {currentStep === 2 && (
            <StepDatabase dbConfig={dbConfig} setDbConfig={setDbConfig} />
          )}
          {currentStep === 3 && (
            <StepAdmin adminConfig={adminConfig} setAdminConfig={setAdminConfig} />
          )}
          {currentStep === 4 && (
            <StepModules modules={modules} setModules={setModules} />
          )}
          {currentStep === 5 && (
            <StepFeeds feeds={feeds} setFeeds={setFeeds} />
          )}

          <div className="flex items-center justify-between mt-6 pt-4 border-t border-border">
            <Button variant="outline" onClick={() => setCurrentStep(Math.max(1, currentStep - 1))} disabled={currentStep === 1}>
              <ArrowLeft className="h-4 w-4 mr-2" /> Back
            </Button>
            {currentStep < 5 ? (
              <Button onClick={() => setCurrentStep(Math.min(5, currentStep + 1))}>
                Next <ArrowRight className="h-4 w-4 ml-2" />
              </Button>
            ) : (
              <Button onClick={() => (window.location.href = "/login")}>
                Complete Setup <CheckCircle2 className="h-4 w-4 ml-2" />
              </Button>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
