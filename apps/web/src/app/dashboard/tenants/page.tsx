"use client";

import { useState } from "react";
import {
  Building2, Plus, Users, Bot, Ban, CheckCircle2, XCircle,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface Tenant {
  id: string;
  name: string;
  slug: string;
  status: "Active" | "Suspended";
  users: number;
  agents: number;
  created: string;
}

const mockTenants: Tenant[] = [
  { id: "1", name: "Acme Corporation", slug: "acme-corp", status: "Active", users: 45, agents: 12, created: "2023-06-15" },
  { id: "2", name: "TechStart Inc", slug: "techstart", status: "Active", users: 18, agents: 6, created: "2023-09-20" },
  { id: "3", name: "SecureBank Ltd", slug: "securebank", status: "Active", users: 120, agents: 34, created: "2023-03-10" },
  { id: "4", name: "CloudNet Systems", slug: "cloudnet", status: "Suspended", users: 8, agents: 3, created: "2023-11-01" },
  { id: "5", name: "DataFlow Analytics", slug: "dataflow", status: "Active", users: 32, agents: 9, created: "2024-01-05" },
];

export default function TenantsPage() {
  const [tenants, setTenants] = useState(mockTenants);
  const [showCreate, setShowCreate] = useState(false);
  const [form, setForm] = useState({ name: "", slug: "", adminEmail: "" });

  const handleCreate = () => {
    if (!form.name || !form.slug || !form.adminEmail) return;
    setTenants((prev) => [
      ...prev,
      {
        id: Date.now().toString(),
        name: form.name,
        slug: form.slug,
        status: "Active",
        users: 1,
        agents: 0,
        created: new Date().toISOString().split("T")[0],
      },
    ]);
    setForm({ name: "", slug: "", adminEmail: "" });
    setShowCreate(false);
  };

  const toggleStatus = (id: string) => {
    setTenants((prev) =>
      prev.map((t) =>
        t.id === id ? { ...t, status: t.status === "Active" ? "Suspended" : "Active" } : t
      )
    );
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Tenant Management</h2>
          <p className="text-muted-foreground">Admin-only multi-tenant administration</p>
        </div>
        <Button onClick={() => setShowCreate(!showCreate)} className="gap-2">
          <Plus className="h-4 w-4" /> Create Tenant
        </Button>
      </div>

      {showCreate && (
        <Card className="border-border border-primary/30">
          <CardHeader className="pb-3">
            <CardTitle className="text-lg">Create New Tenant</CardTitle>
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
                <Label htmlFor="tenant-email">Admin Email</Label>
                <Input id="tenant-email" type="email" placeholder="admin@acme.com" value={form.adminEmail} onChange={(e) => setForm({ ...form, adminEmail: e.target.value })} />
              </div>
            </div>
            <div className="flex gap-2">
              <Button onClick={handleCreate} size="sm">Create</Button>
              <Button variant="outline" size="sm" onClick={() => setShowCreate(false)}>Cancel</Button>
            </div>
          </CardContent>
        </Card>
      )}

      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Building2 className="h-5 w-5 text-blue-400" /> Tenants
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
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Users</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Agents</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Created</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Actions</th>
                </tr>
              </thead>
              <tbody>
                {tenants.map((tenant) => (
                  <tr key={tenant.id} className="border-b border-border hover:bg-muted/20">
                    <td className="px-4 py-3 font-medium text-foreground">{tenant.name}</td>
                    <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{tenant.slug}</td>
                    <td className="px-4 py-3">
                      <span className={`inline-flex items-center gap-1.5 text-xs font-medium ${tenant.status === "Active" ? "text-green-400" : "text-red-400"}`}>
                        {tenant.status === "Active" ? <CheckCircle2 className="h-3.5 w-3.5" /> : <XCircle className="h-3.5 w-3.5" />}
                        {tenant.status}
                      </span>
                    </td>
                    <td className="px-4 py-3"><span className="flex items-center gap-1 text-muted-foreground"><Users className="h-3.5 w-3.5" />{tenant.users}</span></td>
                    <td className="px-4 py-3"><span className="flex items-center gap-1 text-muted-foreground"><Bot className="h-3.5 w-3.5" />{tenant.agents}</span></td>
                    <td className="px-4 py-3 text-muted-foreground">{tenant.created}</td>
                    <td className="px-4 py-3">
                      <Button variant="outline" size="sm" onClick={() => toggleStatus(tenant.id)} className={`gap-1 text-xs ${tenant.status === "Active" ? "hover:text-red-400" : "hover:text-green-400"}`}>
                        {tenant.status === "Active" ? <><Ban className="h-3 w-3" /> Suspend</> : <><CheckCircle2 className="h-3 w-3" /> Activate</>}
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
