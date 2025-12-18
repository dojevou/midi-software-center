import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['chromium'] },
    },
  ],

  // Web server already running via pnpm dev
  // Comment out webServer to use existing server
  // webServer: {
  //   command: 'npm run tauri dev',
  //   url: 'http://localhost:1420',
  //   reuseExistingServer: !process.env.CI,
  // },
});