"use client";

import { useState } from "react";
import { Plus } from "lucide-react";
import { TokenDisplay, ActiveTokensTable, EnrolledAgentsTable } from "@/components/agents/token-display";

interface GeneratedToken {
  token_id: string;
  token: string;
  expires_at: string;
  install_command_linux: string;
  install_command_windows: string;
}

export default function AgentsPage() {
  const [generatedToken, setGeneratedToken] = useState<GeneratedToken | null>(null);
  const [copied, setCopied] = useState<string | null>(null);

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopied(id);
    setTimeout(() => setCopied(null), 2000);
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
          className="flex items-center gap-2 px-4 py-2 bg-emerald-600 
                     hover:bg-emerald-700 text-white rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" />
          Add Agent
        </button>
      </div>

      {generatedToken && (
        <TokenDisplay token={generatedToken} onCopy={copyToClipboard} copied={copied} />
      )}

      <ActiveTokensTable />
      <EnrolledAgentsTable />
    </div>
  );
}

