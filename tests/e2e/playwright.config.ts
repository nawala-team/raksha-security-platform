import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright E2E test configuration for Raksha Security Platform.
 *
 * Run with: npx playwright test
 * UI mode: npx playwright test --ui
 */
export default defineConfig({
  testDir: "./specs",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI
    ? [["html", { open: "never" }], ["junit", { outputFile: "results.xml" }]]
    : [["html", { open: "on-failure" }]],
  timeout: 30_000,
  expect: {
    timeout: 5_000,
  },

  use: {
    baseURL: process.env.E2E_BASE_URL || "http://localhost:3000",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "on-first-retry",
    actionTimeout: 10_000,
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "firefox",
      use: { ...devices["Desktop Firefox"] },
    },
    {
      name: "webkit",
      use: { ...devices["Desktop Safari"] },
    },
    {
      name: "mobile-chrome",
      use: { ...devices["Pixel 5"] },
    },
  ],

  webServer: process.env.CI
    ? undefined
    : {
        command: "npm run dev",
        cwd: "../../apps/web",
        url: "http://localhost:3000",
        reuseExistingServer: !process.env.CI,
        timeout: 120_000,
      },
});
