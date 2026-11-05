import { defineConfig } from '@playwright/test';

/**
 * TEST-053 — cấu hình Playwright RIÊNG của bài: máy chủ lab ở cổng 3153 (3100 + 53), API kiểm thử bật,
 * đồng hồ 09:00 giờ VN 04/11/2026 (ngoài cao điểm), không mưa, tắt giới hạn tần suất.
 * Chạy (từ examples/toi-noi-testing): npx playwright test -c articles/checklist-based-testing/playwright.config.ts
 */
export default defineConfig({
  testDir: '.',
  testMatch: 'e2e/**/*.spec.ts',
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [['list']],
  outputDir: '../../reports/playwright/checklist-based-testing',
  use: {
    baseURL: 'http://localhost:3153',
    channel: 'chrome',
    headless: true,
    timezoneId: 'Asia/Ho_Chi_Minh',
    trace: 'retain-on-failure',
  },
  webServer: {
    command: 'npx tsx src/server.ts',
    cwd: '../..',
    url: 'http://localhost:3153/api/health',
    reuseExistingServer: false,
    timeout: 60_000,
    env: {
      PORT: '3153',
      NODE_ENV: 'test',
      TOINOI_START_AT: '2026-11-04T09:00:00+07:00',
      TRIP_RATE_LIMIT_MAX: '0',
    },
  },
});
