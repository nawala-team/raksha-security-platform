import { Network, Globe, Shield, ArrowUpDown } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

const mockEvents = [
  { id: "1", sourceIp: "203.0.113.45", destinationIp: "10.0.1.10", protocol: "TCP", port: 443, action: "allow" as const, timestamp: "2024-01-15T10:30:00Z", bytesTransferred: 15234, threat: undefined },
  { id: "2", sourceIp: "198.51.100.23", destinationIp: "10.0.1.10", protocol: "TCP", port: 22, action: "block" as const, timestamp: "2024-01-15T10:29:00Z", bytesTransferred: 0, threat: "high" as const },
  { id: "3", sourceIp: "10.0.1.11", destinationIp: "192.0.2.100", protocol: "TCP", port: 8080, action: "monitor" as const, timestamp: "2024-01-15T10:28:00Z", bytesTransferred: 45678, threat: "medium" as const },
  { id: "4", sourceIp: "10.0.2.10", destinationIp: "10.0.3.10", protocol: "TCP", port: 6379, action: "allow" as const, timestamp: "2024-01-15T10:27:00Z", bytesTransferred: 8923, threat: undefined },
  { id: "5", sourceIp: "172.16.0.50", destinationIp: "10.0.1.20", protocol: "UDP", port: 53, action: "allow" as const, timestamp: "2024-01-15T10:26:00Z", bytesTransferred: 512, threat: undefined },
  { id: "6", sourceIp: "45.33.32.156", destinationIp: "10.0.1.10", protocol: "TCP", port: 3389, action: "block" as const, timestamp: "2024-01-15T10:25:00Z", bytesTransferred: 0, threat: "critical" as const },
];

const actionColors = { allow: "text-green-400", block: "text-red-400", monitor: "text-yellow-400" };

export default function NetworkPage() {
  const stats = [
    { label: "Total Traffic", value: "2.4 TB", sub: "Last 24h" },
    { label: "Blocked Requests", value: "1,284", sub: "+12% today" },
    { label: "Active Connections", value: "3,421", sub: "Across all hosts" },
    { label: "Firewall Rules", value: "156", sub: "12 updated today" },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Network Security</h2>
        <p className="text-muted-foreground">Network traffic monitoring and firewall management</p>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.label} className="border-border">
            <CardContent className="p-4">
              <p className="text-sm text-muted-foreground">{stat.label}</p>
              <p className="text-2xl font-bold text-foreground">{stat.value}</p>
              <p className="text-xs text-muted-foreground">{stat.sub}</p>
            </CardContent>
          </Card>
        ))}
      </div>

      <Card className="border-border">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <ArrowUpDown className="h-5 w-5 text-primary" />
            Recent Network Events
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border text-left">
                  <th className="pb-2 font-medium text-muted-foreground">Source</th>
                  <th className="pb-2 font-medium text-muted-foreground">Destination</th>
                  <th className="pb-2 font-medium text-muted-foreground">Proto/Port</th>
                  <th className="pb-2 font-medium text-muted-foreground">Action</th>
                  <th className="pb-2 font-medium text-muted-foreground">Threat</th>
                  <th className="pb-2 font-medium text-muted-foreground">Time</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {mockEvents.map((event) => (
                  <tr key={event.id} className="hover:bg-accent/50">
                    <td className="py-2 font-mono text-xs">{event.sourceIp}</td>
                    <td className="py-2 font-mono text-xs">{event.destinationIp}</td>
                    <td className="py-2 text-xs">{event.protocol}/{event.port}</td>
                    <td className={`py-2 text-xs font-medium capitalize ${actionColors[event.action]}`}>{event.action}</td>
                    <td className="py-2">{event.threat ? <Badge variant={event.threat as "critical" | "high" | "medium"}>{event.threat}</Badge> : <span className="text-xs text-muted-foreground">—</span>}</td>
                    <td className="py-2 text-xs text-muted-foreground">{new Date(event.timestamp).toLocaleTimeString()}</td>
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
