import { redirect } from "next/navigation";

export default function Home() {
  // In production, check setup status and auth state
  // For now, redirect to dashboard
  redirect("/dashboard");
}
