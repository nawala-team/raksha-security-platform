import { redirect } from "next/navigation";

export default function Home() {
  // Send visitors to /login; the dashboard's AuthGuard forwards already
  // authenticated users straight through, so this is the safe default.
  redirect("/login");
}
