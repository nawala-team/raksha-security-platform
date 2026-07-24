"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  Shield,
  LayoutDashboard,
  AlertTriangle,
  Server,
  Network,
  Database,
  ClipboardCheck,
  ScrollText,
  Users,
  FileText,
  Settings,
  LogOut,
  Bot,
  Globe,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Separator } from "@/components/ui/separator";

const navigation = [
  { name: "Dashboard", href: "/dashboard", icon: LayoutDashboard },
  { name: "Alerts", href: "/dashboard/alerts", icon: AlertTriangle },
  { name: "Servers", href: "/dashboard/servers", icon: Server },
  { name: "Network", href: "/dashboard/network", icon: Network },
  { name: "Database", href: "/dashboard/database", icon: Database },
  { name: "Agents", href: "/dashboard/agents", icon: Bot },
  { name: "Threat Intel", href: "/dashboard/threat-intel", icon: Globe },
  { name: "Compliance", href: "/dashboard/compliance", icon: ClipboardCheck },
  { name: "Audit Trail", href: "/dashboard/audit", icon: ScrollText },
  { name: "Users", href: "/dashboard/users", icon: Users },
  { name: "Documents", href: "/dashboard/documents", icon: FileText },
  { name: "Settings", href: "/dashboard/settings", icon: Settings },
];

export function Sidebar() {
  const pathname = usePathname();

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
              {item.name === "Alerts" && (
                <span className="ml-auto flex h-5 w-5 items-center justify-center rounded-full bg-destructive text-[10px] font-bold text-destructive-foreground">
                  3
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
