import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { RealtimeNotifications } from "@/components/notifications/realtime-notifications";
import { AuthGuard } from "@/components/auth/auth-guard";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <AuthGuard>
      <div className="min-h-screen bg-background">
        <Sidebar />
        <div className="pl-64">
          <Header />
          <main className="p-6">{children}</main>
        </div>
        <RealtimeNotifications />
      </div>
    </AuthGuard>
  );
}
