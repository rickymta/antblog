import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: 'articles/system-testing-compatibility-test/e2e',
  workers: 1, retries: 0, timeout: 30_000,
  reporter: [['list']], outputDir: 'reports/compatibility',
  use: { baseURL: 'http://127.0.0.1:3116', headless: true,
    trace: 'retain-on-failure', screenshot: 'only-on-failure' },
  projects: [
    { name: 'chrome-desktop-vi', use: { channel: 'chrome', viewport: { width: 1280, height: 800 }, locale: 'vi-VN', timezoneId: 'Asia/Ho_Chi_Minh' } },
    { name: 'edge-desktop-en', use: { channel: 'msedge', viewport: { width: 1280, height: 800 }, locale: 'en-US', timezoneId: 'America/New_York' } },
    { name: 'chrome-narrow-touch', use: { channel: 'chrome', viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true, locale: 'vi-VN', timezoneId: 'Asia/Ho_Chi_Minh' } },
  ],
  webServer: {
    command: 'npx tsx src/server.ts', url: 'http://127.0.0.1:3116/api/health',
    reuseExistingServer: false, timeout: 60_000,
    env: { PORT: '3116', NODE_ENV: 'test', TOINOI_TEST_API: '1', TOINOI_DB: 'pglite',
      TOINOI_RAIN: '', TOINOI_START_AT: '2026-11-04T09:00:00+07:00',
      TRIP_RATE_LIMIT_MAX: '5', TRIP_RATE_LIMIT_WINDOW_MS: '60000' },
  },
});
