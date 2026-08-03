import type { User, AuthToken } from "@/types";

// The token pair is persisted as a single JSON blob so that this module and
// `lib/api.ts` agree on one storage format.
const TOKEN_KEY = "raksha_auth_token";

function storage(): Storage | null {
  if (typeof window === "undefined" || !window.localStorage) return null;
  return window.localStorage;
}

/** Read the stored token pair, or null when absent/corrupt. */
export function getStoredTokens(): AuthToken | null {
  const store = storage();
  if (!store) return null;

  const raw = store.getItem(TOKEN_KEY);
  if (!raw) return null;

  try {
    const parsed = JSON.parse(raw) as AuthToken;
    return parsed.access_token ? parsed : null;
  } catch {
    store.removeItem(TOKEN_KEY);
    return null;
  }
}

/** Return just the access token, which is what Authorization headers need. */
export function getStoredToken(): string | null {
  return getStoredTokens()?.access_token ?? null;
}

export function getStoredRefreshToken(): string | null {
  return getStoredTokens()?.refresh_token ?? null;
}

export function setStoredToken(token: AuthToken): void {
  storage()?.setItem(TOKEN_KEY, JSON.stringify(token));
}

export function clearStoredToken(): void {
  storage()?.removeItem(TOKEN_KEY);
}

export function isAuthenticated(): boolean {
  const token = getStoredToken();
  return !!token && !isTokenExpired(token);
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

  // Mirrors the server-side hierarchy in raksha-core: UserRole::level().
  const roleHierarchy: Record<User["role"], number> = {
    super_admin: 5,
    admin: 4,
    analyst: 3,
    operator: 2,
    viewer: 1,
  };

  const actual = roleHierarchy[user.role];
  const required = roleHierarchy[requiredRole];
  if (actual === undefined || required === undefined) return false;

  return actual >= required;
}
