"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import type { ThreatLevel } from "@/types";

export type WsEventChannel =
  | "alerts"
  | "metrics"
  | "agent_status"
  | "compliance"
  | "audit_log"
  | "system_health";

export interface WsRealtimeEvent {
  channel: WsEventChannel;
  event_type: string;
  payload: Record<string, unknown>;
  timestamp: string;
}

interface UseRealtimeOptions {
  channels: WsEventChannel[];
  onEvent?: (event: WsRealtimeEvent) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
  autoReconnect?: boolean;
  maxReconnectAttempts?: number;
}

interface RealtimeState {
  connected: boolean;
  sessionId: string | null;
  subscribedChannels: WsEventChannel[];
  reconnectAttempts: number;
}

/**
 * Resolve the WebSocket endpoint.
 *
 * A hardcoded localhost default breaks any deployment that is not on the
 * developer's machine, so derive the URL from the current origin instead and
 * let NEXT_PUBLIC_WS_URL override it when the portal lives elsewhere.
 */
function resolveWsUrl(): string {
  if (process.env.NEXT_PUBLIC_WS_URL) return process.env.NEXT_PUBLIC_WS_URL;

  if (typeof window !== "undefined") {
    const scheme = window.location.protocol === "https:" ? "wss:" : "ws:";
    return `${scheme}//${window.location.host}/api/v1/ws`;
  }

  return "ws://localhost:8080/api/v1/ws";
}

export function useRealtime({
  channels,
  onEvent,
  onConnect,
  onDisconnect,
  autoReconnect = true,
  maxReconnectAttempts = 5,
}: UseRealtimeOptions) {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const [state, setState] = useState<RealtimeState>({
    connected: false,
    sessionId: null,
    subscribedChannels: [],
    reconnectAttempts: 0,
  });

  // Use refs for callbacks to avoid re-creating WebSocket on callback changes
  const onEventRef = useRef(onEvent);
  const onConnectRef = useRef(onConnect);
  const onDisconnectRef = useRef(onDisconnect);

  // Update refs when callbacks change
  useEffect(() => {
    onEventRef.current = onEvent;
    onConnectRef.current = onConnect;
    onDisconnectRef.current = onDisconnect;
  }, [onEvent, onConnect, onDisconnect]);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    try {
      const ws = new WebSocket(resolveWsUrl());
      wsRef.current = ws;

      ws.onopen = () => {
        reconnectAttemptsRef.current = 0;
        setState((s) => ({ ...s, connected: true, reconnectAttempts: 0 }));
        // Subscribe to channels
        ws.send(JSON.stringify({ action: "subscribe", channels }));
        onConnectRef.current?.();
      };

      ws.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data);
          switch (msg.type) {
            case "connected":
              setState((s) => ({ ...s, sessionId: msg.session_id }));
              break;
            case "subscribed":
              setState((s) => ({ ...s, subscribedChannels: msg.channels }));
              break;
            case "event":
              onEventRef.current?.(msg as WsRealtimeEvent);
              break;
            case "pong":
              break;
          }
        } catch {
          // Ignore parse errors
        }
      };

      ws.onclose = () => {
        setState((s) => ({ ...s, connected: false }));
        onDisconnectRef.current?.();
        
        if (autoReconnect && reconnectAttemptsRef.current < maxReconnectAttempts) {
          const delay = Math.min(1000 * 2 ** reconnectAttemptsRef.current, 30000);
          reconnectTimerRef.current = setTimeout(() => {
            reconnectAttemptsRef.current += 1;
            setState((s) => ({ ...s, reconnectAttempts: reconnectAttemptsRef.current }));
            connect();
          }, delay);
        }
      };

      ws.onerror = () => {
        ws.close();
      };
    } catch {
      // Connection failed
    }
  }, [channels, autoReconnect, maxReconnectAttempts]);

  const disconnect = useCallback(() => {
    if (reconnectTimerRef.current) {
      clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }
    wsRef.current?.close();
    wsRef.current = null;
    reconnectAttemptsRef.current = 0;
    setState({ connected: false, sessionId: null, subscribedChannels: [], reconnectAttempts: 0 });
  }, []);

  const sendPing = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ action: "ping" }));
    }
  }, []);

  useEffect(() => {
    connect();
    // Heartbeat every 30s
    const heartbeat = setInterval(sendPing, 30000);
    return () => {
      clearInterval(heartbeat);
      disconnect();
    };
  }, [connect, disconnect, sendPing]);

  return { ...state, disconnect, reconnect: connect };
}
