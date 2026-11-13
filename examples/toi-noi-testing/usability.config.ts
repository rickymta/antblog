import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: 'articles/system-testing-usability-test/e2e', workers: 1, retries: 0,
  reporter: [['list']], outputDir: 'reports/usability',
  use: { baseURL: 'http://127.0.0.1:3117', channel: 'chrome', headless: true,
    viewport: { width: 1280, height: 800 }, trace: 'retain-on-failure', screenshot: 'only-on-failure' },
  webServer: {
    command: 'npx tsx src/server.ts', url: 'http://127.0.0.1:3117/api/health',
    reuseExistingServer: false, timeout: 60_000,
    env: { PORT: '3117', NODE_ENV: 'test', TOINOI_TEST_API: '1', TOINOI_DB: 'pglite',
      TOINOI_RAIN: '', TOINOI_START_AT: '2026-11-04T09:00:00+07:00',
      TRIP_RATE_LIMIT_MAX: '5', TRIP_RATE_LIMIT_WINDOW_MS: '60000' },
  },
});
