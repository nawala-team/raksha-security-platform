import type { ApiResponse, AuthToken, AuthResponse, LoginCredentials } from "@/types";

// Keep browser requests same-origin; Next.js proxies /api to the internal portal.
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "/api/v1";

class ApiClient {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;

    // Restore token from localStorage on init
    if (typeof window !== "undefined") {
      const stored = localStorage.getItem("raksha_auth_token");
      if (stored) {
        try {
          const parsed = JSON.parse(stored);
          this.token = parsed.access_token || null;
        } catch {
          localStorage.removeItem("raksha_auth_token");
        }
      }
    }
  }

  setToken(token: string) {
    this.token = token;
  }

  clearToken() {
    this.token = null;
    if (typeof window !== "undefined") {
      localStorage.removeItem("raksha_auth_token");
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
    acknowledge: (id: string) => apiClient.put(`/alerts/${id}/acknowledge`),
    resolve: (id: string) => apiClient.put(`/alerts/${id}/resolve`),
  },
  servers: {
    list: () => apiClient.get("/servers"),
    get: (id: string) => apiClient.get(`/servers/${id}`),
  },
  network: {
    events: (params?: Record<string, string>) => {
      const query = params ? `?${new URLSearchParams(params)}` : "";
      return apiClient.get(`/network/events${query}`);
    },
    rules: () => apiClient.get("/network/rules"),
  },
  databases: {
    list: () => apiClient.get("/databases"),
    get: (id: string) => apiClient.get(`/databases/${id}`),
  },
  compliance: {
    frameworks: () => apiClient.get("/compliance/frameworks"),
    get: (id: string) => apiClient.get(`/compliance/frameworks/${id}`),
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
    delete: (id: string) => apiClient.delete(`/users/${id}`),
  },
  setup: {
    systemCheck: () => apiClient.get("/setup/system-check"),
    configure: (config: unknown) => apiClient.post("/setup/configure", config),
    status: () => apiClient.get("/setup/status"),
  },
};
