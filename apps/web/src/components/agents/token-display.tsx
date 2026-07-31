"use client";

import { Shield, Copy, Check, Clock } from "lucide-react";

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
  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6">
      <h3 className="text-lg font-semibold text-white mb-4">
        Active Enrollment Tokens
      </h3>
      <div className="text-center py-8 text-zinc-500">
        <Shield className="w-12 h-12 mx-auto mb-3 opacity-50" />
        <p>No active tokens. Click &quot;Add Agent&quot; to generate one.</p>
      </div>
    </div>
  );
}

export function EnrolledAgentsTable() {
  return (
    <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6">
      <h3 className="text-lg font-semibold text-white mb-4">
        Enrolled Agents
      </h3>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-zinc-400 border-b border-zinc-800">
              <th className="text-left py-3 px-2">Agent</th>
              <th className="text-left py-3 px-2">Host</th>
              <th className="text-left py-3 px-2">OS</th>
              <th className="text-left py-3 px-2">Status</th>
              <th className="text-left py-3 px-2">Last Seen</th>
              <th className="text-left py-3 px-2">Cert Expires</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td colSpan={6} className="text-center py-8 text-zinc-500">
                No agents enrolled yet
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}
