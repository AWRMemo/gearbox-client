import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './playwright',
  timeout: 30000,
  expect: { timeout: 10000 },
  fullyParallel: false,
  retries: 1,
  use: {
    baseURL: 'http://localhost:1420',
    headless: true,
  },
  webServer: {
    command: 'pnpm dev',
    port: 1420,
    reuseExistingServer: true,
  },
})
