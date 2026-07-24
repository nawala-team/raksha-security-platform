import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import LoginPage from "@/app/login/page";

// Mock window.location
const mockLocation = { href: "" };
Object.defineProperty(window, "location", {
  value: mockLocation,
  writable: true,
});

describe("LoginPage", () => {
  beforeEach(() => {
    mockLocation.href = "";
  });

  it("renders the login form with email and password fields", () => {
    render(<LoginPage />);

    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /sign in/i })).toBeInTheDocument();
  });

  it("renders the Raksha branding", () => {
    render(<LoginPage />);

    expect(screen.getByText("Raksha")).toBeInTheDocument();
    expect(screen.getByText("Security Platform")).toBeInTheDocument();
  });

  it("shows validation when submitting empty form", async () => {
    render(<LoginPage />);

    const emailInput = screen.getByLabelText(/email/i);
    const passwordInput = screen.getByLabelText(/password/i);

    // HTML5 required attribute should prevent submission
    expect(emailInput).toBeRequired();
    expect(passwordInput).toBeRequired();
  });

  it("allows typing into email and password fields", async () => {
    const user = userEvent.setup();
    render(<LoginPage />);

    const emailInput = screen.getByLabelText(/email/i);
    const passwordInput = screen.getByLabelText(/password/i);

    await user.type(emailInput, "admin@organization.com");
    await user.type(passwordInput, "SecurePassword123!");

    expect(emailInput).toHaveValue("admin@organization.com");
    expect(passwordInput).toHaveValue("SecurePassword123!");
  });

  it("shows MFA step after initial login submission", async () => {
    const user = userEvent.setup();
    render(<LoginPage />);

    // Fill in credentials
    await user.type(screen.getByLabelText(/email/i), "admin@test.com");
    await user.type(screen.getByLabelText(/password/i), "Password123!");

    // Submit the form
    await user.click(screen.getByRole("button", { name: /sign in/i }));

    // Wait for MFA step to appear
    await waitFor(() => {
      expect(screen.getByText(/multi-factor authentication/i)).toBeInTheDocument();
    });

    expect(screen.getByLabelText(/authentication code/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /verify/i })).toBeInTheDocument();
  });

  it("shows loading state during submission", async () => {
    const user = userEvent.setup();
    render(<LoginPage />);

    await user.type(screen.getByLabelText(/email/i), "admin@test.com");
    await user.type(screen.getByLabelText(/password/i), "Password123!");

    const submitButton = screen.getByRole("button", { name: /sign in/i });
    await user.click(submitButton);

    // Button should be disabled during loading
    expect(submitButton).toBeDisabled();
  });

  it("allows going back from MFA to login form", async () => {
    const user = userEvent.setup();
    render(<LoginPage />);

    // Submit credentials to get to MFA step
    await user.type(screen.getByLabelText(/email/i), "admin@test.com");
    await user.type(screen.getByLabelText(/password/i), "Password123!");
    await user.click(screen.getByRole("button", { name: /sign in/i }));

    // Wait for MFA step
    await waitFor(() => {
      expect(screen.getByText(/multi-factor authentication/i)).toBeInTheDocument();
    });

    // Click "Back to login"
    await user.click(screen.getByRole("button", { name: /back to login/i }));

    // Should show login form again
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
  });

  it("MFA input only accepts numeric characters", async () => {
    const user = userEvent.setup();
    render(<LoginPage />);

    // Get to MFA step
    await user.type(screen.getByLabelText(/email/i), "admin@test.com");
    await user.type(screen.getByLabelText(/password/i), "Password123!");
    await user.click(screen.getByRole("button", { name: /sign in/i }));

    await waitFor(() => {
      expect(screen.getByLabelText(/authentication code/i)).toBeInTheDocument();
    });

    const mfaInput = screen.getByLabelText(/authentication code/i);
    await user.type(mfaInput, "abc123");

    // Non-numeric chars should be filtered out
    expect(mfaInput).toHaveValue("123");
  });

  it("redirects to dashboard after MFA verification", async () => {
    const user = userEvent.setup();
    render(<LoginPage />);

    // Login step
    await user.type(screen.getByLabelText(/email/i), "admin@test.com");
    await user.type(screen.getByLabelText(/password/i), "Password123!");
    await user.click(screen.getByRole("button", { name: /sign in/i }));

    // MFA step
    await waitFor(() => {
      expect(screen.getByLabelText(/authentication code/i)).toBeInTheDocument();
    });

    await user.type(screen.getByLabelText(/authentication code/i), "123456");
    await user.click(screen.getByRole("button", { name: /verify/i }));

    // Should redirect to dashboard
    await waitFor(() => {
      expect(mockLocation.href).toBe("/dashboard");
    });
  });

  it("has proper autocomplete attributes for security", () => {
    render(<LoginPage />);

    const emailInput = screen.getByLabelText(/email/i);
    const passwordInput = screen.getByLabelText(/password/i);

    expect(emailInput).toHaveAttribute("autocomplete", "email");
    expect(passwordInput).toHaveAttribute("autocomplete", "current-password");
  });
});
