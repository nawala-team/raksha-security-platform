import type { ApiResponse, AuthToken, AuthResponse, LoginCredentials } from "@/types";

// Keep browser requests same-origin; Next.js proxies /api to the internal portal.
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "/api/v1";

class ApiClient {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;

    // Restore token from localStorage on init.
    // Access it via `window` so the guard above actually covers the call:
    // in non-browser environments (SSR, jsdom before setup) the bare
    // `localStorage` global may be undefined even when `window` exists.
    if (typeof window !== "undefined" && window.localStorage) {
      const stored = window.localStorage.getItem("raksha_auth_token");
      if (stored) {
        try {
          const parsed = JSON.parse(stored);
          this.token = parsed.access_token || null;
        } catch {
          window.localStorage.removeItem("raksha_auth_token");
        }
      }
    }
  }

  setToken(token: string) {
    this.token = token;
  }

  clearToken() {
    this.token = null;
    if (typeof window !== "undefined" && window.localStorage) {
      window.localStorage.removeItem("raksha_auth_token");
    }
  }

  getToken(): string | null {
    return this.token;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const headers: HeadersInit = {
      "Content-Type": "application/json",
      ...options.headers,
    };

    if (this.token) {
      (headers as Record<string, string>)["Authorization"] = `Bearer ${this.token}`;
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({
        error: { message: "An unexpected error occurred" },
      }));
      throw new Error(error.error?.message || `HTTP ${response.status}`);
    }

    return response.json();
  }

  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: "GET" });
  }

  async post<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "POST",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async put<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "PUT",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async patch<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "PATCH",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: "DELETE" });
  }
}

export const apiClient = new ApiClient(API_BASE_URL);

// API endpoint helpers
export const api = {
  auth: {
    login: (credentials: LoginCredentials) =>
      apiClient.post<AuthResponse>("/auth/login", credentials),
    register: (data: { email: string; password: string; name: string }) =>
      apiClient.post<AuthResponse>("/auth/register", data),
    logout: () => apiClient.post<{ message: string }>("/auth/logout"),
    refresh: (refreshToken: string) =>
      apiClient.post<AuthToken>("/auth/refresh", { refresh_token: refreshToken }),
  },
  dashboard: {
    stats: () => apiClient.get("/dashboard/stats"),
    securityScore: () => apiClient.get("/dashboard/security-score"),
  },
  agents: {
    list: () => apiClient.get("/agents"),
    get: (id: string) => apiClient.get(`/agents/${id}`),
    metrics: (id: string) => apiClient.get(`/agents/${id}/metrics`),
    generateToken: (data?: {
      agent_name?: string;
      labels?: string[];
      expiry_hours?: number;
      max_uses?: number;
      allowed_modules?: string[];
    }) => apiClient.post("/agents/tokens", data || {}),
    listTokens: () => apiClient.get("/agents/tokens"),
    revokeToken: (tokenId: string) => apiClient.delete(`/agents/tokens/${tokenId}`),
    enroll: (data: { token: string; fingerprint: Record<string, unknown> }) =>
      apiClient.post("/agents/enroll", data),
    rotateCertificate: (agentId: string) =>
      apiClient.post(`/agents/${agentId}/rotate-certificate`),
  },
  alerts: {
    list: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/alerts${query}`);
    },
    get: (id: string) => apiClient.get(`/alerts/${id}`),
    create: (data: unknown) => apiClient.post("/alerts", data),
    acknowledge: (id: string) =>
      apiClient.patch(`/alerts/${id}/status`, { status: "acknowledged" }),
    resolve: (id: string) =>
      apiClient.patch(`/alerts/${id}/status`, { status: "resolved" }),
  },
  tenants: {
    list: () => apiClient.get("/tenants"),
    get: (id: string) => apiClient.get(`/tenants/${id}`),
    create: (data: unknown) => apiClient.post("/tenants", data),
    update: (id: string, data: unknown) => apiClient.put(`/tenants/${id}`, data),
    suspend: (id: string) => apiClient.post(`/tenants/${id}/suspend`),
    stats: (id: string) => apiClient.get(`/tenants/${id}/stats`),
  },
  servers: {
    list: () => apiClient.get("/servers"),
    get: (id: string) => apiClient.get(`/servers/${id}`),
    summary: () => apiClient.get("/servers/summary"),
  },
  network: {
    events: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/network/events${query}`);
    },
    rules: () => apiClient.get("/network/rules"),
    summary: () => apiClient.get("/network/summary"),
    topTalkers: () => apiClient.get("/network/top-talkers"),
  },
  containers: {
    list: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/containers${query}`);
    },
    get: (id: string) => apiClient.get(`/containers/${id}`),
    summary: () => apiClient.get("/containers/summary"),
    scans: () => apiClient.get("/containers/scans"),
  },
  honeypots: {
    list: () => apiClient.get("/honeypots"),
    summary: () => apiClient.get("/honeypots/summary"),
    interactions: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/honeypots/interactions${query}`);
    },
    topAttackers: () => apiClient.get("/honeypots/top-attackers"),
  },
  darkweb: {
    monitors: () => apiClient.get("/darkweb/monitors"),
    findings: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/darkweb/findings${query}`);
    },
    finding: (id: string) => apiClient.get(`/darkweb/findings/${id}`),
    summary: () => apiClient.get("/darkweb/summary"),
  },
  hunting: {
    queries: () => apiClient.get("/hunting/queries"),
    query: (id: string) => apiClient.get(`/hunting/queries/${id}`),
    queryRuns: (id: string) => apiClient.get(`/hunting/queries/${id}/runs`),
    runs: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/hunting/runs${query}`);
    },
    validate: (rql: string) => apiClient.post("/hunting/validate", { rql }),
  },
  backups: {
    jobs: () => apiClient.get("/backups/jobs"),
    job: (id: string) => apiClient.get(`/backups/jobs/${id}`),
    jobRuns: (id: string) => apiClient.get(`/backups/jobs/${id}/runs`),
    runs: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/backups/runs${query}`);
    },
    summary: () => apiClient.get("/backups/summary"),
  },
  documents: {
    list: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/documents${query}`);
    },
    get: (id: string) => apiClient.get(`/documents/${id}`),
    summary: () => apiClient.get("/documents/summary"),
    expiring: () => apiClient.get("/documents/expiring"),
  },
  incidents: {
    list: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/incidents${query}`);
    },
    get: (id: string) => apiClient.get(`/incidents/${id}`),
    timeline: (id: string) => apiClient.get(`/incidents/${id}/timeline`),
    tasks: (id: string) => apiClient.get(`/incidents/${id}/tasks`),
    summary: () => apiClient.get("/incidents/summary"),
  },
  grc: {
    risks: () => apiClient.get("/grc/risks"),
    risk: (id: string) => apiClient.get(`/grc/risks/${id}`),
    policies: () => apiClient.get("/grc/policies"),
    controls: () => apiClient.get("/grc/controls"),
    summary: () => apiClient.get("/grc/summary"),
  },
  vulnerabilities: {
    scans: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/vulnerabilities/scans${query}`);
    },
    scan: (id: string) => apiClient.get(`/vulnerabilities/scans/${id}`),
    summary: () => apiClient.get("/vulnerabilities/summary"),
  },
  fim: {
    events: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/fim/events${query}`);
    },
    event: (id: string) => apiClient.get(`/fim/events/${id}`),
    summary: () => apiClient.get("/fim/summary"),
    topPaths: () => apiClient.get("/fim/top-paths"),
  },
  settings: {
    channels: () => apiClient.get("/settings/channels"),
    channel: (id: string) => apiClient.get(`/settings/channels/${id}`),
    rules: () => apiClient.get("/settings/rules"),
    templates: () => apiClient.get("/settings/templates"),
    summary: () => apiClient.get("/settings/summary"),
  },
  databases: {
    list: () => apiClient.get("/databases"),
    get: (id: string) => apiClient.get(`/databases/${id}`),
    register: (data: unknown) => apiClient.post("/databases", data),
    unregister: (id: string) => apiClient.delete(`/databases/${id}`),
    metrics: (id: string) => apiClient.get(`/databases/${id}/metrics`),
  },
  compliance: {
    scores: () => apiClient.get("/compliance/scores"),
    score: (id: string) => apiClient.get(`/compliance/scores/${id}`),
    standards: () => apiClient.get("/compliance/standards"),
    controls: () => apiClient.get("/compliance/controls"),
  },
  audit: {
    list: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/audit${query}`);
    },
  },
  users: {
    list: () => apiClient.get("/users"),
    get: (id: string) => apiClient.get(`/users/${id}`),
    create: (data: unknown) => apiClient.post("/users", data),
    update: (id: string, data: unknown) => apiClient.put(`/users/${id}`, data),
    updateRole: (id: string, role: string) =>
      apiClient.put(`/users/${id}/role`, { role }),
    delete: (id: string) => apiClient.delete(`/users/${id}`),
  },
  setup: {
    systemCheck: () => apiClient.get("/setup/system-check"),
    configure: (config: unknown) => apiClient.post("/setup/configure", config),
    status: () => apiClient.get("/setup/status"),
  },
  threatIntel: {
    feeds: () => apiClient.get("/threat-intel/feeds"),
    syncFeeds: () => apiClient.post("/threat-intel/feeds/sync"),
    iocs: () => apiClient.get("/threat-intel/iocs"),
    addIoc: (data: unknown) => apiClient.post("/threat-intel/iocs", data),
    searchIocs: (q: string) => apiClient.post("/threat-intel/iocs/search", { q }),
  },
  attackSurface: {
    list: () => apiClient.get("/attack-surface"),
    summary: () => apiClient.get("/attack-surface/summary"),
    add: (data: unknown) => apiClient.post("/attack-surface", data),
    remove: (id: string) => apiClient.delete(`/attack-surface/${id}`),
  },
};
