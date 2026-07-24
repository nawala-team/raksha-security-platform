import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";
import { RealtimeNotifications } from "@/components/notifications/realtime-notifications";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="min-h-screen bg-background">
      <Sidebar />
      <div className="pl-64">
        <Header />
        <main className="p-6">{children}</main>
      </div>
      <RealtimeNotifications />
    </div>
  );
}
