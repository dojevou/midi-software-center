import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  timeout: 90000,
  reporter: 'list',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'off',
    screenshot: 'off',
    video: 'off',
    actionTimeout: 15000,
    navigationTimeout: 60000,
    launchOptions: {
      args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
    },
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'], headless: true },
    },
  ],

  webServer: {
    command: 'cd app && pnpm dev',
    url: 'http://localhost:5173',
    reuseExistingServer: true,
    timeout: 120000,
  },
});