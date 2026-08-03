import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import LoginPage from "@/app/login/page";
import { api, apiClient } from "@/lib/api";

// Mock window.location
const mockLocation = { href: "" };
Object.defineProperty(window, "location", {
  value: mockLocation,
  writable: true,
});

describe("LoginPage", () => {
  beforeEach(() => {
    mockLocation.href = "";
    window.localStorage.clear();
    vi.restoreAllMocks();
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

  it("submits credentials to the auth API and stores the returned tokens", async () => {
    const tokens = {
      access_token: "access-abc",
      refresh_token: "refresh-abc",
      expires_in: 900,
      token_type: "Bearer",
    };
    const loginSpy = vi.spyOn(api.auth, "login").mockResolvedValue({
      user: {
        id: "user-1",
        email: "admin@test.com",
        name: "Admin",
        role: "super_admin",
      },
      tokens,
    } as never);
    const setTokenSpy = vi.spyOn(apiClient, "setToken");

    const user = userEvent.setup();
    render(<LoginPage />);

    await user.type(screen.getByLabelText(/email/i), "admin@test.com");
    await user.type(screen.getByLabelText(/password/i), "Password123!");
    await user.click(screen.getByRole("button", { name: /sign in/i }));

    await waitFor(() => {
      expect(loginSpy).toHaveBeenCalledWith({
        email: "admin@test.com",
        password: "Password123!",
      });
    });

    expect(window.localStorage.getItem("raksha_auth_token")).toBe(
      JSON.stringify(tokens)
    );
    expect(setTokenSpy).toHaveBeenCalledWith("access-abc");
  });

  it("redirects to the dashboard on successful login", async () => {
    vi.spyOn(api.auth, "login").mockResolvedValue({
      user: {
        id: "user-1",
        email: "admin@test.com",
        name: "Admin",
        role: "viewer",
      },
      tokens: {
        access_token: "access-abc",
        refresh_token: "refresh-abc",
        expires_in: 900,
        token_type: "Bearer",
      },
    } as never);

    const user = userEvent.setup();
    render(<LoginPage />);

    await user.type(screen.getByLabelText(/email/i), "admin@test.com");
    await user.type(screen.getByLabelText(/password/i), "Password123!");
    await user.click(screen.getByRole("button", { name: /sign in/i }));

    await waitFor(() => {
      expect(mockLocation.href).toBe("/dashboard");
    });
  });

  it("shows the API error message when login is rejected", async () => {
    vi.spyOn(api.auth, "login").mockRejectedValue(
      new Error("Invalid credentials")
    );

    const user = userEvent.setup();
    render(<LoginPage />);

    await user.type(screen.getByLabelText(/email/i), "admin@test.com");
    await user.type(screen.getByLabelText(/password/i), "WrongPassword1!");
    await user.click(screen.getByRole("button", { name: /sign in/i }));

    await waitFor(() => {
      expect(screen.getByText(/invalid credentials/i)).toBeInTheDocument();
    });

    // A failed login must not redirect or persist a token.
    expect(mockLocation.href).toBe("");
    expect(window.localStorage.getItem("raksha_auth_token")).toBeNull();
  });

  it("re-enables the submit button after a failed login", async () => {
    vi.spyOn(api.auth, "login").mockRejectedValue(new Error("Login failed"));

    const user = userEvent.setup();
    render(<LoginPage />);

    await user.type(screen.getByLabelText(/email/i), "admin@test.com");
    await user.type(screen.getByLabelText(/password/i), "Password123!");

    const submitButton = screen.getByRole("button", { name: /sign in/i });
    await user.click(submitButton);

    await waitFor(() => {
      expect(submitButton).not.toBeDisabled();
    });
  });

  it("disables the submit button while the request is in flight", async () => {
    let resolveLogin: (value: unknown) => void = () => {};
    vi.spyOn(api.auth, "login").mockReturnValue(
      new Promise((resolve) => {
        resolveLogin = resolve;
      }) as never
    );

    const user = userEvent.setup();
    render(<LoginPage />);

    await user.type(screen.getByLabelText(/email/i), "admin@test.com");
    await user.type(screen.getByLabelText(/password/i), "Password123!");

    const submitButton = screen.getByRole("button", { name: /sign in/i });
    await user.click(submitButton);

    await waitFor(() => {
      expect(submitButton).toBeDisabled();
    });

    resolveLogin({
      user: { id: "1", email: "a@b.c", name: "A", role: "viewer" },
      tokens: {
        access_token: "t",
        refresh_token: "r",
        expires_in: 900,
        token_type: "Bearer",
      },
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
