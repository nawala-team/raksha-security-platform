import { test, expect } from "@playwright/test";

test.describe("Login Flow", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/login");
  });

  test("displays login page with correct elements", async ({ page }) => {
    // Branding
    await expect(page.getByText("Raksha")).toBeVisible();
    await expect(page.getByText("Security Platform")).toBeVisible();

    // Form fields
    await expect(page.getByLabel("Email")).toBeVisible();
    await expect(page.getByLabel("Password")).toBeVisible();
    await expect(page.getByRole("button", { name: "Sign In" })).toBeVisible();
  });

  test("shows validation for empty form submission", async ({ page }) => {
    // Try to submit empty form - HTML5 validation should prevent it
    await page.getByRole("button", { name: "Sign In" }).click();

    // Email field should show validation message (browser-native)
    const emailInput = page.getByLabel("Email");
    const validationMessage = await emailInput.evaluate(
      (el: HTMLInputElement) => el.validationMessage
    );
    expect(validationMessage).not.toBe("");
  });

  test("transitions to MFA step after credential submission", async ({
    page,
  }) => {
    await page.getByLabel("Email").fill("admin@organization.com");
    await page.getByLabel("Password").fill("SecurePassword123!");
    await page.getByRole("button", { name: "Sign In" }).click();

    // Wait for MFA step
    await expect(
      page.getByText("Multi-Factor Authentication")
    ).toBeVisible({ timeout: 5000 });
    await expect(page.getByLabel("Authentication Code")).toBeVisible();
    await expect(page.getByRole("button", { name: "Verify" })).toBeVisible();
  });

  test("MFA input accepts only numeric characters", async ({ page }) => {
    // Get to MFA step
    await page.getByLabel("Email").fill("admin@organization.com");
    await page.getByLabel("Password").fill("SecurePassword123!");
    await page.getByRole("button", { name: "Sign In" }).click();

    await expect(page.getByLabel("Authentication Code")).toBeVisible({
      timeout: 5000,
    });

    // Type mixed characters
    await page.getByLabel("Authentication Code").fill("abc123def456");

    // Only digits should remain
    const value = await page.getByLabel("Authentication Code").inputValue();
    expect(value).toMatch(/^\d*$/);
  });

  test("back button returns to login form from MFA", async ({ page }) => {
    // Get to MFA step
    await page.getByLabel("Email").fill("admin@organization.com");
    await page.getByLabel("Password").fill("SecurePassword123!");
    await page.getByRole("button", { name: "Sign In" }).click();

    await expect(
      page.getByText("Multi-Factor Authentication")
    ).toBeVisible({ timeout: 5000 });

    // Click back
    await page.getByRole("button", { name: "Back to login" }).click();

    // Should show login form again
    await expect(page.getByLabel("Email")).toBeVisible();
    await expect(page.getByLabel("Password")).toBeVisible();
  });

  test("completes full login flow and redirects to dashboard", async ({
    page,
  }) => {
    // Credentials step
    await page.getByLabel("Email").fill("admin@organization.com");
    await page.getByLabel("Password").fill("SecurePassword123!");
    await page.getByRole("button", { name: "Sign In" }).click();

    // MFA step
    await expect(page.getByLabel("Authentication Code")).toBeVisible({
      timeout: 5000,
    });
    await page.getByLabel("Authentication Code").fill("123456");
    await page.getByRole("button", { name: "Verify" }).click();

    // Should redirect to dashboard
    await page.waitForURL("**/dashboard", { timeout: 10000 });
    expect(page.url()).toContain("/dashboard");
  });

  test("shows loading indicator during submission", async ({ page }) => {
    await page.getByLabel("Email").fill("admin@organization.com");
    await page.getByLabel("Password").fill("SecurePassword123!");

    // Check that button becomes disabled during submission
    await page.getByRole("button", { name: "Sign In" }).click();
    await expect(
      page.getByRole("button", { name: "Sign In" })
    ).toBeDisabled();
  });
});
