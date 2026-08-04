"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Loader2 } from "lucide-react";
import { isAuthenticated, clearStoredToken } from "@/lib/auth";

/**
 * Client-side route guard for the dashboard.
 * Redirects to login if not authenticated.
 */
export function AuthGuard({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const [status, setStatus] = useState<"checking" | "allowed">("checking");

  useEffect(() => {
    // Check authentication status
    if (isAuthenticated()) {
      setStatus("allowed");
    } else {
      // Clear any expired/invalid token and redirect to login
      clearStoredToken();
      router.replace("/login");
    }
  }, [router]);

  if (status === "checking") {
    return (
      <div
        className="min-h-screen flex items-center justify-center bg-background"
        role="status"
        aria-live="polite"
      >
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        <span className="sr-only">Verifying your session</span>
      </div>
    );
  }

  return <>{children}</>;
}
