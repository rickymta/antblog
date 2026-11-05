/**
 * TEST-053 — chạy checklist smoke CL-SMOKE v1.0 (../checklist-smoke-v1.0.md) trên giao diện web của lab.
 *
 * Spec này là «người thực hiện có ghi chép»: mỗi bước mang mã mục checklist (SM-xx), làm đúng thao tác
 * mà mục mô tả và GHI LẠI điều quan sát được (chữ trên màn hình, trạng thái qua API).
 *  - Mục read-do có kết quả kiểm được ngay (số tiền, trạng thái) ⇒ khẳng định cứng (expect): sai là chặn phát hành.
 *  - Mục do-confirm cần phán đoán (thông báo có dễ hiểu không, khách có biết đã trả tiền chưa…)
 *    ⇒ spec chỉ ghi bằng chứng, người kiểm thử đánh giá (kết quả trong bài viết).
 * Cuối phiên in nhật ký quan sát. Máy chủ: 09:00 giờ VN 04/11/2026, không mưa, dữ liệu mẫu SPEC Phụ lục A.
 * Chạy: npx playwright test -c articles/checklist-based-testing/playwright.config.ts
 */
import { expect, test, type APIRequestContext, type Browser, type Page } from '@playwright/test';

type Kind = 'auto' | 'judge';
const log: { id: string; kind: Kind; observed: string }[] = [];
const note = (id: string, kind: Kind, observed: string) => log.push({ id, kind, observed });

test.describe.configure({ mode: 'serial' });

test.beforeEach(async ({ request }) => {
  await request.post('/api/test/reset');
});

test.afterAll(() => {
  console.log('\nNhật ký phiên CL-SMOKE v1.0 (auto = khẳng định cứng đã đạt, judge = người đánh giá):');
  for (const entry of log.sort((a, b) => a.id.localeCompare(b.id))) {
    console.log(`${entry.id} [${entry.kind}] ${entry.observed}`);
  }
});

async function open(browser: Browser, path: string, timezoneId = 'Asia/Ho_Chi_Minh'): Promise<Page> {
  const page = await (await browser.newContext({ timezoneId })).newPage();
  await page.goto(path);
  return page;
}

async function quote(page: Page, fields: { kind?: string; vehicle?: string; km?: string; kg?: string; food?: string; voucher?: string; pay?: string }) {
  if (fields.kind) await page.getByLabel('Dịch vụ').selectOption(fields.kind);
  if (fields.vehicle) await page.getByLabel('Loại xe').selectOption(fields.vehicle);
  if (fields.km) await page.getByLabel('Quãng đường (km)').fill(fields.km);
  if (fields.kg) await page.getByLabel('Khối lượng bưu kiện (kg)').fill(fields.kg);
  if (fields.food) await page.getByLabel('Tiền món (đ)').fill(fields.food);
  if (fields.voucher !== undefined) await page.getByLabel('Mã giảm giá (không bắt buộc)').fill(fields.voucher);
  if (fields.pay) await page.getByLabel('Thanh toán').selectOption(fields.pay);
  await page.getByRole('button', { name: 'Báo giá' }).click();
}

async function book(page: Page): Promise<string> {
  await page.getByRole('button', { name: 'Đặt chuyến' }).click();
  await expect(page.getByTestId('trip-status')).toHaveText('Đang tìm tài xế');
  return (await page.getByTestId('trip-id').textContent()) ?? '';
}

async function driverAccept(driver: Page, tripId: string): Promise<void> {
  await driver.getByRole('button', { name: 'Tải chuyến đang chờ' }).click();
  await driver.getByTestId(`accept-${tripId}`).click();
  await expect(driver.getByTestId('driver-trip-status')).toHaveText('Tài xế đã nhận');
}

/** Đợi thông báo lỗi xuất hiện rồi đọc (lời gọi API là bất đồng bộ). */
async function alertText(page: Page): Promise<string> {
  await expect(page.getByRole('alert')).not.toBeEmpty();
  return (await page.getByRole('alert').textContent()) ?? '';
}

async function tripApi(request: APIRequestContext, tripId: string) {
  const res = await request.get(`/api/trips/${tripId}`, { headers: { 'x-user': 'customer:C001' } });
  return (await res.json()).trip as { status: string; paymentStatus: string; fare: number; acceptedAt: string };
}

test('A · Đặt xe (SM-01…05, SM-18, SM-19)', async ({ browser, request }) => {
  const customer = await open(browser, '/');
  const driver = await open(browser, '/driver.html');

  await quote(customer, { km: '7.4' });
  await expect(customer.getByTestId('quote-total')).toHaveText('46.000 đ');
  await expect(customer.getByTestId('quote-peak')).toHaveText('Không');
  await expect(customer.getByTestId('quote-rain')).toHaveText('Không');
  note('SM-01', 'auto', 'tổng 46.000 đ, cao điểm Không, mưa Không');

  const expiresVn = await customer.getByTestId('quote-expires').textContent();
  const tokyo = await open(browser, '/', 'Asia/Tokyo'); // khách dùng máy đặt múi giờ khác (UTC+9)
  await quote(tokyo, { km: '7.4' });
  await expect(tokyo.getByTestId('quote-total')).toHaveText('46.000 đ');
  const expiresTokyo = await tokyo.getByTestId('quote-expires').textContent();
  note('SM-18', 'judge', `hạn báo giá: máy giờ VN «${expiresVn}», máy giờ Tokyo «${expiresTokyo}» (không ghi múi giờ)`);

  await quote(customer, { voucher: ' giam20 ', pay: 'vixanh' });
  await expect(customer.getByTestId('quote-discount')).toHaveText('9.060 đ');
  await expect(customer.getByTestId('quote-total')).toHaveText('37.000 đ');
  note('SM-02', 'auto', 'mã « giam20 » được nhận, giảm 9.060 đ, tổng 37.000 đ');

  await customer.getByRole('button', { name: 'Đặt chuyến' }).dblclick();
  await expect(customer.getByTestId('trip-status')).toHaveText('Đang tìm tài xế');
  const tripId = (await customer.getByTestId('trip-id').textContent()) ?? '';
  await driver.getByRole('button', { name: 'Tải chuyến đang chờ' }).click();
  await expect(driver.getByTestId('available-trips').locator('li')).toHaveCount(1);
  const listed = await driver.getByTestId(`available-${tripId}`).textContent();
  note('SM-03', 'auto', `bấm đúp ⇒ tài xế thấy 1 chuyến: «${listed?.replace('Nhận chuyến', '').trim()}»`);
  note('SM-19', 'judge', `tiền ở danh sách tài xế: «${listed?.replace('Nhận chuyến', '').trim()}»`);

  await driver.getByTestId(`accept-${tripId}`).click();
  const steps: [string, string][] = [['action-arrive', 'Tài xế đang tới'], ['action-pickup', 'Đã đón / lấy hàng'], ['action-start', 'Đang chở / giao']];
  for (const [testId, label] of steps) {
    await driver.getByTestId(testId).click();
    await expect(driver.getByTestId('driver-trip-status')).toHaveText(label);
  }
  const t0 = Date.now();
  await driver.getByTestId('action-complete').click();
  await expect(customer.getByTestId('trip-status')).toHaveText('Hoàn thành', { timeout: 3_000 });
  const ms = Date.now() - t0;
  await expect(customer.getByTestId('cancel-button')).toBeDisabled();
  note('SM-04', 'auto', `trang khách tự hiện «Hoàn thành» sau ${ms} ms, nút huỷ khoá`);

  const trip = await tripApi(request, tripId);
  const panel = (await customer.getByTestId('trip-panel').innerText()).replace(/\s+/g, ' ');
  note('SM-05', 'judge', `API: paymentStatus=${trip.paymentStatus}, fare=${trip.fare}; trang khách hiện: «${panel}»`);
});

test('B · Bưu kiện (SM-06…08)', async ({ browser }) => {
  const customer = await open(browser, '/');
  await quote(customer, { kind: 'parcel', km: '7.4', kg: '6.1' });
  await expect(customer.getByTestId('quote-total')).toHaveText('50.000 đ');
  note('SM-06', 'auto', 'bưu kiện 6,1 kg xe máy 7,4 km ⇒ 50.000 đ');

  await customer.getByLabel('Khối lượng bưu kiện (kg)').fill('6.2');
  await expect(customer.getByRole('button', { name: 'Đặt chuyến' })).toBeDisabled();
  await expect(customer.getByTestId('quote-result')).toBeHidden();
  note('SM-08', 'auto', 'sửa khối lượng sau báo giá ⇒ nút Đặt chuyến khoá, báo giá cũ ẩn');

  await quote(customer, { kg: '25' });
  const message = await alertText(customer);
  await expect(customer.getByRole('button', { name: 'Đặt chuyến' })).toBeDisabled();
  note('SM-07', 'judge', `25 kg + xe máy ⇒ thông báo «${message}», nút Đặt chuyến khoá`);
});

test('C · Đồ ăn (SM-09…12)', async ({ browser }) => {
  const customer = await open(browser, '/');
  await quote(customer, { kind: 'food', km: '7.4', food: '60000', voucher: 'FOOD15' });
  await expect(customer.getByTestId('quote-discount')).toHaveText('9.000 đ');
  await expect(customer.getByTestId('quote-total')).toHaveText('97.000 đ');
  note('SM-09', 'auto', 'tiền món 60.000 đ + FOOD15 ⇒ giảm 9.000 đ, tổng 97.000 đ');
  await expect(customer.getByLabel('Loại xe')).toBeDisabled();
  note('SM-10', 'auto', 'chọn Giao đồ ăn ⇒ ô Loại xe bị khoá');

  await quote(customer, { food: '25000', voucher: '' });
  note('SM-11', 'judge', `tiền món 25.000 đ ⇒ thông báo «${await alertText(customer)}»`);

  await quote(customer, { kind: 'ride', voucher: 'FOOD15' });
  note('SM-12', 'judge', `FOOD15 cho chuyến đặt xe ⇒ thông báo «${await alertText(customer)}»`);
});

test('D · Huỷ có phí (SM-13…16, SM-17)', async ({ browser, request }) => {
  const customer = await open(browser, '/');
  const driver = await open(browser, '/driver.html');

  await quote(customer, { km: '7.4' });
  await book(customer);
  await customer.getByRole('button', { name: 'Huỷ chuyến' }).click();
  await expect(customer.getByTestId('cancel-fee')).toHaveText('Phí huỷ: 0 đ');
  note('SM-13', 'auto', 'huỷ khi Đang tìm tài xế ⇒ «Phí huỷ: 0 đ»');

  await quote(customer, { km: '7.4' });
  const tripId = await book(customer);
  await driverAccept(driver, tripId); // D101, ETA 5 phút
  const { acceptedAt } = await tripApi(request, tripId);
  await request.post('/api/test/clock', { data: { now: new Date(Date.parse(acceptedAt) + 3 * 60_000).toISOString() } });
  await customer.getByRole('button', { name: 'Huỷ chuyến' }).click();
  await expect(customer.getByTestId('cancel-fee')).toHaveText('Phí huỷ: 10.000 đ');
  await expect(customer.getByTestId('trip-status')).toHaveText('Đã huỷ');
  await expect(customer.getByTestId('cancel-button')).toBeDisabled();
  note('SM-14', 'auto', 'nhận + 3 phút ⇒ «Phí huỷ: 10.000 đ», trạng thái Đã huỷ, nút huỷ khoá');

  await driver.waitForTimeout(3_000); // cho trang tài xế một cơ hội tự cập nhật (nếu có)
  const driverStatus = await driver.getByTestId('driver-trip-status').textContent();
  await driver.getByTestId('action-arrive').click();
  const driverError = await alertText(driver);
  note('SM-16', 'judge', `3 giây sau khi khách huỷ, trang tài xế vẫn ghi «${driverStatus}»; bấm «Đang tới đón» ⇒ «${driverError}»`);
  note('SM-17', 'judge', `thông báo lỗi trang tài xế: «${driverError}»`);

  await quote(customer, { km: '7.4' });
  const next = await book(customer);
  await driverAccept(driver, next);
  note('SM-15', 'auto', 'sau khi khách huỷ, D101 nhận được chuyến mới (đã về online)');
});

test('E · Chỉ dùng bàn phím (SM-20)', async ({ browser }) => {
  const customer = await open(browser, '/');
  await customer.getByLabel('Quãng đường (km)').focus();
  await customer.keyboard.press('Enter'); // Enter trong ô nhập ⇒ gửi form «Báo giá»
  await expect(customer.getByTestId('quote-total')).toHaveText('46.000 đ');
  const order: string[] = [];
  for (let i = 0; i < 4; i++) {
    await customer.keyboard.press('Tab');
    order.push((await customer.locator(':focus').getAttribute('data-testid')) ?? '?');
  }
  await customer.keyboard.press('Enter');
  await expect(customer.getByTestId('trip-status')).toHaveText('Đang tìm tài xế');
  note('SM-20', 'judge', `Enter ở ô quãng đường ⇒ báo giá; Tab ×4: ${order.join(' → ')}; Enter ⇒ đặt được chuyến`);
});
