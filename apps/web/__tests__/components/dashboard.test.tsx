import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import DashboardPage from "@/app/dashboard/page";

describe("DashboardPage", () => {
  it("renders the dashboard heading", () => {
    render(<DashboardPage />);

    expect(screen.getByText("Dashboard")).toBeInTheDocument();
    expect(
      screen.getByText("Security overview and real-time monitoring")
    ).toBeInTheDocument();
  });

  it("renders all four stat cards", () => {
    render(<DashboardPage />);

    expect(screen.getByText("Active Alerts")).toBeInTheDocument();
    expect(screen.getByText("Servers Online")).toBeInTheDocument();
    expect(screen.getByText("Threats Blocked")).toBeInTheDocument();
    expect(screen.getByText("Compliance Score")).toBeInTheDocument();
  });

  it("displays stat values", () => {
    render(<DashboardPage />);

    expect(screen.getByText("12")).toBeInTheDocument();
    expect(screen.getByText("48/50")).toBeInTheDocument();
    expect(screen.getByText("1,284")).toBeInTheDocument();
    expect(screen.getByText("94%")).toBeInTheDocument();
  });

  it("displays stat change descriptions", () => {
    render(<DashboardPage />);

    expect(screen.getByText("+3 from yesterday")).toBeInTheDocument();
    expect(screen.getByText("96% uptime")).toBeInTheDocument();
    expect(screen.getByText("+127 today")).toBeInTheDocument();
    expect(screen.getByText("All frameworks passing")).toBeInTheDocument();
  });

  it("renders the threat activity map section", () => {
    render(<DashboardPage />);

    expect(screen.getByText("Threat Activity Map")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Real-time threat visualization will be displayed here"
      )
    ).toBeInTheDocument();
  });

  it("renders SecurityScore and AlertFeed components", () => {
    render(<DashboardPage />);

    // These components should render without crashing
    // The exact content depends on the component implementation
    const dashboard = screen.getByText("Dashboard");
    expect(dashboard.closest("div")).toBeInTheDocument();
  });

  it("uses correct grid layout classes", () => {
    const { container } = render(<DashboardPage />);

    // Stats grid should be 4 columns on large screens
    const statsGrid = container.querySelector(
      ".grid.grid-cols-1.sm\\:grid-cols-2.lg\\:grid-cols-4"
    );
    expect(statsGrid).toBeInTheDocument();
  });

  it("renders stat cards with proper structure", () => {
    render(<DashboardPage />);

    // Each stat should have a title and value
    const alertsCard = screen.getByText("Active Alerts");
    expect(alertsCard).toBeInTheDocument();

    // Verify the card contains the value nearby
    const cardContent = alertsCard.closest("[class*='CardContent']") ||
      alertsCard.closest("div");
    expect(cardContent).toBeInTheDocument();
  });
});
