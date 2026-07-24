"use client";

import { useState } from "react";
import { Shield, Lock, KeyRound, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export default function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [mfaCode, setMfaCode] = useState("");
  const [showMfa, setShowMfa] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsLoading(true);

    // Simulate login
    await new Promise((resolve) => setTimeout(resolve, 1000));

    if (!showMfa) {
      setShowMfa(true);
      setIsLoading(false);
      return;
    }

    // After MFA, redirect to dashboard
    window.location.href = "/dashboard";
  };

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="flex items-center justify-center gap-2 mb-8">
          <Shield className="h-10 w-10 text-primary" />
          <div>
            <h1 className="text-2xl font-bold text-foreground">Raksha</h1>
            <p className="text-xs text-muted-foreground">Security Platform</p>
          </div>
        </div>

        <Card className="border-border">
          <CardHeader className="text-center">
            <CardTitle className="flex items-center justify-center gap-2">
              <Lock className="h-5 w-5" />
              {showMfa ? "Multi-Factor Authentication" : "Sign In"}
            </CardTitle>
            <CardDescription>
              {showMfa
                ? "Enter the 6-digit code from your authenticator app"
                : "Enter your credentials to access the platform"}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleLogin} className="space-y-4">
              {!showMfa ? (
                <>
                  <div className="space-y-2">
                    <Label htmlFor="email">Email</Label>
                    <Input
                      id="email"
                      type="email"
                      value={email}
                      onChange={(e) => setEmail(e.target.value)}
                      placeholder="admin@organization.com"
                      required
                      autoComplete="email"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="password">Password</Label>
                    <Input
                      id="password"
                      type="password"
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      placeholder="Enter your password"
                      required
                      autoComplete="current-password"
                    />
                  </div>
                </>
              ) : (
                <div className="space-y-2">
                  <Label htmlFor="mfa">Authentication Code</Label>
                  <div className="flex items-center gap-2">
                    <KeyRound className="h-5 w-5 text-muted-foreground" />
                    <Input
                      id="mfa"
                      type="text"
                      inputMode="numeric"
                      pattern="[0-9]*"
                      maxLength={6}
                      value={mfaCode}
                      onChange={(e) => setMfaCode(e.target.value.replace(/\D/g, ""))}
                      placeholder="000000"
                      required
                      className="text-center text-lg tracking-widest"
                      autoComplete="one-time-code"
                    />
                  </div>
                </div>
              )}

              {error && (
                <p className="text-sm text-destructive">{error}</p>
              )}

              <Button type="submit" className="w-full" disabled={isLoading}>
                {isLoading && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                {showMfa ? "Verify" : "Sign In"}
              </Button>

              {showMfa && (
                <Button
                  type="button"
                  variant="ghost"
                  className="w-full"
                  onClick={() => setShowMfa(false)}
                >
                  Back to login
                </Button>
              )}
            </form>
          </CardContent>
        </Card>

        <p className="text-center text-xs text-muted-foreground mt-4">
          Protected by Raksha Security Platform v1.0
        </p>
      </div>
    </div>
  );
}
