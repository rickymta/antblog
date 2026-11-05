/**
 * TEST-014 — System testing qua giao diện: khách đặt ô tô ở / và tài xế hoàn thành ở /driver.html.
 * Test basis: SPEC R-UI-01, R-UI-02, R-STATE-02, R-STATE-03, R-ACCEPT-01, R-ACCEPT-06, R-FARE-01, R-NFR-04.
 * Máy chủ lab: 09:00 giờ VN 04/11/2026 (ngoài cao điểm), không mưa, dữ liệu mẫu Phụ lục A.
 * Chạy: npx playwright test -c articles/system-testing/playwright.config.ts
 */
import { expect, test, type Browser, type Page } from '@playwright/test';

test.beforeEach(async ({ request }) => {
  await request.post('/api/test/reset');
});

async function openPage(browser: Browser, path: string): Promise<Page> {
  const page = await (await browser.newContext()).newPage(); // hai người dùng ⇒ hai phiên trình duyệt riêng
  await page.goto(path);
  return page;
}

/** Khách C001 báo giá ô tô 7,4 km ở TP.HCM rồi đặt chuyến (bấm đúp). Trả mã chuyến. */
async function bookCar(customer: Page): Promise<string> {
  await customer.getByLabel('Loại xe').selectOption('car');
  await customer.getByLabel('Quãng đường (km)').fill('7.4');
  await customer.getByRole('button', { name: 'Báo giá' }).click();
  await expect(customer.getByTestId('quote-total')).toHaveText('107.000 đ'); // 25.000 + 11.000 × 7,4 = 106.400 ⇒ 107.000
  await expect(customer.getByTestId('quote-peak')).toHaveText('Không');
  await customer.getByRole('button', { name: 'Đặt chuyến' }).dblclick();
  await expect(customer.getByTestId('trip-status')).toHaveText('Đang tìm tài xế');
  return (await customer.getByTestId('trip-id').textContent()) ?? '';
}

async function loadAvailable(driver: Page, driverId: string): Promise<void> {
  await driver.getByLabel('Mã tài xế').fill(driverId);
  await driver.getByRole('button', { name: 'Tải chuyến đang chờ' }).click();
}

/** Thời gian (ms) từ lúc tài xế bấm tới lúc trang khách tự hiện trạng thái mới (R-NFR-04: p95 dưới 2–3 giây). */
async function propagation(customer: Page, label: string, act: () => Promise<void>): Promise<number> {
  const t0 = Date.now();
  await act();
  await expect(customer.getByTestId('trip-status')).toHaveText(label, { timeout: 3_000 });
  return Date.now() - t0;
}

test('TC-SYS-01…05 · khách đặt ô tô, chỉ tài xế ô tô thấy chuyến, D103 đi hết các bước, khách tự thấy Hoàn thành', async ({ browser }) => {
  const customer = await openPage(browser, '/');
  const driver = await openPage(browser, '/driver.html');

  const tripId = await bookCar(customer); // TC-SYS-01, TC-SYS-02

  await loadAvailable(driver, 'D101'); // TC-SYS-03: xe máy cùng thành phố không thấy chuyến ô tô
  await expect(driver.getByTestId('available-trips')).toHaveText('Chưa có chuyến nào.');
  await loadAvailable(driver, 'D103');
  await expect(driver.getByTestId('available-trips').locator('li')).toHaveCount(1); // bấm đúp không sinh chuyến thứ hai
  await expect(driver.getByTestId(`available-${tripId}`)).toContainText('107.000 đ');

  const steps: [string, string][] = [
    ['action-arrive', 'Tài xế đang tới'],
    ['action-pickup', 'Đã đón / lấy hàng'],
    ['action-start', 'Đang chở / giao'],
    ['action-complete', 'Hoàn thành'],
  ];
  const timings: Record<string, number> = {};
  timings['Tài xế đã nhận'] = await propagation(customer, 'Tài xế đã nhận', () => driver.getByTestId(`accept-${tripId}`).click());
  await expect(customer.getByTestId('trip-driver')).toHaveText('D103');
  for (const [testId, label] of steps) {
    // TC-SYS-04: trang tài xế đổi trạng thái đúng thứ tự; TC-SYS-05: trang khách tự cập nhật
    timings[label] = await propagation(customer, label, async () => {
      await driver.getByTestId(testId).click();
      await expect(driver.getByTestId('driver-trip-status')).toHaveText(label);
    });
  }
  await expect(customer.getByTestId('cancel-button')).toBeDisabled(); // chuyến kết thúc: không huỷ được nữa
  console.log(`Lan truyền trạng thái tới trang khách (ms): ${JSON.stringify(timings)}`);
});

test('TC-SYS-06 · bấm «Hoàn thành» khi mới nhận chuyến ⇒ bị từ chối, trạng thái hai phía giữ nguyên', async ({ browser }) => {
  const customer = await openPage(browser, '/');
  const driver = await openPage(browser, '/driver.html');
  const tripId = await bookCar(customer);
  await loadAvailable(driver, 'D103');
  await driver.getByTestId(`accept-${tripId}`).click();
  await expect(driver.getByTestId('driver-trip-status')).toHaveText('Tài xế đã nhận');

  await driver.getByTestId('action-complete').click(); // bỏ qua arrive, pickup, start (R-STATE-03)
  await expect(driver.getByRole('alert')).toContainText('Không thể thực hiện');
  console.log(`Thông báo lỗi trang tài xế: ${await driver.getByRole('alert').textContent()}`);
  await expect(driver.getByTestId('driver-trip-status')).toHaveText('Tài xế đã nhận');
  await expect(customer.getByTestId('trip-status')).toHaveText('Tài xế đã nhận', { timeout: 3_000 });
});
