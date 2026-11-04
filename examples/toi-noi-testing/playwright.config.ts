import { defineConfig } from '@playwright/test';

/**
 * E2E chạy bằng Chrome đã cài trên máy (channel: 'chrome') — lab KHÔNG tải trình duyệt của Playwright.
 * Máy chủ lab được khởi động tự động với API kiểm thử bật và đồng hồ dời về 09:00 giờ VN (ngoài cao điểm).
 */
export default defineConfig({
  testDir: 'articles',
  testMatch: '**/e2e/**/*.spec.ts',
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [['list']],
  outputDir: 'reports/playwright',
  use: {
    baseURL: 'http://localhost:3100',
    channel: 'chrome',
    headless: true,
    trace: 'retain-on-failure',
  },
  webServer: {
    command: 'npx tsx src/server.ts',
    url: 'http://localhost:3100/api/health',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
    env: {
      PORT: '3100',
      NODE_ENV: 'test',
      TOINOI_START_AT: '2026-11-04T09:00:00+07:00',
      TRIP_RATE_LIMIT_MAX: '0',
    },
  },
});
