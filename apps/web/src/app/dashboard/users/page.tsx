"use client";

import { useState } from "react";
import {
  Users,
  Shield,
  ShieldCheck,
  Eye,
  UserCog,
  Plus,
  Pencil,
  Trash2,
  X,
  RefreshCw,
} from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui/select";
import { DataState } from "@/components/ui/data-state";
import { useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import type { UserRole } from "@/types";

/** Mirrors the portal's `UserResponse`. */
interface UserRecord {
  id: string;
  email: string;
  name: string;
  role: UserRole;
  is_active: boolean;
  last_login_at: string | null;
  created_at: string;
}

const roleConfig: Record<UserRole, { icon: typeof Shield; color: string; label: string }> = {
  super_admin: { icon: ShieldCheck, color: "text-purple-400", label: "Super Admin" },
  admin: { icon: Shield, color: "text-red-400", label: "Admin" },
  analyst: { icon: Eye, color: "text-blue-400", label: "Analyst" },
  operator: { icon: UserCog, color: "text-yellow-400", label: "Operator" },
  viewer: { icon: Users, color: "text-green-400", label: "Viewer" },
};

const ROLE_OPTIONS: UserRole[] = ["super_admin", "admin", "analyst", "operator", "viewer"];

const emptyForm = { email: "", name: "", password: "", role: "viewer" as UserRole };

export default function UsersPage() {
  const { items, loading, error, refetch } = useApiList<UserRecord>(() =>
    api.users.list()
  );

  const [showModal, setShowModal] = useState(false);
  const [editing, setEditing] = useState<UserRecord | null>(null);
  const [form, setForm] = useState(emptyForm);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<{ kind: "ok" | "err"; text: string } | null>(null);

  const openAdd = () => {
    setEditing(null);
    setForm(emptyForm);
    setMessage(null);
    setShowModal(true);
  };

  const openEdit = (u: UserRecord) => {
    setEditing(u);
    setForm({ email: u.email, name: u.name, password: "", role: u.role });
    setMessage(null);
    setShowModal(true);
  };

  const submit = async () => {
    setBusy(true);
    setMessage(null);
    try {
      if (editing) {
        const data: Record<string, unknown> = { email: form.email, name: form.name };
        if (form.password) data.password = form.password;
        await api.users.update(editing.id, data);
        if (form.role !== editing.role) {
          await api.users.updateRole(editing.id, form.role);
        }
        setMessage({ kind: "ok", text: "User updated successfully." });
      } else {
        await api.users.create({
          email: form.email,
          name: form.name,
          password: form.password,
          role: form.role,
        });
        setMessage({ kind: "ok", text: "User created successfully." });
      }
      setShowModal(false);
      refetch();
    } catch (err) {
      setMessage({
        kind: "err",
        text: err instanceof Error ? err.message : "Request failed",
      });
    } finally {
      setBusy(false);
    }
  };

  const changeRole = async (u: UserRecord, role: UserRole) => {
    if (role === u.role) return;
    try {
      await api.users.updateRole(u.id, role);
      refetch();
    } catch (err) {
      alert(err instanceof Error ? err.message : "Failed to update role");
    }
  };

  const removeUser = async (u: UserRecord) => {
    if (!window.confirm(`Delete user ${u.email}? This cannot be undone.`)) return;
    try {
      await api.users.delete(u.id);
      refetch();
    } catch (err) {
      alert(err instanceof Error ? err.message : "Failed to delete user");
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Users &amp; Roles</h2>
          <p className="text-muted-foreground">Manage user accounts, roles and permissions</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" onClick={refetch} aria-label="Refresh">
            <RefreshCw className="h-4 w-4" aria-hidden="true" />
          </Button>
          <Button onClick={openAdd}>
            <Plus className="h-4 w-4 mr-2" aria-hidden="true" />Add User
          </Button>
        </div>
      </div>

      <DataState
        loading={loading}
        error={error}
        isEmpty={items.length === 0}
        onRetry={refetch}
        loadingLabel="Loading users"
        emptyTitle="No users yet"
        emptyDescription="Create your first user account with the Add User button."
      >
        <Card className="border-border">
          <CardContent className="p-0">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-border text-left">
                    <th className="p-4 font-medium text-muted-foreground">User</th>
                    <th className="p-4 font-medium text-muted-foreground">Role</th>
                    <th className="p-4 font-medium text-muted-foreground">Status</th>
                    <th className="p-4 font-medium text-muted-foreground">Last Login</th>
                    <th className="p-4 font-medium text-muted-foreground">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {items.map((user) => {
                    const config = roleConfig[user.role] ?? roleConfig.viewer;
                    const RoleIcon = config.icon;
                    return (
                      <tr key={user.id} className="hover:bg-accent/50">
                        <td className="p-4">
                          <div className="font-medium">{user.name}</div>
                          <div className="text-xs text-muted-foreground">{user.email}</div>
                        </td>
                        <td className="p-4">
                          <Select
                            value={user.role}
                            onValueChange={(v) => changeRole(user, v as UserRole)}
                          >
                            <SelectTrigger className="w-44">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              {ROLE_OPTIONS.map((r) => (
                                <SelectItem key={r} value={r}>
                                  <span className="flex items-center gap-2">
                                    <RoleIcon className={`h-3.5 w-3.5 ${config.color}`} aria-hidden="true" />
                                    {roleConfig[r].label}
                                  </span>
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </td>
                        <td className="p-4">
                          <Badge variant={user.is_active ? "default" : "destructive"}>
                            {user.is_active ? "Active" : "Disabled"}
                          </Badge>
                        </td>
                        <td className="p-4 text-xs text-muted-foreground">
                          {user.last_login_at
                            ? new Date(user.last_login_at).toLocaleString()
                            : "Never"}
                        </td>
                        <td className="p-4">
                          <div className="flex items-center gap-1">
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => openEdit(user)}
                              aria-label={`Edit ${user.email}`}
                            >
                              <Pencil className="h-4 w-4" aria-hidden="true" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => removeUser(user)}
                              className="text-red-400 hover:text-red-300"
                              aria-label={`Delete ${user.email}`}
                            >
                              <Trash2 className="h-4 w-4" aria-hidden="true" />
                            </Button>
                          </div>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>
      </DataState>

      {showModal && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
          role="dialog"
          aria-modal="true"
          aria-labelledby="user-modal-title"
        >
          <div className="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-xl">
            <div className="mb-4 flex items-center justify-between">
              <h3 id="user-modal-title" className="text-lg font-semibold text-foreground">
                {editing ? "Edit User" : "Add New User"}
              </h3>
              <Button variant="ghost" size="icon" onClick={() => setShowModal(false)} aria-label="Close">
                <X className="h-4 w-4" aria-hidden="true" />
              </Button>
            </div>
            <div className="space-y-4">
              <div className="space-y-1">
                <Label htmlFor="user-name">Full name</Label>
                <Input
                  id="user-name"
                  value={form.name}
                  onChange={(e) => setForm({ ...form, name: e.target.value })}
                  placeholder="Jane Doe"
                />
              </div>
              <div className="space-y-1">
                <Label htmlFor="user-email">Email</Label>
                <Input
                  id="user-email"
                  type="email"
                  value={form.email}
                  onChange={(e) => setForm({ ...form, email: e.target.value })}
                  placeholder="user@raksha.local"
                />
              </div>
              <div className="space-y-1">
                <Label htmlFor="user-password">
                  {editing ? "New password (leave blank to keep)" : "Password"}
                </Label>
                <Input
                  id="user-password"
                  type="password"
                  value={form.password}
                  onChange={(e) => setForm({ ...form, password: e.target.value })}
                  placeholder="At least 8 characters"
                />
              </div>
              <div className="space-y-1">
                <Label htmlFor="user-role">Role</Label>
                <Select
                  value={form.role}
                  onValueChange={(v) => setForm({ ...form, role: v as UserRole })}
                >
                  <SelectTrigger id="user-role" className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {ROLE_OPTIONS.map((r) => (
                      <SelectItem key={r} value={r}>
                        <span className="capitalize">{roleConfig[r].label}</span>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {message && (
                <p
                  className={`text-sm ${
                    message.kind === "ok" ? "text-green-400" : "text-red-400"
                  }`}
                >
                  {message.text}
                </p>
              )}

              <div className="flex justify-end gap-2 pt-2">
                <Button variant="outline" onClick={() => setShowModal(false)}>
                  Cancel
                </Button>
                <Button
                  onClick={submit}
                  disabled={
                    busy || !form.email || !form.name || (!editing && !form.password)
                  }
                >
                  {busy ? "Saving..." : editing ? "Save Changes" : "Create User"}
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}


