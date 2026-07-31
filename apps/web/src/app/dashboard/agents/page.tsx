"use client";

import { useState } from "react";
import { Plus, Loader2 } from "lucide-react";
import { TokenDisplay, ActiveTokensTable, EnrolledAgentsTable } from "@/components/agents/token-display";
import { api, apiClient } from "@/lib/api";

interface GeneratedToken {
  token_id: string;
  token: string;
  expires_at: string;
  install_command_linux: string;
  install_command_windows: string;
  max_uses: number;
}

export default function AgentsPage() {
  const [generatedToken, setGeneratedToken] = useState<GeneratedToken | null>(null);
  const [copied, setCopied] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopied(id);
    setTimeout(() => setCopied(null), 2000);
  };

  const handleAddAgent = async () => {
    setIsLoading(true);
    setError(null);
    setGeneratedToken(null);

    try {
      // Restore token from localStorage if not set
      if (!apiClient.getToken()) {
        const stored = localStorage.getItem("raksha_auth_token");
        if (stored) {
          const parsed = JSON.parse(stored);
          apiClient.setToken(parsed.access_token);
        }
      }

      const response = await api.agents.generateToken({
        agent_name: "default",
        labels: [],
        expiry_hours: 24,
        max_uses: 1,
      });

      setGeneratedToken(response as unknown as GeneratedToken);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to generate token");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Agents</h1>
          <p className="text-zinc-400 mt-1">
            Manage enrolled agents and generate enrollment tokens
          </p>
        </div>
        <button
          onClick={handleAddAgent}
          disabled={isLoading}
          className="flex items-center gap-2 px-4 py-2 bg-emerald-600 
                     hover:bg-emerald-700 text-white rounded-lg transition-colors
                     disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <Plus className="w-4 h-4" />
          )}
          {isLoading ? "Generating..." : "Add Agent"}
        </button>
      </div>

      {error && (
        <div className="bg-red-900/50 border border-red-500/50 rounded-xl p-4">
          <p className="text-red-300 text-sm">{error}</p>
        </div>
      )}

      {generatedToken && (
        <TokenDisplay token={generatedToken} onCopy={copyToClipboard} copied={copied} />
      )}

      <ActiveTokensTable />
      <EnrolledAgentsTable />
    </div>
  );
}

