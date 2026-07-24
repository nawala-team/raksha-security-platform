import * as React from "react";
import { cn } from "@/lib/utils";

export interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "default" | "secondary" | "destructive" | "outline" | "critical" | "high" | "medium" | "low";
}

function Badge({ className, variant = "default", ...props }: BadgeProps) {
  const variantClasses: Record<string, string> = {
    default: "border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
    secondary: "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
    destructive: "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
    outline: "text-foreground",
    critical: "border-transparent bg-red-500/20 text-red-400 border-red-500/30",
    high: "border-transparent bg-orange-500/20 text-orange-400 border-orange-500/30",
    medium: "border-transparent bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
    low: "border-transparent bg-green-500/20 text-green-400 border-green-500/30",
  };

  return (
    <div
      className={cn(
        "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
        variantClasses[variant],
        className
      )}
      {...props}
    />
  );
}

export { Badge };
