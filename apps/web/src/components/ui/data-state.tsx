"use client";

import { AlertCircle, Inbox, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";

/**
 * Shared loading / error / empty presentation for data-backed dashboard pages.
 *
 * Each state is announced to assistive tech: the spinner uses role="status"
 * with aria-live so screen readers hear it, and failures use role="alert" so
 * they interrupt rather than being missed.
 */

export function LoadingState({ label = "Loading data" }: { label?: string }) {
  return (
    <div
      className="flex items-center justify-center gap-2 py-12"
      role="status"
      aria-live="polite"
    >
      <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" aria-hidden="true" />
      <span className="text-sm text-muted-foreground">{label}</span>
    </div>
  );
}

export function ErrorState({
  message,
  onRetry,
}: {
  message: string;
  onRetry?: () => void;
}) {
  return (
    <div
      className="flex flex-col items-center justify-center gap-3 rounded-lg border border-destructive/40 bg-destructive/5 py-10 px-4 text-center"
      role="alert"
    >
      <AlertCircle className="h-6 w-6 text-destructive" aria-hidden="true" />
      <div>
        <p className="text-sm font-medium text-foreground">Could not load this data</p>
        <p className="mt-1 text-xs text-muted-foreground">{message}</p>
      </div>
      {onRetry && (
        <Button variant="outline" size="sm" onClick={onRetry}>
          Try again
        </Button>
      )}
    </div>
  );
}

export function EmptyState({
  title = "Nothing to show yet",
  description,
}: {
  title?: string;
  description?: string;
}) {
  return (
    <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-border border-dashed py-10 px-4 text-center">
      <Inbox className="h-6 w-6 text-muted-foreground" aria-hidden="true" />
      <p className="text-sm font-medium text-foreground">{title}</p>
      {description && (
        <p className="text-xs text-muted-foreground">{description}</p>
      )}
    </div>
  );
}

/**
 * Renders the right state for a fetch result, or `children` once data is ready.
 *
 * `isEmpty` is passed in by the caller because "empty" depends on the payload
 * shape (an array vs. a summary object).
 */
export function DataState({
  loading,
  error,
  isEmpty = false,
  onRetry,
  loadingLabel,
  emptyTitle,
  emptyDescription,
  children,
}: {
  loading: boolean;
  error: string | null;
  isEmpty?: boolean;
  onRetry?: () => void;
  loadingLabel?: string;
  emptyTitle?: string;
  emptyDescription?: string;
  children: React.ReactNode;
}) {
  if (loading) return <LoadingState label={loadingLabel} />;
  if (error) return <ErrorState message={error} onRetry={onRetry} />;
  if (isEmpty) return <EmptyState title={emptyTitle} description={emptyDescription} />;
  return <>{children}</>;
}
