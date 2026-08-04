"use client";

import { useState } from "react";

import {
  FileText, Plus, CheckCircle2, PencilLine, Eye, CalendarClock, X, Trash2,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DataState } from "@/components/ui/data-state";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useApiData, useApiList } from "@/hooks/use-api-data";
import { api } from "@/lib/api";
import { formatBytes, formatNumber, relativeTime } from "@/lib/utils";

/** Mirrors the portal's `DocumentResponse`. */
interface DocumentRecord {
  id: string;
  title: string;
  description: string | null;
  doc_type: string;
  category: string | null;
  status: string;
  classification: string;
  version: string;
  file_name: string | null;
  mime_type: string | null;
  size_bytes: number | null;
  content_sha256: string | null;
  grc_policy_id: string | null;
  grc_control_id: string | null;
  incident_id: string | null;
  owner_id: string | null;
  approved_by: string | null;
  approved_at: string | null;
  effective_date: string | null;
  expires_at: string | null;
  download_count: number;
  created_at: string;
  updated_at: string;
}

/** Mirrors the portal's `DocumentSummary`. */
interface DocumentSummary {
  total: number;
  published: number;
  draft: number;
  in_review: number;
  expired: number;
  expiring_soon: number;
  total_size_bytes: number;
}

const statusVariants: Record<string, "default" | "secondary" | "outline" | "destructive"> = {
  published: "default",
  draft: "secondary",
  in_review: "outline",
  archived: "outline",
  retired: "destructive",
};

const classificationVariants: Record<string, "critical" | "high" | "medium" | "low"> = {
  restricted: "critical",
  confidential: "high",
  internal: "medium",
  public: "low",
};

/** The `/documents/expiring` endpoint returns both past-due and upcoming docs. */
function isExpired(iso: string | null): boolean {
  if (!iso) return false;
  const ms = new Date(iso).getTime();
  return !Number.isNaN(ms) && ms < Date.now();
}

/** Absolute date for expiry columns, where "in 3 months" is less useful. */
function formatDate(iso: string | null): string {
  if (!iso) return "—";
  const ms = new Date(iso).getTime();
  if (Number.isNaN(ms)) return "—";
  return new Date(ms).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export default function DocumentsPage() {
  const summary = useApiData<DocumentSummary>(() => api.documents.summary());
  const documents = useApiList<DocumentRecord>(() => api.documents.list());
  const expiring = useApiData<DocumentRecord[]>(() => api.documents.expiring());

  const expiringDocs = expiring.data ?? [];

  const [showCreate, setShowCreate] = useState(false);
  const [createForm, setCreateForm] = useState({ title: "", description: "", doc_type: "policy" });
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  const refreshAll = () => {
    summary.refetch();
    documents.refetch();
    expiring.refetch();
  };

  const createDocument = async () => {
    if (!createForm.title.trim()) return;
    setSaving(true);
    setSaveError(null);
    try {
      await api.documents.create({
        title: createForm.title,
        description: createForm.description || undefined,
        doc_type: createForm.doc_type,
      });
      setShowCreate(false);
      setCreateForm({ title: "", description: "", doc_type: "policy" });
      refreshAll();
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : "Failed to create document");
    } finally {
      setSaving(false);
    }
  };

  const removeDocument = async (doc: DocumentRecord) => {
    if (!window.confirm(`Delete document "${doc.title}"?`)) return;
    try {
      await api.documents.remove(doc.id);
      refreshAll();
    } catch (err) {
      window.alert(err instanceof Error ? err.message : "Failed to delete document");
    }
  };

  const stats = [
    {
      label: "Total Documents",
      value: formatNumber(summary.data?.total),
      icon: FileText,
      color: "text-blue-400",
    },
    {
      label: "Published",
      value: formatNumber(summary.data?.published),
      icon: CheckCircle2,
      color: "text-green-400",
    },
    {
      label: "In Review",
      value: formatNumber(summary.data?.in_review),
      icon: Eye,
      color: "text-yellow-400",
    },
    {
      label: "Draft",
      value: formatNumber(summary.data?.draft),
      icon: PencilLine,
      color: "text-muted-foreground",
    },
    {
      label: "Expiring Soon",
      value: formatNumber(summary.data?.expiring_soon),
      icon: CalendarClock,
      color: "text-orange-400",
    },
    {
      label: "Expired",
      value: formatNumber(summary.data?.expired),
      icon: CalendarClock,
      color: "text-red-400",
    },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Security Documents</h2>
          <p className="text-muted-foreground">Policies, runbooks, and reference materials</p>
        </div>
        <div className="flex items-center gap-3">
          <Badge variant="outline" className="text-sm">
            {formatBytes(summary.data?.total_size_bytes)} stored
          </Badge>
          <Button onClick={() => setShowCreate(true)}><Plus className="h-4 w-4 mr-2" aria-hidden="true" />New Document</Button>
        </div>
      </div>

      <DataState
        loading={summary.loading}
        error={summary.error}
        onRetry={summary.refetch}
        loadingLabel="Loading document summary"
      >
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6">
          {stats.map((stat) => (
            <Card key={stat.label} className="border-border">
              <CardContent className="p-4">
                <div className="flex items-center gap-3">
                  <stat.icon className={`h-7 w-7 shrink-0 ${stat.color}`} aria-hidden="true" />
                  <div>
                    <p className="text-2xl font-bold text-foreground">{stat.value}</p>
                    <p className="text-xs text-muted-foreground">{stat.label}</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </DataState>


      {/* Expiring Soon */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-lg">
            <CalendarClock className="h-5 w-5 text-orange-400" aria-hidden="true" />
            Expiring Soon
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <DataState
            loading={expiring.loading}
            error={expiring.error}
            isEmpty={expiringDocs.length === 0}
            onRetry={expiring.refetch}
            loadingLabel="Loading expiring documents"
            emptyTitle="Nothing expiring"
            emptyDescription="Documents due to expire within 30 days appear here."
          >
            <div className="space-y-2">
              {expiringDocs.map((doc) => (
                <div
                  key={doc.id}
                  className="flex items-center justify-between rounded-lg border border-border px-4 py-3"
                >
                  <div className="space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-foreground">{doc.title}</span>
                      <Badge variant="outline" className="text-xs">v{doc.version}</Badge>
                      {classificationVariants[doc.classification] && (
                        <Badge variant={classificationVariants[doc.classification]} className="text-xs">
                          {doc.classification}
                        </Badge>
                      )}
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {doc.doc_type.replace(/_/g, " ")}
                      {doc.category ? ` • ${doc.category}` : ""}
                    </p>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="text-xs text-muted-foreground">
                      Expires {formatDate(doc.expires_at)}
                    </span>
                    {isExpired(doc.expires_at) && (
                      <Badge variant="destructive" className="text-xs">expired</Badge>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </DataState>
        </CardContent>
      </Card>


      {/* Document Library */}
      <Card className="border-border">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2 text-lg">
              <FileText className="h-5 w-5 text-primary" aria-hidden="true" />
              Document Library
            </CardTitle>
            <span className="text-xs text-muted-foreground">{formatNumber(documents.total)} documents</span>
          </div>
        </CardHeader>
        <CardContent className="p-0">
          <DataState
            loading={documents.loading}
            error={documents.error}
            isEmpty={documents.items.length === 0}
            onRetry={documents.refetch}
            loadingLabel="Loading documents"
            emptyTitle="No documents yet"
            emptyDescription="Policies, runbooks and evidence uploaded to the platform appear here."
          >
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <caption className="sr-only">
                  Security documents with type, status, classification, version, size and expiry date.
                </caption>
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Title</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Type</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Status</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Classification</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Version</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Size</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Expires</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Updated</th>
                    <th scope="col" className="px-4 py-3 text-left font-medium text-muted-foreground">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {documents.items.map((doc) => (
                    <tr key={doc.id} className="border-b border-border transition-colors hover:bg-muted/20">
                      <td className="px-4 py-3">
                        <p className="font-medium text-foreground">{doc.title}</p>
                        {doc.file_name && (
                          <p className="font-mono text-xs text-muted-foreground">{doc.file_name}</p>
                        )}
                      </td>
                      <td className="px-4 py-3 text-muted-foreground">
                        {doc.doc_type.replace(/_/g, " ")}
                        {doc.category && (
                          <p className="text-xs text-muted-foreground">{doc.category}</p>
                        )}
                      </td>
                      <td className="px-4 py-3">
                        <Badge variant={statusVariants[doc.status] ?? "outline"}>
                          {doc.status.replace(/_/g, " ")}
                        </Badge>
                      </td>
                      <td className="px-4 py-3">
                        {classificationVariants[doc.classification] ? (
                          <Badge variant={classificationVariants[doc.classification]}>
                            {doc.classification}
                          </Badge>
                        ) : (
                          <span className="text-xs text-muted-foreground">{doc.classification}</span>
                        )}
                      </td>
                      <td className="px-4 py-3 font-mono text-xs text-muted-foreground">v{doc.version}</td>
                      <td className="px-4 py-3 text-muted-foreground">{formatBytes(doc.size_bytes)}</td>
                      <td className="px-4 py-3 text-muted-foreground">{formatDate(doc.expires_at)}</td>
                      <td className="px-4 py-3 text-xs text-muted-foreground">{relativeTime(doc.updated_at)}</td>
                      <td className="px-4 py-3">
                        <Button variant="ghost" size="sm" onClick={() => removeDocument(doc)} className="text-red-400 hover:text-red-300" aria-label={`Delete ${doc.title}`}>
                          <Trash2 className="h-4 w-4" aria-hidden="true" />
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </DataState>
        </CardContent>
      </Card>

      {/* Create Document Modal */}
      {showCreate && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" role="dialog" aria-modal="true" aria-labelledby="create-doc-title">
          <div className="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-xl">
            <div className="mb-4 flex items-center justify-between">
              <h3 id="create-doc-title" className="text-lg font-semibold text-foreground">New Document</h3>
              <Button variant="ghost" size="icon" onClick={() => setShowCreate(false)} aria-label="Close modal"><X className="h-4 w-4" aria-hidden="true" /></Button>
            </div>
            <div className="space-y-4">
              <div className="space-y-1">
                <Label htmlFor="doc-title">Title</Label>
                <Input id="doc-title" placeholder="Security Policy v2" value={createForm.title} onChange={(e) => setCreateForm({ ...createForm, title: e.target.value })} />
              </div>
              <div className="space-y-1">
                <Label htmlFor="doc-desc">Description</Label>
                <Input id="doc-desc" placeholder="Optional description" value={createForm.description} onChange={(e) => setCreateForm({ ...createForm, description: e.target.value })} />
              </div>
              <div className="space-y-1">
                <Label htmlFor="doc-type">Type</Label>
                <select id="doc-type" className="w-full rounded border border-border bg-background px-3 py-2 text-sm" value={createForm.doc_type} onChange={(e) => setCreateForm({ ...createForm, doc_type: e.target.value })}>
                  <option value="policy">Policy</option>
                  <option value="procedure">Procedure</option>
                  <option value="runbook">Runbook</option>
                  <option value="standard">Standard</option>
                  <option value="evidence">Evidence</option>
                </select>
              </div>
              {saveError && <p className="text-sm text-destructive">{saveError}</p>}
              <div className="flex justify-end gap-2 pt-2">
                <Button variant="outline" onClick={() => setShowCreate(false)}>Cancel</Button>
                <Button onClick={createDocument} disabled={saving || !createForm.title.trim()}>
                  {saving ? "Creating..." : "Create Document"}
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

