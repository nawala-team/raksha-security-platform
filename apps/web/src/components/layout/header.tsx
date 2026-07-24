"use client";

import { Bell, Search, User } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export function Header() {
  return (
    <header className="sticky top-0 z-40 flex h-16 items-center gap-4 border-b border-border bg-card/95 px-6 backdrop-blur supports-[backdrop-filter]:bg-card/60">
      {/* Search */}
      <div className="relative flex-1 max-w-md">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          placeholder="Search alerts, servers, events..."
          className="pl-9 bg-background"
          aria-label="Search"
        />
      </div>

      <div className="flex items-center gap-2 ml-auto">
        {/* System Status Indicator */}
        <div className="flex items-center gap-2 rounded-lg border border-border px-3 py-1.5">
          <span className="relative flex h-2.5 w-2.5">
            <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75"></span>
            <span className="relative inline-flex h-2.5 w-2.5 rounded-full bg-green-500"></span>
          </span>
          <span className="text-xs font-medium text-muted-foreground">
            System Operational
          </span>
        </div>

        {/* Notifications */}
        <Button variant="ghost" size="icon" className="relative" aria-label="Notifications">
          <Bell className="h-5 w-5" />
          <span className="absolute -top-0.5 -right-0.5 flex h-4 w-4 items-center justify-center rounded-full bg-destructive text-[10px] font-bold text-destructive-foreground">
            5
          </span>
        </Button>

        {/* User Avatar */}
        <Button variant="ghost" size="icon" className="rounded-full" aria-label="User menu">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20 text-primary">
            <User className="h-4 w-4" />
          </div>
        </Button>
      </div>
    </header>
  );
}
