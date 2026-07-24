import {
  AlertTriangle,
  Server,
  ShieldCheck,
  Activity,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { SecurityScore } from "@/components/dashboard/security-score";
import { AlertFeed } from "@/components/dashboard/alert-feed";

export default function DashboardPage() {
  const stats = [
    {
      title: "Active Alerts",
      value: "12",
      change: "+3 from yesterday",
      icon: AlertTriangle,
      color: "text-red-400",
      bgColor: "bg-red-400/10",
    },
    {
      title: "Servers Online",
      value: "48/50",
      change: "96% uptime",
      icon: Server,
      color: "text-green-400",
      bgColor: "bg-green-400/10",
    },
    {
      title: "Threats Blocked",
      value: "1,284",
      change: "+127 today",
      icon: ShieldCheck,
      color: "text-blue-400",
      bgColor: "bg-blue-400/10",
    },
    {
      title: "Compliance Score",
      value: "94%",
      change: "All frameworks passing",
      icon: Activity,
      color: "text-emerald-400",
      bgColor: "bg-emerald-400/10",
    },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Dashboard</h2>
        <p className="text-muted-foreground">
          Security overview and real-time monitoring
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.title} className="border-border">
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className={`rounded-lg p-2.5 ${stat.bgColor}`}>
                  <stat.icon className={`h-5 w-5 ${stat.color}`} />
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">{stat.title}</p>
                  <p className="text-2xl font-bold text-foreground">
                    {stat.value}
                  </p>
                  <p className="text-xs text-muted-foreground">{stat.change}</p>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Main Content Grid */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <SecurityScore />
        <AlertFeed />
      </div>

      {/* Threat Map Placeholder */}
      <Card className="border-border">
        <CardHeader>
          <CardTitle className="text-base">Threat Activity Map</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex h-64 items-center justify-center rounded-lg border border-dashed border-border">
            <p className="text-sm text-muted-foreground">
              Real-time threat visualization will be displayed here
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
