"use client";

import { useEffect, useState } from "react";
import { AlertTriangle, CheckCircle2, Info, X, XCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import { useRealtime, type WsRealtimeEvent } from "@/hooks/use-realtime";

interface ToastNotification {
  id: string;
  title: string;
  message: string;
  severity: "critical" | "high" | "medium" | "low" | "info";
  timestamp: Date;
  read: boolean;
}

const severityConfig = {
  critical: { icon: XCircle, bg: "bg-red-500/10 border-red-500/30", text: "text-red-400" },
  high: { icon: AlertTriangle, bg: "bg-orange-500/10 border-orange-500/30", text: "text-orange-400" },
  medium: { icon: Info, bg: "bg-yellow-500/10 border-yellow-500/30", text: "text-yellow-400" },
  low: { icon: CheckCircle2, bg: "bg-green-500/10 border-green-500/30", text: "text-green-400" },
  info: { icon: Info, bg: "bg-blue-500/10 border-blue-500/30", text: "text-blue-400" },
};

export function RealtimeNotifications() {
  const [notifications, setNotifications] = useState<ToastNotification[]>([]);
  const [visible, setVisible] = useState<string[]>([]);

  const handleEvent = (event: WsRealtimeEvent) => {
    if (event.channel === "alerts") {
      const notif: ToastNotification = {
        id: crypto.randomUUID(),
        title: (event.payload.title as string) || "Security Alert",
        message: (event.payload.description as string) || "",
        severity: (event.payload.severity as ToastNotification["severity"]) || "medium",
        timestamp: new Date(event.timestamp),
        read: false,
      };

      setNotifications((prev) => [notif, ...prev].slice(0, 50));
      setVisible((prev) => [notif.id, ...prev].slice(0, 5));

      // Auto-dismiss after 8 seconds for non-critical
      if (notif.severity !== "critical") {
        setTimeout(() => {
          setVisible((prev) => prev.filter((id) => id !== notif.id));
        }, 8000);
      }
    }
  };

  const { connected } = useRealtime({
    channels: ["alerts", "agent_status", "system_health"],
    onEvent: handleEvent,
  });

  const dismiss = (id: string) => {
    setVisible((prev) => prev.filter((v) => v !== id));
  };

  const visibleNotifications = notifications.filter((n) => visible.includes(n.id));

  return (
    <div
      className="fixed bottom-4 right-4 z-[100] flex flex-col-reverse gap-2 max-w-sm"
      role="log"
      aria-live="polite"
      aria-label="Security notifications"
    >
      {/* Connection indicator */}
      <div className="flex items-center gap-1.5 text-xs text-muted-foreground px-2">
        <span
          className={cn(
            "h-2 w-2 rounded-full",
            connected ? "bg-green-500 animate-pulse" : "bg-red-500"
          )}
        />
        {connected ? "Live monitoring active" : "Reconnecting..."}
      </div>

      {/* Toast notifications */}
      {visibleNotifications.map((notif) => {
        const config = severityConfig[notif.severity];
        const Icon = config.icon;

        return (
          <div
            key={notif.id}
            className={cn(
              "flex items-start gap-3 rounded-lg border p-3 shadow-lg backdrop-blur-sm animate-in slide-in-from-right-5 fade-in duration-300",
              config.bg
            )}
            role="alert"
          >
            <Icon className={cn("h-5 w-5 mt-0.5 shrink-0", config.text)} />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-foreground truncate">
                {notif.title}
              </p>
              {notif.message && (
                <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2">
                  {notif.message}
                </p>
              )}
              <span className="text-[10px] text-muted-foreground mt-1 block">
                {notif.timestamp.toLocaleTimeString()}
              </span>
            </div>
            <button
              onClick={() => dismiss(notif.id)}
              className="shrink-0 text-muted-foreground hover:text-foreground"
              aria-label="Dismiss notification"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        );
      })}
    </div>
  );
}
