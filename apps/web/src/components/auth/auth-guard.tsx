"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Loader2 } from "lucide-react";
import { isAuthenticated, clearStoredToken } from "@/lib/auth";

/**
 * Client-side route guard for the dashboard.
 *
 * Tokens live in localStorage (not cookies), so Next.js middleware running on
 * the server cannot see them. The check therefore has to happen here, after
 * hydration. Children are never rendered until the session is confirmed, so
 * authenticated markup is not briefly exposed.
 *
 * Note this is a UX guard only: every protected endpoint is independently
 * enforced by the portal's `auth_layer`, which is the real security boundary.
 */
export function AuthGuard({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const [status, setStatus] = useState<"checking" | "allowed">("checking");

  useEffect(() => {
    if (isAuthenticated()) {
      setStatus("allowed");
      return;
    }

    // Drop an expired or malformed token so /login starts from a clean slate.
    clearStoredToken();
    router.replace("/login");
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
