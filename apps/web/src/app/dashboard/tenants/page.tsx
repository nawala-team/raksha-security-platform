"use client";

import { useState } from "react";
import { Building2, Plus, Ban, Pencil, X, RefreshCw } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { DataState } from "@/components/ui/data-state";
import { useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";

/** Mirrors the portal's `TenantResponse`. */
interface TenantRow {
  id: string;
  name: string;
  slug: string;
  settings: unknown;
  status: "active" | "suspended" | "deleted";
  created_at: string;
  updated_at: string;
}

export default function TenantsPage() {
  const { items, loading, error, refetch } = useApiList<TenantRow>(() => api.tenants.list());

  const [showCreate, setShowCreate] = useState(false);
  const [form, setForm] = useState({ name: "", slug: "", contact_email: "" });
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const submitCreate = async () => {
    setBusy(true);
    setMessage(null);
    try {
      await api.tenants.create({ 
        name: form.name, 
        slug: form.slug,
        contact_email: form.contact_email 
      });
      setShowCreate(false);
      setForm({ name: "", slug: "", contact_email: "" });
      refetch();
    } catch (err) {
      setMessage(err instanceof Error ? err.message : "Failed to create tenant");
    } finally {
      setBusy(false);
    }
  };

  const suspendTenant = async (t: TenantRow) => {
    if (t.status !== "active") return;
    if (!window.confirm(`Suspend tenant "${t.name}"?`)) return;
    try {
      await api.tenants.suspend(t.id);
      refetch();
    } catch (err) {
      alert(err instanceof Error ? err.message : "Failed to suspend tenant");
    }
  };

  const renameTenant = async (t: TenantRow) => {
    const name = window.prompt("Rename tenant:", t.name);
    if (!name || name === t.name) return;
    try {
      await api.tenants.update(t.id, { name });
      refetch();
    } catch (err) {
      alert(err instanceof Error ? err.message : "Failed to rename tenant");
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Tenant Management</h2>
          <p className="text-muted-foreground">Admin-only multi-tenant administration</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={refetch} aria-label="Refresh">
            <RefreshCw className="h-4 w-4" aria-hidden="true" />
          </Button>
          <Button onClick={() => setShowCreate((v) => !v)} className="gap-2">
            <Plus className="h-4 w-4" aria-hidden="true" /> Create Tenant
          </Button>
        </div>
      </div>

      {showCreate && (
        <Card className="border-border border-primary/30">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center justify-between text-lg">
              Create New Tenant
              <Button variant="ghost" size="icon" onClick={() => setShowCreate(false)} aria-label="Close">
                <X className="h-4 w-4" aria-hidden="true" />
              </Button>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <div className="space-y-2">
                <Label htmlFor="tenant-name">Tenant Name</Label>
                <Input id="tenant-name" placeholder="Acme Corp" value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="tenant-slug">Slug</Label>
                <Input id="tenant-slug" placeholder="acme-corp" value={form.slug} onChange={(e) => setForm({ ...form, slug: e.target.value })} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="tenant-email">Contact Email</Label>
                <Input id="tenant-email" type="email" placeholder="admin@acme.com" value={form.contact_email} onChange={(e) => setForm({ ...form, contact_email: e.target.value })} />
              </div>
            </div>
            {message && <p className="text-sm text-red-400">{message}</p>}
            <div className="flex gap-2">
              <Button onClick={submitCreate} size="sm" disabled={busy || !form.name || !form.slug || !form.contact_email}>
                {busy ? "Creating..." : "Create"}
              </Button>
              <Button variant="outline" size="sm" onClick={() => setShowCreate(false)}>Cancel</Button>
            </div>
          </CardContent>
        </Card>
      )}

      <DataState
        loading={loading}
        error={error}
        isEmpty={items.length === 0}
        onRetry={refetch}
        loadingLabel="Loading tenants"
        emptyTitle="No tenants"
        emptyDescription="No tenant organizations exist yet."
      >
        <Card className="border-border">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center gap-2 text-lg">
              <Building2 className="h-5 w-5 text-blue-400" aria-hidden="true" /> Tenants
            </CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            <div className="overflow-x-auto">
              <table className="w-full text-sm" role="table">
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th className="px-4 py-3 text-left font-medium text-muted-foreground">Name</th>
                    <th className="px-4 py-3 text-left font-medium text-muted-foreground">Slug</th>
                    <th className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                    <th className="px-4 py-3 text-left font-medium text-muted-foreground">Created</th>
                    <th className="px-4 py-3 text-left font-medium text-muted-foreground">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {items.map((tenant) => (
                    <tr key={tenant.id} className="border-b border-border hover:bg-muted/20">
                      <td className="px-4 py-3 font-medium text-foreground">{tenant.name}</td>
                      <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{tenant.slug}</td>
                      <td className="px-4 py-3">
                        <Badge variant={tenant.status === "active" ? "default" : tenant.status === "suspended" ? "destructive" : "outline"}>
                          {tenant.status}
                        </Badge>
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">
                        {new Date(tenant.created_at).toLocaleDateString()}
                      </td>
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-1">
                          <Button variant="ghost" size="sm" onClick={() => renameTenant(tenant)} aria-label={`Rename ${tenant.name}`}>
                            <Pencil className="h-4 w-4" aria-hidden="true" />
                          </Button>
                          {tenant.status === "active" && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => suspendTenant(tenant)}
                              className="text-red-400 hover:text-red-300"
                              aria-label={`Suspend ${tenant.name}`}
                            >
                              <Ban className="h-4 w-4" aria-hidden="true" />
                            </Button>
                          )}
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>
      </DataState>
    </div>
  );
}

