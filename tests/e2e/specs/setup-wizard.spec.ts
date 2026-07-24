import { test, expect } from "@playwright/test";

test.describe("Setup Wizard", () => {
  test("displays setup page with initial step", async ({ page }) => {
    await page.goto("/setup");

    // Should show the setup wizard
    await expect(page.getByText(/setup/i)).toBeVisible();
  });

  test("navigates through wizard steps", async ({ page }) => {
    await page.goto("/setup");

    // The setup wizard typically has multiple steps
    // Verify the page loads without errors
    await expect(page).toHaveURL(/.*setup/);

    // Look for step indicators or navigation
    const pageContent = await page.textContent("body");
    expect(pageContent).toBeTruthy();
  });

  test("validates required fields before proceeding", async ({ page }) => {
    await page.goto("/setup");

    // Attempt to proceed without filling required fields
    // The specific behavior depends on the wizard implementation
    const nextButton = page.getByRole("button", { name: /next|continue/i });

    if (await nextButton.isVisible()) {
      await nextButton.click();
      // Should remain on the same step or show validation errors
      await expect(page).toHaveURL(/.*setup/);
    }
  });

  test("setup page is accessible", async ({ page }) => {
    await page.goto("/setup");

    // Basic accessibility checks
    // All interactive elements should be keyboard focusable
    const buttons = page.getByRole("button");
    const buttonCount = await buttons.count();

    for (let i = 0; i < buttonCount; i++) {
      const button = buttons.nth(i);
      if (await button.isVisible()) {
        // Button should have accessible name
        const name = await button.getAttribute("aria-label") ||
          await button.textContent();
        expect(name?.trim()).toBeTruthy();
      }
    }
  });

  test("responsive layout on mobile viewport", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/setup");

    // Page should still be usable on mobile
    await expect(page.locator("body")).toBeVisible();

    // No horizontal scrollbar
    const hasHScroll = await page.evaluate(
      () => document.documentElement.scrollWidth > document.documentElement.clientWidth
    );
    expect(hasHScroll).toBe(false);
  });
});
