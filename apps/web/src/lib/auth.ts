import type { User, AuthToken } from "@/types";

const TOKEN_KEY = "raksha_auth_token";
const REFRESH_KEY = "raksha_refresh_token";

export function getStoredToken(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(TOKEN_KEY);
}

export function setStoredToken(token: AuthToken): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(TOKEN_KEY, token.accessToken);
  localStorage.setItem(REFRESH_KEY, token.refreshToken);
}

export function clearStoredToken(): void {
  if (typeof window === "undefined") return;
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(REFRESH_KEY);
}

export function isAuthenticated(): boolean {
  return !!getStoredToken();
}

export function parseJwt(token: string): Record<string, unknown> | null {
  try {
    const base64Url = token.split(".")[1];
    const base64 = base64Url.replace(/-/g, "+").replace(/_/g, "/");
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split("")
        .map((c) => "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2))
        .join("")
    );
    return JSON.parse(jsonPayload);
  } catch {
    return null;
  }
}

export function getCurrentUser(): User | null {
  const token = getStoredToken();
  if (!token) return null;

  const payload = parseJwt(token);
  if (!payload) return null;

  return {
    id: payload.sub as string,
    email: payload.email as string,
    name: payload.name as string,
    role: payload.role as User["role"],
    mfaEnabled: payload.mfaEnabled as boolean,
    createdAt: payload.createdAt as string,
  };
}

export function isTokenExpired(token: string): boolean {
  const payload = parseJwt(token);
  if (!payload || !payload.exp) return true;
  return Date.now() >= (payload.exp as number) * 1000;
}

export function hasRole(requiredRole: User["role"]): boolean {
  const user = getCurrentUser();
  if (!user) return false;

  const roleHierarchy: Record<User["role"], number> = {
    admin: 4,
    analyst: 3,
    operator: 2,
    viewer: 1,
  };

  return roleHierarchy[user.role] >= roleHierarchy[requiredRole];
}
