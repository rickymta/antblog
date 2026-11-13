import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: 'articles/system-testing-system-test/e2e',
  workers: 1,
  retries: 0,
  timeout: 30_000,
  expect: { timeout: 8_000 },
  reporter: [['list']],
  outputDir: 'reports/system-testing',
  use: {
    baseURL: 'http://127.0.0.1:3115',
    channel: 'chrome',
    headless: true,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  webServer: {
    command: 'npx tsx src/server.ts',
    url: 'http://127.0.0.1:3115/api/health',
    reuseExistingServer: false,
    timeout: 60_000,
    env: {
      PORT: '3115', NODE_ENV: 'test', TOINOI_TEST_API: '1',
      TOINOI_DB: 'pglite', TOINOI_RAIN: '',
      TOINOI_START_AT: '2026-11-04T09:00:00+07:00',
      TRIP_RATE_LIMIT_MAX: '5', TRIP_RATE_LIMIT_WINDOW_MS: '60000',
    },
  },
});
