"use client";

import { useState } from "react";
import {
  Search, Play, Save, Clock, ChevronDown, BookOpen, Calendar,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

interface QueryResult {
  [key: string]: string;
}

interface SavedQuery {
  id: string;
  name: string;
  query: string;
  lastRun: string;
  scheduled: boolean;
}

const exampleQueries = [
  { label: "Failed SSH logins (last 24h)", query: 'event.type = "auth_failure" AND service = "sshd" | timerange 24h' },
  { label: "Outbound connections to rare IPs", query: 'direction = "outbound" AND dest.reputation = "unknown" | count by dest.ip | sort count desc' },
  { label: "Process execution with encoded args", query: 'event.type = "process_start" AND cmdline matches "*base64*" | table host, user, cmdline' },
  { label: "Lateral movement indicators", query: 'event.type = "network" AND src.zone = "internal" AND dest.zone = "internal" AND dest.port in (445, 3389, 5985)' },
];

const mockResults: QueryResult[] = [
  { timestamp: "2024-01-15 10:32:14", host: "prod-web-01", user: "root", source_ip: "45.33.32.156", event: "auth_failure" },
  { timestamp: "2024-01-15 10:30:02", host: "prod-web-01", user: "admin", source_ip: "45.33.32.156", event: "auth_failure" },
  { timestamp: "2024-01-15 10:28:45", host: "prod-api-01", user: "deploy", source_ip: "198.51.100.23", event: "auth_failure" },
  { timestamp: "2024-01-15 10:25:11", host: "prod-db-01", user: "postgres", source_ip: "203.0.113.50", event: "auth_failure" },
  { timestamp: "2024-01-15 10:22:33", host: "prod-web-02", user: "www-data", source_ip: "45.33.32.156", event: "auth_failure" },
];

const mockSavedQueries: SavedQuery[] = [
  { id: "1", name: "Brute Force Detection", query: 'event.type = "auth_failure" | count by source_ip | where count > 5', lastRun: "2 min ago", scheduled: true },
  { id: "2", name: "Suspicious Processes", query: 'event.type = "process_start" AND user = "root" AND parent != "systemd"', lastRun: "1 hour ago", scheduled: false },
  { id: "3", name: "Data Exfil Candidates", query: 'direction = "outbound" AND bytes_out > 100MB | timerange 1h', lastRun: "30 min ago", scheduled: true },
];

export default function HuntingPage() {
  const [query, setQuery] = useState('event.type = "auth_failure" AND service = "sshd" | timerange 24h');
  const [results, setResults] = useState<QueryResult[]>([]);
  const [running, setRunning] = useState(false);
  const [savedQueries] = useState(mockSavedQueries);

  const handleRun = () => {
    setRunning(true);
    setTimeout(() => {
      setResults(mockResults);
      setRunning(false);
    }, 1500);
  };

  const handleSelectExample = (value: string) => {
    const example = exampleQueries.find((e) => e.label === value);
    if (example) setQuery(example.query);
  };

  const columns = results.length > 0 ? Object.keys(results[0]) : [];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground">Threat Hunting</h2>
        <p className="text-muted-foreground">Query security data with Raksha Query Language (RQL)</p>
      </div>

      {/* Query Editor */}
      <Card className="border-border">
        <CardContent className="p-4 space-y-4">
          <div className="flex items-center gap-2">
            <Select onValueChange={handleSelectExample}>
              <SelectTrigger className="w-64"><SelectValue placeholder="Example queries..." /></SelectTrigger>
              <SelectContent>
                {exampleQueries.map((eq) => (
                  <SelectItem key={eq.label} value={eq.label}>{eq.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <textarea
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="w-full h-32 rounded-md border border-border bg-muted/30 p-3 font-mono text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring resize-y"
            placeholder="Enter RQL query..."
            spellCheck={false}
            aria-label="Query editor"
          />
          <div className="flex items-center gap-2">
            <Button onClick={handleRun} disabled={running || !query.trim()} className="gap-2">
              <Play className="h-4 w-4" />{running ? "Running..." : "Run Query"}
            </Button>
            <Button variant="outline" className="gap-2"><Save className="h-4 w-4" /> Save Query</Button>
            <div className="ml-auto flex items-center gap-2">
              <span className="text-xs text-muted-foreground">Schedule</span>
              <button
                className="relative h-5 w-9 rounded-full bg-muted transition-colors focus:outline-none focus:ring-2 focus:ring-ring"
                role="switch"
                aria-checked="false"
                aria-label="Toggle schedule"
              >
                <span className="absolute left-0.5 top-0.5 h-4 w-4 rounded-full bg-muted-foreground transition-transform" />
              </button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Results */}
      {results.length > 0 && (
        <Card className="border-border">
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-lg">{results.length} Results</CardTitle>
              <span className="text-xs text-muted-foreground">Query completed in 0.34s</span>
            </div>
          </CardHeader>
          <CardContent className="p-0">
            <div className="overflow-x-auto">
              <table className="w-full text-sm" role="table">
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    {columns.map((col) => (
                      <th key={col} className="px-4 py-3 text-left font-medium text-muted-foreground">{col}</th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {results.map((row, i) => (
                    <tr key={i} className="border-b border-border hover:bg-muted/20">
                      {columns.map((col) => (
                        <td key={col} className="px-4 py-3 font-mono text-xs text-foreground">{row[col]}</td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Saved Queries */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <BookOpen className="h-5 w-5 text-blue-400" /> Saved Queries
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          {savedQueries.map((sq) => (
            <div key={sq.id} className="flex items-center justify-between rounded-lg border border-border px-4 py-3">
              <div className="space-y-1">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-foreground">{sq.name}</span>
                  {sq.scheduled && <Badge variant="secondary" className="text-xs"><Calendar className="mr-1 h-3 w-3" />Scheduled</Badge>}
                </div>
                <p className="font-mono text-xs text-muted-foreground truncate max-w-lg">{sq.query}</p>
              </div>
              <div className="flex items-center gap-3">
                <span className="text-xs text-muted-foreground">Last: {sq.lastRun}</span>
                <Button variant="outline" size="sm" onClick={() => { setQuery(sq.query); handleRun(); }}>Run</Button>
              </div>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}
