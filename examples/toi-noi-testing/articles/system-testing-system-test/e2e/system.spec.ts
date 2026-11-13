import { test, expect, type Page } from '@playwright/test';

test.beforeEach(async ({ request }) => {
  expect((await request.post('/api/test/reset')).status()).toBe(200);
  expect((await request.post('/api/test/clock', { data: { now: '2026-11-04T09:00:00+07:00' } })).status()).toBe(200);
});

async function book(page: Page) {
  await page.goto('/');
  await page.getByTestId('distance').fill('5');
  await page.getByTestId('payment-method').selectOption('vixanh');
  await page.getByTestId('quote-button').click();
  await expect(page.getByTestId('quote-total')).toHaveText('35.000 đ');
  await page.getByTestId('book-button').click();
  await expect(page.getByTestId('trip-status')).toHaveAttribute('data-status', 'searching');
  const id = await page.getByTestId('trip-id').textContent();
  expect(id).toBeTruthy();
  return id!;
}

test('SY-01 · khách đặt, tài xế hoàn tất, hai UI và API thống nhất', async ({ page, browser, request }) => {
  const id = await book(page);
  const driverContext = await browser.newContext();
  try {
    const driver = await driverContext.newPage();
    await driver.goto(new URL('/driver.html', page.url()).href);
    await driver.getByTestId('load-available').click();
    await driver.getByTestId(`accept-${id}`).click();
    await expect(driver.getByTestId('current-trip-id')).toHaveText(id);
    await expect(page.getByTestId('trip-driver')).toHaveText('D101');
    for (const [action, label] of [
      ['arrive', 'Tài xế đang tới'], ['pickup', 'Đã đón / lấy hàng'],
      ['start', 'Đang chở / giao'], ['complete', 'Hoàn thành'],
    ]) {
      await driver.getByTestId(`action-${action}`).click();
      await expect(driver.getByTestId('driver-trip-status')).toHaveText(label!);
    }
    await expect(page.getByTestId('trip-status')).toHaveText('Hoàn thành');
    await expect(page.getByTestId('cancel-button')).toBeDisabled();
    const res = await request.get(`/api/trips/${id}`, { headers: { 'x-user': 'customer:C001' } });
    expect(res.status()).toBe(200);
    expect((await res.json()).trip).toMatchObject({ id, status: 'completed', paymentStatus: 'paid', fare: 35_000, driverId: 'D101' });
    const other = await request.get(`/api/trips/${id}`, { headers: { 'x-user': 'customer:C002' } });
    expect(other.status()).toBe(404);
    expect((await other.json()).error.code).toBe('TRIP_NOT_FOUND');
  } finally { await driverContext.close(); }
});

test('SY-02 · khách huỷ khi đang tìm tài xế, UI và API cùng ghi nhận phí 0', async ({ page, request }) => {
  const id = await book(page);
  await page.getByTestId('cancel-button').click();
  await expect(page.getByTestId('trip-status')).toHaveText('Đã huỷ');
  await expect(page.getByTestId('cancel-fee')).toHaveText('Phí huỷ: 0 đ');
  await expect(page.getByTestId('cancel-button')).toBeDisabled();
  const res = await request.get(`/api/trips/${id}`, { headers: { 'x-user': 'customer:C001' } });
  expect(res.status()).toBe(200);
  expect((await res.json()).trip).toMatchObject({ status: 'cancelled', cancellationFee: 0, paymentStatus: 'none' });
});

test('SY-03 · sửa dữ liệu làm vô hiệu báo giá cũ và nhận lỗi nghiệp vụ qua UI', async ({ page }) => {
  await page.goto('/');
  await page.getByTestId('distance').fill('5');
  await page.getByTestId('quote-button').click();
  await expect(page.getByTestId('quote-total')).toHaveText('35.000 đ');
  await expect(page.getByTestId('book-button')).toBeEnabled();
  await page.getByTestId('distance').fill('0.1');
  await expect(page.getByTestId('book-button')).toBeDisabled();
  await expect(page.getByTestId('quote-result')).toBeHidden();
  const response = page.waitForResponse(r => r.url().endsWith('/api/quotes') && r.request().method() === 'POST');
  await page.getByTestId('quote-button').click();
  const res = await response;
  expect(res.status()).toBe(422);
  expect((await res.json()).error.code).toBe('DISTANCE_OUT_OF_RANGE');
  await expect(page.getByTestId('error')).toContainText('Quãng đường');
  await expect(page.getByTestId('book-button')).toBeDisabled();
});
