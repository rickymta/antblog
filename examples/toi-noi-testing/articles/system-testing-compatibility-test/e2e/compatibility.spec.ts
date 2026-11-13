import { test, expect, type Page } from '@playwright/test';

test.beforeEach(async ({ request, browser }, info) => {
  expect((await request.post('/api/test/reset')).status()).toBe(200);
  expect((await request.post('/api/test/clock', { data: { now: '2026-11-04T09:00:00+07:00' } })).status()).toBe(200);
  const environment = { project: info.project.name, browser: browser.version(), use: info.project.use };
  console.log(JSON.stringify(environment));
  await info.attach('environment', { body: JSON.stringify(environment, null, 2), contentType: 'application/json' });
});

async function activate(page: Page, id: string, touch: boolean) {
  if (touch) await page.getByTestId(id).tap();
  else await page.getByTestId(id).click();
}
async function noHorizontalOverflow(page: Page) {
  const size = await page.evaluate('({content: document.documentElement.scrollWidth, viewport: document.documentElement.clientWidth})') as { content: number; viewport: number };
  expect(size.content).toBeLessThanOrEqual(size.viewport + 1);
}

test('CP-01 · báo giá, đặt, huỷ và bố cục ngang trên cấu hình được chọn', async ({ page }, info) => {
  const touch = !!info.project.use.hasTouch;
  await page.goto('/');
  await page.getByTestId('distance').fill('5');
  await noHorizontalOverflow(page);
  await activate(page, 'quote-button', touch);
  await expect(page.getByTestId('quote-total')).toHaveText('35.000 đ');
  await expect(page.getByTestId('quote-peak')).toHaveText('Không');
  await activate(page, 'book-button', touch);
  await expect(page.getByTestId('trip-status')).toHaveText('Đang tìm tài xế');
  await noHorizontalOverflow(page);
  await activate(page, 'cancel-button', touch);
  await expect(page.getByTestId('trip-status')).toHaveText('Đã huỷ');
  await expect(page.getByTestId('cancel-fee')).toHaveText('Phí huỷ: 0 đ');
  await expect(page.getByTestId('cancel-button')).toBeDisabled();
});

test('CP-02 · chọn food rồi quay lại ride, trường động và lỗi không mất chức năng', async ({ page }, info) => {
  const touch = !!info.project.use.hasTouch;
  await page.goto('/');
  await page.getByTestId('kind').selectOption('food');
  await expect(page.getByTestId('food-amount')).toBeVisible();
  await expect(page.getByTestId('vehicle')).toBeDisabled();
  await page.getByTestId('distance').fill('5');
  await activate(page, 'quote-button', touch);
  await expect(page.getByTestId('quote-total')).toHaveText('95.000 đ');
  await noHorizontalOverflow(page);
  await page.getByTestId('kind').selectOption('ride');
  await expect(page.getByTestId('food-amount')).toBeHidden();
  await expect(page.getByTestId('vehicle')).toBeEnabled();
  await expect(page.getByTestId('book-button')).toBeDisabled();
  await page.getByTestId('distance').fill('0.1');
  await activate(page, 'quote-button', touch);
  await expect(page.getByTestId('error')).toContainText('Quãng đường');
  await noHorizontalOverflow(page);
});
