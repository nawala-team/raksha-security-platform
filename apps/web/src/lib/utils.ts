import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * Format a byte count into a human-readable size (e.g. "2.4 TB").
 *
 * Uses binary units, which is what the agents report. Accepts null/undefined so
 * pages can pass optional API fields straight through.
 */
export function formatBytes(bytes: number | null | undefined, decimals = 1): string {
  if (bytes === null || bytes === undefined || Number.isNaN(bytes)) return "—";
  if (bytes === 0) return "0 B";

  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  const exponent = Math.min(
    Math.floor(Math.log(Math.abs(bytes)) / Math.log(1024)),
    units.length - 1
  );
  const value = bytes / Math.pow(1024, exponent);

  // Whole bytes read oddly as "512.0 B", so drop the decimals at that scale.
  return `${value.toFixed(exponent === 0 ? 0 : decimals)} ${units[exponent]}`;
}

/** Thousands-separated integer, with an em dash for missing values. */
export function formatNumber(value: number | null | undefined): string {
  if (value === null || value === undefined || Number.isNaN(value)) return "—";
  return value.toLocaleString();
}

/** Compact relative time such as "10s ago"; "never" when the value is absent. */
export function relativeTime(iso: string | null | undefined): string {
  if (!iso) return "never";
  const ms = new Date(iso).getTime();
  if (Number.isNaN(ms)) return "unknown";

  const seconds = Math.floor((Date.now() - ms) / 1000);
  if (seconds < 0) return "just now";
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}
