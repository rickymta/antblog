import { expect, test } from '@playwright/test';

// Kiểm khói: webServer + Chrome cài sẵn chạy được. Máy chủ lab ở 09:00 giờ VN (ngoài cao điểm), không mưa.
test.beforeEach(async ({ request }) => {
  await request.post('/api/test/reset'); // chỉ có khi NODE_ENV=test (playwright.config.ts bật sẵn)
});

test('khách báo giá 7,4 km xe máy rồi đặt chuyến', async ({ page }) => {
  await page.goto('/');
  await page.getByLabel('Quãng đường (km)').fill('7.4');
  await page.getByRole('button', { name: 'Báo giá' }).click();

  await expect(page.getByTestId('quote-total')).toHaveText('46.000 đ'); // 45.300 làm tròn lên
  await expect(page.getByTestId('quote-peak')).toHaveText('Không');

  await page.getByRole('button', { name: 'Đặt chuyến' }).click();
  await expect(page.getByTestId('trip-status')).toHaveText('Đang tìm tài xế');
});
