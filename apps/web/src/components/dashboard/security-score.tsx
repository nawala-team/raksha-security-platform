"use client";

import { Shield, TrendingUp, TrendingDown, Minus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

interface SecurityScoreProps {
  score?: number;
  trend?: "improving" | "stable" | "declining";
  categories?: {
    network: number;
    endpoint: number;
    identity: number;
    data: number;
    application: number;
  };
}

export function SecurityScore({
  score = 87,
  trend = "improving",
  categories = {
    network: 92,
    endpoint: 85,
    identity: 88,
    data: 80,
    application: 90,
  },
}: SecurityScoreProps) {
  const getScoreColor = (value: number) => {
    if (value >= 90) return "text-green-400";
    if (value >= 75) return "text-yellow-400";
    if (value >= 50) return "text-orange-400";
    return "text-red-400";
  };

  const getScoreRingColor = (value: number) => {
    if (value >= 90) return "stroke-green-400";
    if (value >= 75) return "stroke-yellow-400";
    if (value >= 50) return "stroke-orange-400";
    return "stroke-red-400";
  };

  const TrendIcon = trend === "improving" ? TrendingUp : trend === "declining" ? TrendingDown : Minus;
  const trendColor = trend === "improving" ? "text-green-400" : trend === "declining" ? "text-red-400" : "text-yellow-400";
  const trendLabel = trend === "improving" ? "+3.2% this week" : trend === "declining" ? "-2.1% this week" : "No change";

  const circumference = 2 * Math.PI * 45;
  const strokeDashoffset = circumference - (score / 100) * circumference;

  return (
    <Card className="border-border">
      <CardHeader className="pb-2">
        <CardTitle className="flex items-center gap-2 text-base">
          <Shield className="h-5 w-5 text-primary" />
          Security Score
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="flex items-center gap-6">
          {/* Circular Score */}
          <div className="relative flex h-28 w-28 items-center justify-center">
            <svg className="h-28 w-28 -rotate-90" viewBox="0 0 100 100" aria-hidden="true">
              <circle
                cx="50"
                cy="50"
                r="45"
                fill="none"
                stroke="currentColor"
                strokeWidth="6"
                className="text-muted/30"
              />
              <circle
                cx="50"
                cy="50"
                r="45"
                fill="none"
                strokeWidth="6"
                strokeLinecap="round"
                strokeDasharray={circumference}
                strokeDashoffset={strokeDashoffset}
                className={getScoreRingColor(score)}
              />
            </svg>
            <div className="absolute flex flex-col items-center">
              <span className={cn("text-2xl font-bold", getScoreColor(score))}>
                {score}
              </span>
              <span className="text-xs text-muted-foreground">/100</span>
            </div>
          </div>

          {/* Categories */}
          <div className="flex-1 space-y-2">
            {Object.entries(categories).map(([key, value]) => (
              <div key={key} className="flex items-center justify-between">
                <span className="text-xs capitalize text-muted-foreground">
                  {key}
                </span>
                <div className="flex items-center gap-2">
                  <div className="h-1.5 w-16 overflow-hidden rounded-full bg-muted">
                    <div
                      className={cn(
                        "h-full rounded-full",
                        value >= 90
                          ? "bg-green-400"
                          : value >= 75
                          ? "bg-yellow-400"
                          : value >= 50
                          ? "bg-orange-400"
                          : "bg-red-400"
                      )}
                      style={{ width: `${value}%` }}
                    />
                  </div>
                  <span className={cn("text-xs font-medium", getScoreColor(value))}>
                    {value}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Trend */}
        <div className="mt-4 flex items-center gap-1.5 border-t border-border pt-3">
          <TrendIcon className={cn("h-4 w-4", trendColor)} />
          <span className={cn("text-xs font-medium", trendColor)}>
            {trendLabel}
          </span>
        </div>
      </CardContent>
    </Card>
  );
}
