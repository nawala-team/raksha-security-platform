import { Users, Shield, Eye, UserCog, Plus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { User, UserRole } from "@/types";

const mockUsers: User[] = [
  { id: "1", email: "admin@raksha.io", name: "System Administrator", role: "admin", mfaEnabled: true, createdAt: "2024-01-01", lastLogin: "2024-01-15T10:30:00Z" },
  { id: "2", email: "analyst@raksha.io", name: "Sarah Chen", role: "analyst", mfaEnabled: true, createdAt: "2024-01-03", lastLogin: "2024-01-15T09:45:00Z" },
  { id: "3", email: "ops@raksha.io", name: "James Wilson", role: "operator", mfaEnabled: true, createdAt: "2024-01-05", lastLogin: "2024-01-15T08:20:00Z" },
  { id: "4", email: "viewer@raksha.io", name: "Alex Kumar", role: "viewer", mfaEnabled: false, createdAt: "2024-01-10", lastLogin: "2024-01-14T16:00:00Z" },
  { id: "5", email: "security@raksha.io", name: "Maria Lopez", role: "analyst", mfaEnabled: true, createdAt: "2024-01-07", lastLogin: "2024-01-15T10:15:00Z" },
];

const roleConfig: Record<UserRole, { icon: typeof Shield; color: string }> = {
  admin: { icon: Shield, color: "text-red-400" },
  analyst: { icon: Eye, color: "text-blue-400" },
  operator: { icon: UserCog, color: "text-yellow-400" },
  viewer: { icon: Users, color: "text-green-400" },
};

export default function UsersPage() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Users & Roles</h2>
          <p className="text-muted-foreground">Manage user accounts and permissions</p>
        </div>
        <Button><Plus className="h-4 w-4 mr-2" />Add User</Button>
      </div>

      <Card className="border-border">
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border text-left">
                  <th className="p-4 font-medium text-muted-foreground">User</th>
                  <th className="p-4 font-medium text-muted-foreground">Role</th>
                  <th className="p-4 font-medium text-muted-foreground">MFA</th>
                  <th className="p-4 font-medium text-muted-foreground">Last Login</th>
                  <th className="p-4 font-medium text-muted-foreground">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {mockUsers.map((user) => {
                  const config = roleConfig[user.role];
                  const RoleIcon = config.icon;
                  return (
                    <tr key={user.id} className="hover:bg-accent/50">
                      <td className="p-4">
                        <div>
                          <p className="font-medium">{user.name}</p>
                          <p className="text-xs text-muted-foreground">{user.email}</p>
                        </div>
                      </td>
                      <td className="p-4">
                        <div className="flex items-center gap-2">
                          <RoleIcon className={`h-4 w-4 ${config.color}`} />
                          <span className="capitalize">{user.role}</span>
                        </div>
                      </td>
                      <td className="p-4">
                        <Badge variant={user.mfaEnabled ? "default" : "destructive"}>
                          {user.mfaEnabled ? "Enabled" : "Disabled"}
                        </Badge>
                      </td>
                      <td className="p-4 text-xs text-muted-foreground">
                        {user.lastLogin ? new Date(user.lastLogin).toLocaleString() : "Never"}
                      </td>
                      <td className="p-4">
                        <Button variant="outline" size="sm">Edit</Button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
