import { test, expect } from '@playwright/test';

test.beforeEach(async ({ request }) => {
  expect((await request.post('/api/test/reset')).status()).toBe(200);
  expect((await request.post('/api/test/clock', { data: { now: '2026-11-04T09:00:00+07:00' } })).status()).toBe(200);
});

test('US-A01 · nhãn, đoạn tab order và kích hoạt báo giá bằng Enter', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByLabel('Mã khách hàng', { exact: true })).toHaveValue('C001');
  await page.getByLabel('Quãng đường (km)', { exact: true }).fill('5');
  await page.getByLabel('Mã giảm giá (không bắt buộc)', { exact: true }).focus();
  await page.keyboard.press('Tab');
  await expect(page.getByLabel('Thanh toán', { exact: true })).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(page.getByRole('button', { name: 'Báo giá', exact: true })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.getByTestId('quote-total')).toHaveText('35.000 đ');
  await expect(page.getByRole('region', { name: 'Báo giá', exact: true })).toHaveAttribute('aria-live', 'polite');
  await expect(page.getByRole('button', { name: 'Đặt chuyến', exact: true })).toBeEnabled();
});

test('US-A02 · lỗi có thông báo, sửa dữ liệu và báo giá lại được', async ({ page }) => {
  await page.goto('/');
  await page.getByLabel('Quãng đường (km)', { exact: true }).fill('0.1');
  await page.getByRole('button', { name: 'Báo giá', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Quãng đường');
  await expect(page.getByRole('button', { name: 'Đặt chuyến', exact: true })).toBeDisabled();
  await page.getByLabel('Quãng đường (km)', { exact: true }).fill('5');
  await page.getByRole('button', { name: 'Báo giá', exact: true }).click();
  await expect(page.getByRole('alert')).toBeEmpty();
  await expect(page.getByTestId('quote-total')).toHaveText('35.000 đ');
  await expect(page.getByRole('button', { name: 'Đặt chuyến', exact: true })).toBeEnabled();
});
