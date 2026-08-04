"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import {
  Shield,
  LayoutDashboard,
  AlertTriangle,
  Server,
  Network,
  Database,
  FileSearch,
  ClipboardCheck,
  ScrollText,
  Users,
  FileText,
  Settings,
  LogOut,
  Bot,
  Globe,
  Bug,
  Siren,
  Radar,
  Container,
  Crosshair,
  ShieldCheck,
  Eye,
  HardDrive,
  Building2,
  Hexagon,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Separator } from "@/components/ui/separator";
import { api, apiClient } from "@/lib/api";
import { clearStoredToken } from "@/lib/auth";
import { useApiList } from "@/hooks/use-api-data";

const navigation = [
  { name: "Dashboard", href: "/dashboard", icon: LayoutDashboard },
  { name: "Alerts", href: "/dashboard/alerts", icon: AlertTriangle },
  { name: "Incidents", href: "/dashboard/incidents", icon: Siren },
  { name: "Servers", href: "/dashboard/servers", icon: Server },
  { name: "Containers", href: "/dashboard/containers", icon: Container },
  { name: "Network", href: "/dashboard/network", icon: Network },
  { name: "Database", href: "/dashboard/database", icon: Database },
  { name: "FIM", href: "/dashboard/fim", icon: FileSearch },
  { name: "Agents", href: "/dashboard/agents", icon: Bot },
  { name: "Vulnerabilities", href: "/dashboard/vulnerabilities", icon: Bug },
  { name: "Attack Surface", href: "/dashboard/attack-surface", icon: Radar },
  { name: "Threat Intel", href: "/dashboard/threat-intel", icon: Globe },
  { name: "Hunting", href: "/dashboard/hunting", icon: Crosshair },
  { name: "Compliance", href: "/dashboard/compliance", icon: ClipboardCheck },
  { name: "GRC", href: "/dashboard/grc", icon: ShieldCheck },
  { name: "Dark Web", href: "/dashboard/darkweb", icon: Eye },
  { name: "Honeypots", href: "/dashboard/honeypots", icon: Hexagon },
  { name: "Backups", href: "/dashboard/backups", icon: HardDrive },
  { name: "Audit Trail", href: "/dashboard/audit", icon: ScrollText },
  { name: "Users", href: "/dashboard/users", icon: Users },
  { name: "Tenants", href: "/dashboard/tenants", icon: Building2 },
  { name: "Documents", href: "/dashboard/documents", icon: FileText },
  { name: "Settings", href: "/dashboard/settings", icon: Settings },
];

export function Sidebar() {
  const pathname = usePathname();
  const router = useRouter();
  const { total: alertCount } = useApiList(() => api.alerts.list({ per_page: "1" }));

  const handleLogout = async () => {
    try {
      await api.auth.logout();
    } catch {
      // Still clear local state if the access token is already expired/revoked.
    } finally {
      apiClient.clearToken();
      clearStoredToken();
      router.replace("/login");
    }
  };

  return (
    <aside className="fixed inset-y-0 left-0 z-50 flex w-64 flex-col border-r border-border bg-card">
      {/* Logo */}
      <div className="flex h-16 items-center gap-2 px-6">
        <Shield className="h-8 w-8 text-primary" />
        <div>
          <h1 className="text-lg font-bold text-foreground">Raksha</h1>
          <p className="text-xs text-muted-foreground">Security Platform</p>
        </div>
      </div>

      <Separator />

      {/* Navigation */}
      <nav className="flex-1 space-y-1 overflow-y-auto px-3 py-4" aria-label="Main navigation">
        {navigation.map((item) => {
          const isActive =
            pathname === item.href ||
            (item.href !== "/dashboard" && pathname.startsWith(item.href));

          return (
            <Link
              key={item.name}
              href={item.href}
              className={cn(
                "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                isActive
                  ? "bg-primary/10 text-primary"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              )}
              aria-current={isActive ? "page" : undefined}
            >
              <item.icon className="h-5 w-5" />
              {item.name}
              {item.name === "Alerts" && alertCount > 0 && (
                <span className="ml-auto flex h-5 w-5 items-center justify-center rounded-full bg-destructive text-[10px] font-bold text-destructive-foreground">
                  {alertCount > 99 ? "99+" : alertCount}
                </span>
              )}
            </Link>
          );
        })}
      </nav>

      <Separator />

      {/* Footer */}
      <div className="p-3">
        <button
          type="button"
          onClick={handleLogout}
          className="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
          aria-label="Sign out"
        >
          <LogOut className="h-5 w-5" />
          Sign Out
        </button>
      </div>
    </aside>
  );
}
