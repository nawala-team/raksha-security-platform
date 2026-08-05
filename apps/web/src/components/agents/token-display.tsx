"use client";

import { useState, useEffect } from "react";
import { Shield, Copy, Check, Clock, Server, RefreshCw } from "lucide-react";
import { api, apiClient } from "@/lib/api";

interface GeneratedToken {
  token_id: string;
  token: string;
  expires_at: string;
  max_uses: number;
  install_command_linux: string;
  install_command_windows: string;
}

interface TokenDisplayProps {
  token: GeneratedToken;
  onCopy: (text: string, id: string) => void;
  copied: string | null;
}

interface Agent {
  id: string;
  name: string;
  hostname: string;
  os: string;
  version: string;
  status: string;
  last_seen: string;
  enrolled_at: string;
}

interface TokenInfo {
  id: string;
  token: string;
  expires_at: string;
  uses_remaining: number;
  description?: string;
}

export function TokenDisplay({ token, onCopy, copied }: TokenDisplayProps) {
  return (
    <div className="bg-zinc-900 border border-emerald-500/50 rounded-xl p-6">
      <div className="flex items-center gap-2 mb-4">
        <Shield className="w-5 h-5 text-emerald-400" />
        <h3 className="text-lg font-semibold text-white">
          Enrollment Token Generated
        </h3>
      </div>
      <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-3 mb-4">
        <p className="text-yellow-300 text-sm">
          ⚠️ Token shown only once. Copy now. Expires in 24h, one-time use.
        </p>
      </div>

      <div className="space-y-4">
        <CommandBlock
          label="Token (one-time use)"
          value={token.token}
          id="token"
          onCopy={onCopy}
          copied={copied}
          colorClass="text-emerald-300"
        />
        <CommandBlock
          label="Install Command (Linux/macOS)"
          value={token.install_command_linux}
          id="linux"
          onCopy={onCopy}
          copied={copied}
          colorClass="text-blue-300"
        />
        <CommandBlock
          label="Install Command (Windows PowerShell)"
          value={token.install_command_windows}
          id="win"
          onCopy={onCopy}
          copied={copied}
          colorClass="text-purple-300"
        />
        <div className="flex items-center gap-2 text-sm text-zinc-500">
          <Clock className="w-4 h-4" />
          <span>Expires: {new Date(token.expires_at).toLocaleString()}</span>
        </div>
      </div>
    </div>
  );
}

function CommandBlock({
  label, value, id, onCopy, copied, colorClass,
}: {
  label: string; value: string; id: string;
  onCopy: (t: string, id: string) => void;
  copied: string | null; colorClass: string;
}) {
  return (
    <div>
      <label className="text-sm text-zinc-400 block mb-1">{label}</label>
      <div className="flex items-center gap-2">
        <code className={`flex-1 bg-zinc-800 px-3 py-2 rounded text-sm font-mono overflow-x-auto ${colorClass}`}>
          {value}
        </code>
        <button onClick={() => onCopy(value, id)} className="p-2 hover:bg-zinc-700 rounded">
          {copied === id ? (
            <Check className="w-4 h-4 text-emerald-400" />
          ) : (
            <Copy className="w-4 h-4 text-zinc-400" />
          )}
        </button>
      </div>
    </div>
  );
}

export function ActiveTokensTable() {
  const [tokens, setTokens] = useState<TokenInfo[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchTokens = async () => {
      try {
        if (!apiClient.getToken()) {
          const stored = localStorage.getItem("raksha_auth_token");
          if (stored) {
            const parsed = JSON.parse(stored);
            apiClient.setToken(parsed.access_token);
          }
        }
        const response = await api.agents.listTokens();
        setTokens((response as any).tokens || []);
      } catch (err) {
        console.error("Failed to fetch tokens:", err);
      } finally {
        setLoading(false);
      }
    };
    fetchTokens();
  }, []);

  if (loading) {
    return (
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Active Enrollment Tokens</h3>
        <div className="text-center py-8 text-zinc-500">
          <RefreshCw className="w-8 h-8 mx-auto mb-3 animate-spin opacity-50" />
          <p>Loading tokens...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6">
      <h3 className="text-lg font-semibold text-white mb-4">Active Enrollment Tokens</h3>
      {tokens.length === 0 ? (
        <div className="text-center py-8 text-zinc-500">
          <Shield className="w-12 h-12 mx-auto mb-3 opacity-50" />
          <p>No active tokens. Click &quot;Add Agent&quot; to generate one.</p>
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-zinc-400 border-b border-zinc-800">
                <th className="text-left py-3 px-2">Token Prefix</th>
                <th className="text-left py-3 px-2">Description</th>
                <th className="text-left py-3 px-2">Uses Left</th>
                <th className="text-left py-3 px-2">Expires</th>
              </tr>
            </thead>
            <tbody>
              {tokens.map((t) => (
                <tr key={t.id} className="border-b border-zinc-800/50 hover:bg-zinc-800/30">
                  <td className="py-3 px-2 font-mono text-emerald-400">{t.token}...</td>
                  <td className="py-3 px-2 text-zinc-300">{t.description || "-"}</td>
                  <td className="py-3 px-2 text-zinc-300">{t.uses_remaining}</td>
                  <td className="py-3 px-2 text-zinc-400">{new Date(t.expires_at).toLocaleString()}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

export function EnrolledAgentsTable() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchAgents = async () => {
      try {
        if (!apiClient.getToken()) {
          const stored = localStorage.getItem("raksha_auth_token");
          if (stored) {
            const parsed = JSON.parse(stored);
            apiClient.setToken(parsed.access_token);
          }
        }
        const response = await api.agents.list();
        setAgents((response as any).data || []);
      } catch (err) {
        console.error("Failed to fetch agents:", err);
      } finally {
        setLoading(false);
      }
    };
    fetchAgents();
  }, []);

  const getStatusBadge = (status: string) => {
    const colors: Record<string, string> = {
      online: "bg-emerald-500/20 text-emerald-400 border-emerald-500/30",
      offline: "bg-red-500/20 text-red-400 border-red-500/30",
    };
    return colors[status] || colors.offline;
  };

  const formatLastSeen = (dateStr: string) => {
    const date = new Date(dateStr);
    const diffMins = Math.floor((Date.now() - date.getTime()) / 60000);
    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`;
    return date.toLocaleDateString();
  };

  if (loading) {
    return (
      <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Enrolled Agents</h3>
        <div className="text-center py-8 text-zinc-500">
          <RefreshCw className="w-8 h-8 mx-auto mb-3 animate-spin opacity-50" />
          <p>Loading agents...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6">
      <h3 className="text-lg font-semibold text-white mb-4">Enrolled Agents ({agents.length})</h3>
      {agents.length === 0 ? (
        <div className="text-center py-8 text-zinc-500">
          <Server className="w-12 h-12 mx-auto mb-3 opacity-50" />
          <p>No agents enrolled yet</p>
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-zinc-400 border-b border-zinc-800">
                <th className="text-left py-3 px-2">Agent</th>
                <th className="text-left py-3 px-2">Host</th>
                <th className="text-left py-3 px-2">OS</th>
                <th className="text-left py-3 px-2">Status</th>
                <th className="text-left py-3 px-2">Last Seen</th>
              </tr>
            </thead>
            <tbody>
              {agents.map((agent) => (
                <tr key={agent.id} className="border-b border-zinc-800/50 hover:bg-zinc-800/30">
                  <td className="py-3 px-2">
                    <div className="flex items-center gap-2">
                      <Server className="w-4 h-4 text-zinc-500" />
                      <span className="text-white font-medium">{agent.name}</span>
                    </div>
                  </td>
                  <td className="py-3 px-2 text-zinc-300 font-mono text-xs">{agent.hostname}</td>
                  <td className="py-3 px-2 text-zinc-300 capitalize">{agent.os}</td>
                  <td className="py-3 px-2">
                    <span className={`px-2 py-1 rounded-full text-xs border ${getStatusBadge(agent.status)}`}>
                      {agent.status}
                    </span>
                  </td>
                  <td className="py-3 px-2 text-zinc-400">{formatLastSeen(agent.last_seen)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
