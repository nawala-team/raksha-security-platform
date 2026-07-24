import type { ApiResponse, AuthToken, LoginCredentials } from "@/types";

const API_BASE_URL = process.env.API_BASE_URL || "http://localhost:3001/api";

class ApiClient {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  setToken(token: string) {
    this.token = token;
  }

  clearToken() {
    this.token = null;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
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
        message: "An unexpected error occurred",
      }));
      throw new Error(error.message || `HTTP ${response.status}`);
    }

    return response.json();
  }

  async get<T>(endpoint: string): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: "GET" });
  }

  async post<T>(endpoint: string, data?: unknown): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: "POST",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async put<T>(endpoint: string, data?: unknown): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: "PUT",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async delete<T>(endpoint: string): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: "DELETE" });
  }
}

export const apiClient = new ApiClient(API_BASE_URL);

// API endpoint helpers
export const api = {
  auth: {
    login: (credentials: LoginCredentials) =>
      apiClient.post<AuthToken>("/auth/login", credentials),
    logout: () => apiClient.post("/auth/logout"),
    refresh: (refreshToken: string) =>
      apiClient.post<AuthToken>("/auth/refresh", { refreshToken }),
    verifyMfa: (code: string) =>
      apiClient.post<AuthToken>("/auth/mfa/verify", { code }),
  },
  dashboard: {
    stats: () => apiClient.get("/dashboard/stats"),
    securityScore: () => apiClient.get("/dashboard/security-score"),
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
