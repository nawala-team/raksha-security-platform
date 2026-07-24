// TypeScript type definitions for Raksha Security Platform

export type ThreatLevel = "critical" | "high" | "medium" | "low" | "info";

export type AlertStatus = "active" | "acknowledged" | "resolved" | "false_positive";

export type ServerStatus = "online" | "offline" | "degraded" | "maintenance";

export type ComplianceStatus = "compliant" | "non_compliant" | "partial" | "pending";

export interface User {
  id: string;
  email: string;
  name: string;
  role: UserRole;
  avatar?: string;
  lastLogin?: string;
  mfaEnabled: boolean;
  createdAt: string;
}

export type UserRole = "admin" | "analyst" | "operator" | "viewer";

export interface Alert {
  id: string;
  title: string;
  description: string;
  severity: ThreatLevel;
  status: AlertStatus;
  source: string;
  timestamp: string;
  assignee?: string;
  tags: string[];
}

export interface Server {
  id: string;
  hostname: string;
  ipAddress: string;
  status: ServerStatus;
  cpuUsage: number;
  memoryUsage: number;
  diskUsage: number;
  os: string;
  lastHeartbeat: string;
  alerts: number;
}

export interface NetworkEvent {
  id: string;
  sourceIp: string;
  destinationIp: string;
  protocol: string;
  port: number;
  action: "allow" | "block" | "monitor";
  timestamp: string;
  bytesTransferred: number;
  threat?: ThreatLevel;
}

export interface DatabaseInstance {
  id: string;
  name: string;
  type: "postgresql" | "mysql" | "mongodb" | "redis";
  status: ServerStatus;
  connections: number;
  maxConnections: number;
  queryRate: number;
  replicationLag?: number;
  size: string;
}

export interface ComplianceFramework {
  id: string;
  name: string;
  status: ComplianceStatus;
  score: number;
  totalControls: number;
  passedControls: number;
  failedControls: number;
  lastAudit: string;
}

export interface AuditEntry {
  id: string;
  action: string;
  actor: string;
  resource: string;
  timestamp: string;
  ipAddress: string;
  details: string;
  result: "success" | "failure";
}

export interface SecurityDocument {
  id: string;
  title: string;
  category: string;
  version: string;
  lastUpdated: string;
  author: string;
  status: "draft" | "published" | "archived";
}

export interface Agent {
  id: string;
  name: string;
  hostname: string;
  os: "linux" | "windows" | "darwin";
  arch: "x86_64" | "aarch64";
  version: string;
  status: "online" | "offline" | "degraded";
  lastSeen: string;
  enrolledAt: string;
  ipAddress?: string;
  networkZone?: string;
  labels: string[];
  modules: string[];
}

export interface EnrollmentToken {
  id: string;
  tokenPreview: string;
  agentName: string;
  labels: string[];
  createdAt: string;
  expiresAt: string;
  maxUses: number;
  usedCount: number;
  status: "active" | "expired" | "revoked" | "used";
  createdBy: string;
}


  overall: number;
  categories: {
    network: number;
    endpoint: number;
    identity: number;
    data: number;
    application: number;
  };
  trend: "improving" | "stable" | "declining";
  lastUpdated: string;
}

export interface SetupConfig {
  systemCheck: {
    completed: boolean;
    results: { check: string; passed: boolean; message: string }[];
  };
  database: {
    host: string;
    port: number;
    name: string;
    username: string;
    password: string;
    type: "postgresql" | "mysql";
  };
  admin: {
    email: string;
    name: string;
    password: string;
  };
  modules: {
    serverMonitoring: boolean;
    networkSecurity: boolean;
    databaseMonitoring: boolean;
    complianceEngine: boolean;
    threatIntelligence: boolean;
    auditTrail: boolean;
  };
  intelligenceFeeds: {
    otx: boolean;
    abuseIpDb: boolean;
    virusTotal: boolean;
    customFeeds: string[];
  };
}

export interface DashboardStats {
  securityScore: SecurityScore;
  activeAlerts: number;
  criticalAlerts: number;
  serversOnline: number;
  serversTotal: number;
  threatsBlocked: number;
  complianceScore: number;
}

export interface ApiResponse<T> {
  data: T;
  success: boolean;
  message?: string;
  pagination?: {
    page: number;
    pageSize: number;
    total: number;
    totalPages: number;
  };
}

export interface LoginCredentials {
  email: string;
  password: string;
  mfaCode?: string;
}

export interface AuthToken {
  accessToken: string;
  refreshToken: string;
  expiresAt: string;
}
