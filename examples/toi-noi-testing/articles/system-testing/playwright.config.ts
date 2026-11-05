import { defineConfig } from '@playwright/test';

/**
 * TEST-014 — cấu hình Playwright RIÊNG của bài: máy chủ lab ở cổng 3114 (không đụng cổng 3100 mà bài khác
 * có thể đang dùng), API kiểm thử bật, đồng hồ 09:00 giờ VN 04/11/2026, tắt giới hạn tần suất.
 * Chạy (từ examples/toi-noi-testing): npx playwright test -c articles/system-testing/playwright.config.ts
 * Spec cũng chạy được bằng cấu hình gốc (npm run test:e2e) vì chỉ dùng đường dẫn tương đối.
 */
export default defineConfig({
  testDir: '.',
  testMatch: 'e2e/**/*.spec.ts',
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [['list']],
  outputDir: '../../reports/playwright/system-testing',
  use: {
    baseURL: 'http://localhost:3114',
    channel: 'chrome',
    headless: true,
    trace: 'retain-on-failure',
  },
  webServer: {
    command: 'npx tsx src/server.ts',
    cwd: '../..',
    url: 'http://localhost:3114/api/health',
    reuseExistingServer: false,
    timeout: 60_000,
    env: {
      PORT: '3114',
      NODE_ENV: 'test',
      TOINOI_START_AT: '2026-11-04T09:00:00+07:00',
      TRIP_RATE_LIMIT_MAX: '0',
    },
  },
});
