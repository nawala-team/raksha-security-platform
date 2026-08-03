"use client";

import { useCallback, useEffect, useRef, useState } from "react";

/**
 * Shape of a paginated portal response (`PaginatedResponse<T>` on the Rust side).
 */
export interface PaginatedPayload<T> {
  data: T[];
  meta: {
    page: number;
    per_page: number;
    total: number;
    total_pages: number;
  };
}

export interface UseApiDataResult<T> {
  data: T | null;
  loading: boolean;
  /** Human-readable message when the request failed, otherwise null. */
  error: string | null;
  /** Re-run the fetch, e.g. from a Retry button. */
  refetch: () => void;
}

/**
 * Fetch data from the portal API and track loading/error state.
 *
 * The project has no data-fetching library, so this keeps the plain
 * `useEffect` + `apiClient` convention used elsewhere in one place instead of
 * repeating it on every dashboard page.
 *
 * `fetcher` is intentionally *not* part of the dependency list: inline arrow
 * functions get a new identity on every render, which would loop forever. Pass
 * anything that varies via `deps` instead.
 */
export function useApiData<T>(
  fetcher: () => Promise<unknown>,
  deps: unknown[] = []
): UseApiDataResult<T> {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  // Keep the latest fetcher without making it a re-render trigger.
  const fetcherRef = useRef(fetcher);
  fetcherRef.current = fetcher;

  useEffect(() => {
    // Guards against setting state after the component unmounts, or after the
    // deps change and an older in-flight request resolves late.
    let active = true;

    setLoading(true);
    setError(null);

    fetcherRef
      .current()
      .then((result) => {
        if (!active) return;
        setData(result as T);
        setError(null);
      })
      .catch((err: unknown) => {
        if (!active) return;
        setData(null);
        setError(err instanceof Error ? err.message : "Failed to load data");
      })
      .finally(() => {
        if (active) setLoading(false);
      });

    return () => {
      active = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [reloadKey, ...deps]);

  const refetch = useCallback(() => setReloadKey((k) => k + 1), []);

  return { data, loading, error, refetch };
}

/**
 * Convenience wrapper for endpoints returning `PaginatedResponse<T>`; exposes
 * the rows directly so pages do not have to unwrap `.data` themselves.
 */
export function useApiList<T>(
  fetcher: () => Promise<unknown>,
  deps: unknown[] = []
): UseApiDataResult<PaginatedPayload<T>> & { items: T[]; total: number } {
  const result = useApiData<PaginatedPayload<T>>(fetcher, deps);

  return {
    ...result,
    items: result.data?.data ?? [],
    total: result.data?.meta.total ?? 0,
  };
}
