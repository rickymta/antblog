/**
 * TEST-052 — Exploratory Testing: tái hiện tự động các phát hiện của phiên khám phá /driver.html (charter CH-01).
 * Phiên exploratory làm TAY; test này chỉ "đóng băng" phát hiện để PO/dev tái hiện được và để làm hồi quy sau khi sửa.
 * Mỗi test khẳng định HÀNH VI HIỆN TẠI (nên đang xanh); khi đội sửa, đổi kỳ vọng theo quyết định của PO.
 * Chạy: npx playwright test -c articles/exploratory-testing/playwright.config.ts
 */
import { expect, test, type APIRequestContext, type Page } from '@playwright/test';

async function bookBike(request: APIRequestContext, customer: string, distanceKm: number): Promise<string> {
  const q = await request.post('/api/quotes', {
    headers: { 'x-user': `customer:${customer}` },
    data: { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm },
  });
  const { quote } = await q.json();
  const t = await request.post('/api/trips', {
    headers: { 'x-user': `customer:${customer}`, 'idempotency-key': `es-${customer}-${distanceKm}-${Date.now()}` },
    data: { quoteId: quote.quoteId },
  });
  expect(t.status()).toBe(201);
  return (await t.json()).trip.id as string;
}

async function openDriver(page: Page, driverId: string): Promise<void> {
  await page.goto('/driver.html');
  await page.getByLabel('Mã tài xế').fill(driverId);
  await page.getByRole('button', { name: 'Tải chuyến đang chờ' }).click();
}

test.beforeEach(async ({ request }) => {
  await request.post('/api/test/reset');
});

test('ES-01 · tải lại trang khi đang giữ chuyến ⇒ mất khối «Chuyến đang chạy», chuyến không còn đường quay lại', async ({ page, request }) => {
  const tripId = await bookBike(request, 'C001', 7.4);
  await openDriver(page, 'D101');
  await page.getByTestId(`accept-${tripId}`).click();
  await expect(page.getByTestId('driver-trip-status')).toHaveText('Tài xế đã nhận');

  await page.reload();
  await page.getByRole('button', { name: 'Tải chuyến đang chờ' }).click();
  await expect(page.getByTestId('available-trips')).toHaveText('Chưa có chuyến nào.');
  await expect(page.getByTestId('current-trip')).toBeHidden();

  const res = await request.get(`/api/trips/${tripId}`, { headers: { 'x-user': 'driver:D101' } });
  const { trip } = await res.json();
  expect(trip).toMatchObject({ status: 'accepted', driverId: 'D101' }); // chuyến vẫn gán cho D101 — tài xế "busy" mà không thấy chuyến
});

test('ES-02 · danh sách cũ: chuyến đã bị hệ thống huỷ (hết 5 phút) vẫn có nút «Nhận chuyến», bấm ra thông báo mã nội bộ', async ({ page, request }) => {
  const tripId = await bookBike(request, 'C002', 2);
  await openDriver(page, 'D102');
  await expect(page.getByTestId(`accept-${tripId}`)).toBeVisible();

  await request.post('/api/test/clock', { data: { now: '2026-11-04T09:06:00+07:00' } }); // > 5 phút sau khi tạo
  const sweep = await (await request.post('/api/test/expire-searches')).json();
  expect(sweep.expired).toBe(1);

  await page.getByTestId(`accept-${tripId}`).click();
  await expect(page.getByTestId('error')).toHaveText('Không thể thực hiện "accept" khi chuyến đang ở trạng thái "cancelled".');
});

test('ES-03 · tài xế «Huỷ nhận» ⇒ danh sách không tự làm mới; tải lại thì chuyến đứng đầu và chính tài xế đó nhận lại được ngay', async ({ page, request }) => {
  const first = await bookBike(request, 'C001', 7.4);
  const second = await bookBike(request, 'C002', 2);
  await openDriver(page, 'D102');
  await page.getByTestId(`accept-${first}`).click();
  await page.getByTestId('action-arrive').click();
  await expect(page.getByTestId('driver-trip-status')).toHaveText('Tài xế đang tới');

  await page.getByTestId('action-cancel').click();
  await expect(page.getByTestId('current-trip')).toBeHidden();
  await expect(page.getByTestId(`available-${first}`)).toHaveCount(0); // chưa làm mới

  await page.getByRole('button', { name: 'Tải chuyến đang chờ' }).click();
  const items = page.getByTestId('available-trips').locator('li');
  await expect(items.first()).toHaveAttribute('data-testid', `available-${first}`); // xếp theo requestedAt — đứng trước `second`
  await expect(page.getByTestId(`available-${second}`)).toBeVisible();

  await page.getByTestId(`accept-${first}`).click(); // nhận lại ngay, không hỏi gì
  await expect(page.getByTestId('driver-trip-status')).toHaveText('Tài xế đã nhận');
});

test('ES-04 · đổi «Mã tài xế» giữa chuyến ⇒ bấm bước kế báo «Không tìm thấy chuyến.» cho chính chuyến đang hiện', async ({ page, request }) => {
  const tripId = await bookBike(request, 'C001', 7.4);
  await openDriver(page, 'D102');
  await page.getByTestId(`accept-${tripId}`).click();
  await expect(page.getByTestId('current-trip-id')).toHaveText(tripId);

  await page.getByLabel('Mã tài xế').fill('D101');
  await page.getByTestId('action-arrive').click();
  await expect(page.getByTestId('error')).toHaveText('Không tìm thấy chuyến.');
  await expect(page.getByTestId('current-trip-id')).toHaveText(tripId);
});

test('ES-06 · bấm đúp «Nhận chuyến» ⇒ hai lần tải danh sách chạy chồng nhau, một chuyến hiện HAI dòng', async ({ page, request }) => {
  const first = await bookBike(request, 'C001', 7.4);
  const other = await bookBike(request, 'C002', 2);
  await openDriver(page, 'D101');
  await page.getByTestId(`accept-${first}`).dblclick();
  await expect(page.getByTestId('driver-trip-status')).toHaveText('Tài xế đã nhận');
  await expect(page.getByTestId(`available-${other}`)).toHaveCount(2); // cùng data-testid xuất hiện hai lần
});

test('ES-05 · ETA 0 hoặc để trống ⇒ thông báo chung, không nói ô nào sai dù ô có min=1 max=60', async ({ page, request }) => {
  const tripId = await bookBike(request, 'C001', 7.4);
  await openDriver(page, 'D101');
  for (const eta of ['0', '']) {
    await page.getByLabel('ETA tới điểm đón (phút)').fill(eta);
    await page.getByTestId(`accept-${tripId}`).click();
    await expect(page.getByTestId('error')).toHaveText('Dữ liệu gửi lên không hợp lệ.');
    await expect(page.getByTestId('current-trip')).toBeHidden();
  }
  await expect(page.getByLabel('ETA tới điểm đón (phút)')).toHaveAttribute('min', '1');
});
