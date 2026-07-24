import { FileText, Download, Plus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { SecurityDocument } from "@/types";

const mockDocuments: SecurityDocument[] = [
  { id: "1", title: "Incident Response Plan", category: "Policy", version: "3.2", lastUpdated: "2024-01-10", author: "Security Team", status: "published" },
  { id: "2", title: "Network Security Architecture", category: "Architecture", version: "2.1", lastUpdated: "2024-01-08", author: "Network Team", status: "published" },
  { id: "3", title: "Data Classification Policy", category: "Policy", version: "1.5", lastUpdated: "2024-01-05", author: "Compliance Team", status: "published" },
  { id: "4", title: "Disaster Recovery Runbook", category: "Runbook", version: "4.0-draft", lastUpdated: "2024-01-14", author: "Operations", status: "draft" },
  { id: "5", title: "Access Control Matrix", category: "Reference", version: "2.0", lastUpdated: "2024-01-12", author: "IAM Team", status: "published" },
  { id: "6", title: "Legacy VPN Decommission Plan", category: "Architecture", version: "1.0", lastUpdated: "2023-12-01", author: "Network Team", status: "archived" },
];

const statusColors = { published: "default" as const, draft: "secondary" as const, archived: "outline" as const };

export default function DocumentsPage() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-foreground">Security Documents</h2>
          <p className="text-muted-foreground">Policies, runbooks, and reference materials</p>
        </div>
        <Button><Plus className="h-4 w-4 mr-2" />New Document</Button>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
        {mockDocuments.map((doc) => (
          <Card key={doc.id} className="border-border hover:border-primary/30 transition-colors">
            <CardHeader className="pb-3">
              <div className="flex items-start justify-between">
                <CardTitle className="text-sm flex items-center gap-2">
                  <FileText className="h-4 w-4 text-primary shrink-0" />
                  {doc.title}
                </CardTitle>
                <Badge variant={statusColors[doc.status]}>{doc.status}</Badge>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-2 text-xs text-muted-foreground">
                <div className="flex justify-between">
                  <span>Category</span><span>{doc.category}</span>
                </div>
                <div className="flex justify-between">
                  <span>Version</span><span>{doc.version}</span>
                </div>
                <div className="flex justify-between">
                  <span>Author</span><span>{doc.author}</span>
                </div>
                <div className="flex justify-between">
                  <span>Updated</span><span>{new Date(doc.lastUpdated).toLocaleDateString()}</span>
                </div>
              </div>
              <div className="mt-3 pt-3 border-t border-border">
                <Button variant="outline" size="sm" className="w-full">
                  <Download className="h-3 w-3 mr-2" />Download
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
