import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";

export function StepDatabase({
  dbConfig,
  setDbConfig,
}: {
  dbConfig: { host: string; port: string; name: string; username: string; password: string };
  setDbConfig: (config: typeof dbConfig) => void;
}) {
  return (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-2">
          <Label htmlFor="db-host">Host</Label>
          <Input id="db-host" value={dbConfig.host} onChange={(e) => setDbConfig({ ...dbConfig, host: e.target.value })} />
        </div>
        <div className="space-y-2">
          <Label htmlFor="db-port">Port</Label>
          <Input id="db-port" value={dbConfig.port} onChange={(e) => setDbConfig({ ...dbConfig, port: e.target.value })} />
        </div>
      </div>
      <div className="space-y-2">
        <Label htmlFor="db-name">Database Name</Label>
        <Input id="db-name" value={dbConfig.name} onChange={(e) => setDbConfig({ ...dbConfig, name: e.target.value })} />
      </div>
      <div className="space-y-2">
        <Label htmlFor="db-user">Username</Label>
        <Input id="db-user" value={dbConfig.username} onChange={(e) => setDbConfig({ ...dbConfig, username: e.target.value })} />
      </div>
      <div className="space-y-2">
        <Label htmlFor="db-pass">Password</Label>
        <Input id="db-pass" type="password" value={dbConfig.password} onChange={(e) => setDbConfig({ ...dbConfig, password: e.target.value })} placeholder="Enter database password" />
      </div>
    </div>
  );
}

export function StepAdmin({
  adminConfig,
  setAdminConfig,
}: {
  adminConfig: { name: string; email: string; password: string; confirmPassword: string };
  setAdminConfig: (config: typeof adminConfig) => void;
}) {
  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="admin-name">Full Name</Label>
        <Input id="admin-name" value={adminConfig.name} onChange={(e) => setAdminConfig({ ...adminConfig, name: e.target.value })} placeholder="Security Administrator" />
      </div>
      <div className="space-y-2">
        <Label htmlFor="admin-email">Email Address</Label>
        <Input id="admin-email" type="email" value={adminConfig.email} onChange={(e) => setAdminConfig({ ...adminConfig, email: e.target.value })} placeholder="admin@organization.com" />
      </div>
      <div className="space-y-2">
        <Label htmlFor="admin-pass">Password</Label>
        <Input id="admin-pass" type="password" value={adminConfig.password} onChange={(e) => setAdminConfig({ ...adminConfig, password: e.target.value })} placeholder="Minimum 12 characters" />
      </div>
      <div className="space-y-2">
        <Label htmlFor="admin-confirm">Confirm Password</Label>
        <Input id="admin-confirm" type="password" value={adminConfig.confirmPassword} onChange={(e) => setAdminConfig({ ...adminConfig, confirmPassword: e.target.value })} placeholder="Re-enter password" />
      </div>
      <p className="text-xs text-muted-foreground">
        Password must contain at least 12 characters, including uppercase, lowercase, numbers, and special characters.
      </p>
    </div>
  );
}
