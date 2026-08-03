"use client";

import { useState } from "react";
import {
  Database,
  Activity,
  Shield,
  Users,
  Plus,
  Trash2,
  X,
  RefreshCw,
  HardDrive,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { DataState } from "@/components/ui/data-state";
import { useApiData } from "@/hooks/use-api-data";
import { api } from "@/lib/api";

/** Mirrors the portal's `DatabaseInstanceResponse`. */
interface DbRow {
  id: string;
  name: string;
  db_type: string;
  host: string;
  port: number;
  status: string;
  connections: number;
  max_connections: number;
  query_rate: number;
  size_bytes: number;
  encrypted: boolean;
  version: string | null;
  alerts: number;
  created_at: string;
}

const typeIcons: Record<string, string> = {
  postgresql: "PostgreSQL",
  mysql: "MySQL",
  mongodb: "MongoDB",
  redis: "Redis",
};

export default function DatabasePage() {
  const { data, loading, error, refetch } = useApiData<DbRow[]>(() => api.databases.list());
  const dbs = data ?? [];

  const [showRegister, setShowRegister] = useState(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [form, setForm] = useState({
    name: "",
    db_type: "postgresql",
    host: "127.0.0.1",
    port: "5432",
    username: "",
    password: "",
    ssl_enabled: true,
  });

  const submitRegister = async () => {
    setBusy(true);
    setMessage(null);
    try {
      await api.databases.register({
        name: form.name,
        db_type: form.db_type,
        host: form.host,
        port: Number(form.port),
        username: form.username,
        password: form.password,
        ssl_enabled: form.ssl_enabled,
      });
      setShowRegister(false);
      setForm({ name: "", db_type: "postgresql", host: "127.0.0.1", port: "5432", username: "", password: "", ssl_enabled: true });
      refetch();
    } catch (err) {
      setMessage(err instanceof Error ? err.message : "Failed to register database");
    } finally {
      setBusy(false);
    }
  };

  const removeDb = async (db: DbRow) => {
    if (!window.confirm(`Remove monitored database "${db.name}"?`)) return;
    try {
      await api.databases.unregister(db.id);
      refetch();
    } catch (err) {
      alert(err instanceof Error ? err.message : "Failed to remove database");
    }
  };

  const totalConnections = dbs.reduce((a, d) => a + (d.connections || 0), 0);
  const totalQueryRate = dbs.reduce((a, d) => a + (d.query_rate || 0), 0);
  const encryptedCount = dbs.filter((d) => d.encrypted).length;

  const stats = [
    { label: "Instances", value: dbs.length, icon: Database, color: "text-blue-400" },
    { label: "Connections", value: totalConnections, icon: Users, color: "text-green-400" },
    { label: "Queries/sec", value: totalQueryRate.toLocaleString(), icon: Activity, color: "text-yellow-400" },
    { label: "Encrypted", value: `${encryptedCount}/${dbs.length}`, icon: Shield, color: "text-purple-400" },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Database Security</h2>
          <p className="text-muted-foreground">Monitor database instances and security posture</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={refetch} aria-label="Refresh">
            <RefreshCw className="h-4 w-4" aria-hidden="true" />
          </Button>
          <Button onClick={() => setShowRegister((v) => !v)} className="gap-2">
            <Plus className="h-4 w-4" aria-hidden="true" /> Register Database
          </Button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((s) => (
          <Card key={s.label} className="border-border">
            <CardContent className="flex items-center gap-3 p-4">
              <s.icon className={`h-8 w-8 ${s.color}`} aria-hidden="true" />
              <div>
                <p className="text-2xl font-bold text-foreground">{s.value}</p>
                <p className="text-xs text-muted-foreground">{s.label}</p>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Register form */}
      {showRegister && (
        <Card className="border-border border-primary/30">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center justify-between text-base">
              Register a Database to Monitor
              <Button variant="ghost" size="icon" onClick={() => setShowRegister(false)} aria-label="Close">
                <X className="h-4 w-4" aria-hidden="true" />
              </Button>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <div className="space-y-2">
                <Label htmlFor="db-name">Display Name</Label>
                <Input id="db-name" placeholder="primary-postgres" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="db-type">Type</Label>
                <Select value={form.db_type} onValueChange={(v) => setForm({ ...form, db_type: v })}>
                  <SelectTrigger id="db-type" className="w-full"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="postgresql">PostgreSQL</SelectItem>
                    <SelectItem value="mysql">MySQL</SelectItem>
                    <SelectItem value="mongodb">MongoDB</SelectItem>
                    <SelectItem value="redis">Redis</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="db-host">Host</Label>
                <Input id="db-host" placeholder="127.0.0.1" value={form.host} onChange={(e) => setForm({ ...form, host: e.target.value })} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="db-port">Port</Label>
                <Input id="db-port" value={form.port} onChange={(e) => setForm({ ...form, port: e.target.value })} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="db-user">Username</Label>
                <Input id="db-user" value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="db-pass">Password</Label>
                <Input id="db-pass" type="password" value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} />
              </div>
            </div>
            {message && <p className="text-sm text-red-400">{message}</p>}
            <div className="flex gap-2">
              <Button onClick={submitRegister} size="sm" disabled={busy || !form.name || !form.host}>
                {busy ? "Registering..." : "Register"}
              </Button>
              <Button variant="outline" size="sm" onClick={() => setShowRegister(false)}>Cancel</Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Table */}
      <DataState
        loading={loading}
        error={error}
        isEmpty={dbs.length === 0}
        onRetry={refetch}
        loadingLabel="Loading databases"
        emptyTitle="No monitored databases"
        emptyDescription="Register a database to start monitoring its security posture."
      >
        <Card className="border-border">
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Monitored Databases</CardTitle>
          </CardHeader>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border bg-muted/30 text-left">
                  <th className="px-4 py-3 font-medium text-muted-foreground">Name</th>
                  <th className="px-4 py-3 font-medium text-muted-foreground">Type</th>
                  <th className="px-4 py-3 font-medium text-muted-foreground">Host:Port</th>
                  <th className="px-4 py-3 font-medium text-muted-foreground">Status</th>
                  <th className="px-4 py-3 font-medium text-muted-foreground">Encrypted</th>
                  <th className="px-4 py-3 font-medium text-muted-foreground">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {dbs.map((db) => (
                  <tr key={db.id} className="hover:bg-accent/50">
                    <td className="px-4 py-3">
                      <div className="font-medium">{db.name}</div>
                      <div className="text-xs text-muted-foreground">{db.version || "—"}</div>
                    </td>
                    <td className="px-4 py-3">
                      <span className="inline-flex items-center gap-1.5 text-xs">
                        <Database className="h-3.5 w-3.5 text-muted-foreground" aria-hidden="true" />
                        {typeIcons[db.db_type] ?? db.db_type}
                      </span>
                    </td>
                    <td className="px-4 py-3 font-mono text-xs text-muted-foreground">
                      {db.host}:{db.port}
                    </td>
                    <td className="px-4 py-3">
                      <Badge variant={db.status === "online" ? "default" : "destructive"}>{db.status}</Badge>
                    </td>
                    <td className="px-4 py-3">
                      <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
                        <Shield className={`h-3.5 w-3.5 ${db.encrypted ? "text-green-400" : "text-red-400"}`} aria-hidden="true" />
                        {db.encrypted ? "Encrypted" : "Plaintext"}
                      </span>
                    </td>
                    <td className="px-4 py-3">
                      <Button variant="ghost" size="sm" onClick={() => removeDb(db)} className="text-red-400 hover:text-red-300" aria-label={`Remove ${db.name}`}>
                        <Trash2 className="h-4 w-4" aria-hidden="true" />
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      </DataState>
    </div>
  );
}


