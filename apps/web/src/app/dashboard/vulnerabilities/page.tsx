"use client";

import { useState, useMemo } from "react";
import {
  Bug, Search, Filter, ExternalLink, CheckCircle2, XCircle,
  AlertTriangle, ShieldAlert, ShieldCheck, ChevronDown, ChevronRight,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";

type Severity = "critical" | "high" | "medium" | "low";
type VulnStatus = "open" | "patched" | "accepted";

interface Vulnerability {
  id: string;
  cveId: string;
  pkg: string;
  version: string;
  cvssScore: number;
  severity: Severity;
  exploitable: boolean;
  status: VulnStatus;
  server: string;
  discoveredAt: string;
  description: string;
  remediation: string;
  references: string[];
}

const MOCK_DATA: Vulnerability[] = [
  { id: "1", cveId: "CVE-2024-3094", pkg: "xz-utils", version: "5.6.0", cvssScore: 10.0, severity: "critical", exploitable: true, status: "open", server: "prod-web-01", discoveredAt: "2024-03-29", description: "Malicious code in upstream tarballs of xz starting with 5.6.0.", remediation: "Downgrade xz-utils to version 5.4.x immediately.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2024-3094"] },
  { id: "2", cveId: "CVE-2024-21626", pkg: "runc", version: "1.1.11", cvssScore: 8.6, severity: "high", exploitable: true, status: "open", server: "prod-docker-02", discoveredAt: "2024-01-31", description: "Container breakout via leaked file descriptors in runc < 1.1.12.", remediation: "Update runc to 1.1.12+. Update Docker Engine to 25.0.1+.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2024-21626"] },
  { id: "3", cveId: "CVE-2023-44487", pkg: "nginx", version: "1.24.0", cvssScore: 7.5, severity: "high", exploitable: true, status: "patched", server: "prod-lb-01", discoveredAt: "2023-10-10", description: "HTTP/2 Rapid Reset Attack allows DoS via rapid stream resets.", remediation: "Update nginx to 1.25.3+ or apply vendor patches.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2023-44487"] },
  { id: "4", cveId: "CVE-2024-0567", pkg: "gnutls", version: "3.8.2", cvssScore: 7.5, severity: "high", exploitable: false, status: "open", server: "prod-api-01", discoveredAt: "2024-01-16", description: "GnuTLS rejects certificate chain with distributed trust.", remediation: "Update gnutls to version 3.8.3 or later.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2024-0567"] },
  { id: "5", cveId: "CVE-2023-5363", pkg: "openssl", version: "3.1.3", cvssScore: 5.3, severity: "medium", exploitable: false, status: "accepted", server: "staging-web-01", discoveredAt: "2023-10-25", description: "OpenSSL incorrectly handles key and IV lengths for symmetric ciphers.", remediation: "Update OpenSSL to 3.1.4 or 3.0.12.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2023-5363"] },
  { id: "6", cveId: "CVE-2024-22365", pkg: "pam", version: "1.5.2", cvssScore: 5.5, severity: "medium", exploitable: false, status: "open", server: "prod-web-01", discoveredAt: "2024-01-17", description: "linux-pam before 1.6.0 allows DoS via login name with special character.", remediation: "Update linux-pam to version 1.6.0 or later.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2024-22365"] },
  { id: "7", cveId: "CVE-2023-6246", pkg: "glibc", version: "2.37", cvssScore: 8.4, severity: "high", exploitable: true, status: "open", server: "prod-api-01", discoveredAt: "2024-01-30", description: "Heap-based buffer overflow in __vsyslog_internal allows privilege escalation.", remediation: "Update glibc to patched version from your distribution.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2023-6246"] },
  { id: "8", cveId: "CVE-2023-4911", pkg: "glibc", version: "2.34", cvssScore: 7.8, severity: "high", exploitable: true, status: "patched", server: "prod-db-01", discoveredAt: "2023-10-03", description: "Buffer overflow in ld.so (Looney Tunables) allows privilege escalation.", remediation: "Update glibc. All major distributions have released patches.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2023-4911"] },
  { id: "9", cveId: "CVE-2024-1086", pkg: "linux-kernel", version: "6.6.14", cvssScore: 7.8, severity: "high", exploitable: true, status: "open", server: "prod-docker-02", discoveredAt: "2024-02-01", description: "Use-after-free in netfilter nf_tables allows privilege escalation.", remediation: "Update Linux kernel to 6.6.15+ or 6.7.3+.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2024-1086"] },
  { id: "10", cveId: "CVE-2023-50164", pkg: "struts", version: "6.3.0", cvssScore: 9.8, severity: "critical", exploitable: true, status: "open", server: "prod-app-01", discoveredAt: "2023-12-07", description: "Apache Struts file upload path traversal enables RCE.", remediation: "Upgrade Apache Struts to 6.3.0.2 or 2.5.33.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2023-50164"] },
  { id: "11", cveId: "CVE-2023-48795", pkg: "openssh", version: "9.5", cvssScore: 5.9, severity: "medium", exploitable: false, status: "patched", server: "prod-web-01", discoveredAt: "2023-12-18", description: "Terrapin attack: SSH protocol prefix truncation weakness.", remediation: "Update OpenSSH to 9.6+.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2023-48795"] },
  { id: "12", cveId: "CVE-2023-46604", pkg: "activemq", version: "5.17.5", cvssScore: 3.7, severity: "low", exploitable: false, status: "accepted", server: "staging-mq-01", discoveredAt: "2023-10-27", description: "Info disclosure in ActiveMQ management console.", remediation: "Upgrade to ActiveMQ 5.17.6+.", references: ["https://nvd.nist.gov/vuln/detail/CVE-2023-46604"] },
];

function getSeverityVariant(severity: Severity) {
  return severity as "critical" | "high" | "medium" | "low";
}

function getStatusIcon(status: VulnStatus) {
  switch (status) {
    case "open": return <XCircle className="h-4 w-4 text-red-400" />;
    case "patched": return <CheckCircle2 className="h-4 w-4 text-green-400" />;
    case "accepted": return <AlertTriangle className="h-4 w-4 text-yellow-400" />;
  }
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString("en-US", {
    year: "numeric", month: "short", day: "numeric",
  });
}

export default function VulnerabilitiesPage() {
  const [severityFilter, setSeverityFilter] = useState<string>("all");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [serverFilter, setServerFilter] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedRow, setExpandedRow] = useState<string | null>(null);

  const servers = useMemo(() => [...new Set(MOCK_DATA.map((v) => v.server))], []);

  const filtered = useMemo(() => {
    return MOCK_DATA.filter((v) => {
      if (severityFilter !== "all" && v.severity !== severityFilter) return false;
      if (statusFilter !== "all" && v.status !== statusFilter) return false;
      if (serverFilter !== "all" && v.server !== serverFilter) return false;
      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        if (!v.pkg.toLowerCase().includes(q) && !v.cveId.toLowerCase().includes(q)) return false;
      }
      return true;
    });
  }, [severityFilter, statusFilter, serverFilter, searchQuery]);

  const stats = useMemo(() => ({
    total: MOCK_DATA.length,
    critical: MOCK_DATA.filter((v) => v.severity === "critical").length,
    high: MOCK_DATA.filter((v) => v.severity === "high").length,
    medium: MOCK_DATA.filter((v) => v.severity === "medium").length,
    low: MOCK_DATA.filter((v) => v.severity === "low").length,
    patched: MOCK_DATA.filter((v) => v.status === "patched").length,
  }), []);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-foreground flex items-center gap-2">
          <Bug className="h-6 w-6 text-primary" />
          Vulnerability Dashboard
        </h2>
        <p className="text-muted-foreground">CVE tracking and vulnerability management across all servers</p>
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6">
        <Card className="border-border"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-foreground">{stats.total}</p><p className="text-xs text-muted-foreground">Total CVEs</p></CardContent></Card>
        <Card className="border-red-500/30 bg-red-500/5"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-red-400">{stats.critical}</p><p className="text-xs text-red-400/80">Critical</p></CardContent></Card>
        <Card className="border-orange-500/30 bg-orange-500/5"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-orange-400">{stats.high}</p><p className="text-xs text-orange-400/80">High</p></CardContent></Card>
        <Card className="border-yellow-500/30 bg-yellow-500/5"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-yellow-400">{stats.medium}</p><p className="text-xs text-yellow-400/80">Medium</p></CardContent></Card>
        <Card className="border-green-500/30 bg-green-500/5"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-green-400">{stats.low}</p><p className="text-xs text-green-400/80">Low</p></CardContent></Card>
        <Card className="border-blue-500/30 bg-blue-500/5"><CardContent className="p-4 text-center"><p className="text-2xl font-bold text-blue-400">{stats.patched}</p><p className="text-xs text-blue-400/80">Patched</p></CardContent></Card>
      </div>

      {/* Severity Breakdown Bar */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium text-muted-foreground">Severity Breakdown</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex h-4 w-full overflow-hidden rounded-full">
            {stats.critical > 0 && <div className="bg-red-500" style={{ width: `${(stats.critical / stats.total) * 100}%` }} title={`Critical: ${stats.critical}`} role="img" aria-label={`Critical: ${stats.critical}`} />}
            {stats.high > 0 && <div className="bg-orange-500" style={{ width: `${(stats.high / stats.total) * 100}%` }} title={`High: ${stats.high}`} role="img" aria-label={`High: ${stats.high}`} />}
            {stats.medium > 0 && <div className="bg-yellow-500" style={{ width: `${(stats.medium / stats.total) * 100}%` }} title={`Medium: ${stats.medium}`} role="img" aria-label={`Medium: ${stats.medium}`} />}
            {stats.low > 0 && <div className="bg-green-500" style={{ width: `${(stats.low / stats.total) * 100}%` }} title={`Low: ${stats.low}`} role="img" aria-label={`Low: ${stats.low}`} />}
          </div>
          <div className="mt-2 flex flex-wrap gap-4 text-xs text-muted-foreground">
            <span className="flex items-center gap-1"><span className="h-2.5 w-2.5 rounded-full bg-red-500" />Critical ({stats.critical})</span>
            <span className="flex items-center gap-1"><span className="h-2.5 w-2.5 rounded-full bg-orange-500" />High ({stats.high})</span>
            <span className="flex items-center gap-1"><span className="h-2.5 w-2.5 rounded-full bg-yellow-500" />Medium ({stats.medium})</span>
            <span className="flex items-center gap-1"><span className="h-2.5 w-2.5 rounded-full bg-green-500" />Low ({stats.low})</span>
          </div>
        </CardContent>
      </Card>

      {/* Filters */}
      <Card className="border-border">
        <CardContent className="p-4">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex items-center gap-2">
              <Filter className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm font-medium text-muted-foreground">Filters:</span>
            </div>
            <div className="relative">
              <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input placeholder="Search CVE or package..." value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} className="h-9 w-56 pl-8" aria-label="Search by CVE ID or package name" />
            </div>
            <Select value={severityFilter} onValueChange={setSeverityFilter}>
              <SelectTrigger className="h-9 w-36" aria-label="Filter by severity"><SelectValue placeholder="Severity" /></SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Severities</SelectItem>
                <SelectItem value="critical">Critical</SelectItem>
                <SelectItem value="high">High</SelectItem>
                <SelectItem value="medium">Medium</SelectItem>
                <SelectItem value="low">Low</SelectItem>
              </SelectContent>
            </Select>
            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger className="h-9 w-32" aria-label="Filter by status"><SelectValue placeholder="Status" /></SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Status</SelectItem>
                <SelectItem value="open">Open</SelectItem>
                <SelectItem value="patched">Patched</SelectItem>
                <SelectItem value="accepted">Accepted</SelectItem>
              </SelectContent>
            </Select>
            <Select value={serverFilter} onValueChange={setServerFilter}>
              <SelectTrigger className="h-9 w-40" aria-label="Filter by server"><SelectValue placeholder="Server" /></SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Servers</SelectItem>
                {servers.map((s) => (<SelectItem key={s} value={s}>{s}</SelectItem>))}
              </SelectContent>
            </Select>
            {(severityFilter !== "all" || statusFilter !== "all" || serverFilter !== "all" || searchQuery) && (
              <Button variant="ghost" size="sm" onClick={() => { setSeverityFilter("all"); setStatusFilter("all"); setServerFilter("all"); setSearchQuery(""); }}>Clear filters</Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Vulnerability Table */}
      <Card className="border-border">
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full text-sm" role="table">
              <thead>
                <tr className="border-b border-border bg-muted/30">
                  <th className="w-8 px-4 py-3" />
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">CVE ID</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Package</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Version</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">CVSS</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Severity</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Exploitable</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Server</th>
                  <th className="px-4 py-3 text-left font-medium text-muted-foreground">Discovered</th>
                </tr>
              </thead>
              <tbody>
                {filtered.length === 0 && (
                  <tr><td colSpan={10} className="px-4 py-12 text-center text-muted-foreground">No vulnerabilities match the current filters.</td></tr>
                )}
                {filtered.map((vuln) => (
                  <VulnRow key={vuln.id} vuln={vuln} expanded={expandedRow === vuln.id} onToggle={() => setExpandedRow(expandedRow === vuln.id ? null : vuln.id)} />
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      <p className="text-xs text-muted-foreground">Showing {filtered.length} of {MOCK_DATA.length} vulnerabilities</p>
    </div>
  );
}

function VulnRow({ vuln, expanded, onToggle }: { vuln: Vulnerability; expanded: boolean; onToggle: () => void }) {
  return (
    <>
      <tr className="border-b border-border transition-colors hover:bg-muted/20 cursor-pointer" onClick={onToggle} aria-expanded={expanded} role="row">
        <td className="px-4 py-3">
          <button className="text-muted-foreground hover:text-foreground" aria-label={expanded ? "Collapse details" : "Expand details"} onClick={(e) => { e.stopPropagation(); onToggle(); }}>
            {expanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
          </button>
        </td>
        <td className="px-4 py-3 font-mono text-xs text-foreground">{vuln.cveId}</td>
        <td className="px-4 py-3 font-medium text-foreground">{vuln.pkg}</td>
        <td className="px-4 py-3 font-mono text-xs text-muted-foreground">{vuln.version}</td>
        <td className="px-4 py-3">
          <span className={vuln.cvssScore >= 9 ? "font-bold text-red-400" : vuln.cvssScore >= 7 ? "font-semibold text-orange-400" : vuln.cvssScore >= 4 ? "text-yellow-400" : "text-green-400"}>
            {vuln.cvssScore.toFixed(1)}
          </span>
        </td>
        <td className="px-4 py-3"><Badge variant={getSeverityVariant(vuln.severity)}>{vuln.severity}</Badge></td>
        <td className="px-4 py-3">
          {vuln.exploitable ? (
            <span className="flex items-center gap-1 text-red-400"><ShieldAlert className="h-3.5 w-3.5" />Yes</span>
          ) : (
            <span className="flex items-center gap-1 text-muted-foreground"><ShieldCheck className="h-3.5 w-3.5" />No</span>
          )}
        </td>
        <td className="px-4 py-3"><span className="flex items-center gap-1.5 capitalize">{getStatusIcon(vuln.status)}{vuln.status}</span></td>
        <td className="px-4 py-3 text-muted-foreground">{vuln.server}</td>
        <td className="px-4 py-3 text-muted-foreground">{formatDate(vuln.discoveredAt)}</td>
      </tr>
      {expanded && (
        <tr className="border-b border-border bg-muted/10">
          <td colSpan={10} className="px-8 py-4">
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <h4 className="text-sm font-semibold text-foreground">Description</h4>
                <p className="text-sm text-muted-foreground leading-relaxed">{vuln.description}</p>
              </div>
              <div className="space-y-4">
                <div className="space-y-2">
                  <h4 className="text-sm font-semibold text-foreground">Remediation</h4>
                  <p className="text-sm text-muted-foreground leading-relaxed">{vuln.remediation}</p>
                </div>
                <div className="space-y-2">
                  <h4 className="text-sm font-semibold text-foreground">References</h4>
                  <ul className="space-y-1">
                    {vuln.references.map((ref, i) => (
                      <li key={i}>
                        <a href={ref} target="_blank" rel="noopener noreferrer" className="inline-flex items-center gap-1 text-xs text-primary hover:underline">
                          <ExternalLink className="h-3 w-3" />{ref}
                        </a>
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
            </div>
          </td>
        </tr>
      )}
    </>
  );
}
