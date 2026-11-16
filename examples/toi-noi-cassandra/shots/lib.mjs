// Thư viện chụp ảnh DBStudio cho series Cassandra.
//
// Giao diện DBStudio (frontend React) chạy bằng Vite và nói chuyện với backend THẬT của DBStudio qua
// "dev bridge" (bản debug build với feature dev-bridge). Playwright điều khiển Microsoft Edge headless.
// Xem shots/README.md để dựng môi trường.
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { chromium } from 'playwright-core';

export const UI_URL = process.env.DBSTUDIO_UI_URL ?? 'http://localhost:1440/';
export const BRIDGE = process.env.DBSTUDIO_BRIDGE_URL ?? 'http://127.0.0.1:1441';
const ALLOWED_ORIGIN = 'http://localhost:1420'; // origin mà bridge chấp nhận

export const CONN = 'toinoi-lab';

export async function launch() {
  return chromium.launch({ channel: 'msedge', headless: true, args: ['--disable-web-security'] });
}

/** Trang mới, sạch: đóng mọi tab cũ, ẩn nhãn "bridge" (chỉ có ở bản dev, người dùng không thấy). */
export async function newPage(browser, { width = 1280, height = 760 } = {}) {
  const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 2, locale: 'vi-VN', colorScheme: 'light' });
  await page.route(`${BRIDGE}/**`, (r) => r.continue({ headers: { ...r.request().headers(), origin: ALLOWED_ORIGIN } }));
  await page.goto(UI_URL);
  await page.getByText('KẾT NỐI').first().waitFor();
  await page.waitForTimeout(800);
  await closeAllTabs(page);
  await page.addStyleTag({ content: '.react-flow__minimap{display:none!important}' });
  await page.evaluate(() => {
    for (const el of document.querySelectorAll('span,div')) {
      if (el.childElementCount === 0 && el.textContent?.trim() === 'bridge') el.style.display = 'none';
    }
  });
  return page;
}

export async function closeAllTabs(page) {
  for (let i = 0; i < 20; i++) {
    const btn = page.getByRole('button', { name: 'Đóng tab' }).first();
    if (!(await btn.count())) break;
    await btn.click({ force: true }).catch(() => {});
    // tab query có nội dung có thể hỏi lại trước khi đóng
    const discard = page.getByRole('button', { name: /Không lưu|Đóng không lưu|Bỏ/ }).first();
    if (await discard.count()) await discard.click().catch(() => {});
    await page.waitForTimeout(150);
  }
}

/** Tạo kết nối toinoi-lab nếu chưa có. Trả về locator hộp thoại khi `keepDialog`. */
export async function fillConnectionDialog(page) {
  await page.getByRole('button', { name: 'Kết nối mới' }).first().click();
  const d = page.locator('[role=dialog]');
  await d.locator('input[id$="-uri"]').fill('cassandra://127.0.0.1:19042/toinoi');
  await d.getByRole('button', { name: 'Điền từ URI' }).click();
  await d.locator('input[id$="-name"]').fill(CONN);
  await d.locator('button[id$="-env"]').click();
  await page.getByRole('option', { name: 'Phát triển' }).click();
  await d.locator('button[id$="-authmode"]').click();
  await page.getByRole('option', { name: 'Không cần xác thực' }).click();
  await d.locator('button[id$="-tls"]').click();
  await page.getByRole('option', { name: 'Tắt' }).click();
  return d;
}

export async function ensureConnection(page) {
  if (await page.getByRole('tree').getByText(CONN, { exact: true }).count()) return;
  const d = await fillConnectionDialog(page);
  await d.getByRole('button', { name: 'Lưu' }).click();
  await label(page, CONN).waitFor();
}

/** Nhãn chữ của một nút trong cây đối tượng (khớp chính xác, không dính nút cha). */
export const label = (page, text) => page.getByRole('tree').getByText(text, { exact: true }).first();
const itemOf = (lbl) => lbl.locator('xpath=ancestor::*[@role="treeitem"][1]');

export async function openConnection(page) {
  await ensureConnection(page);
  const ks = page.getByRole('tree').getByText('toinoi', { exact: true });
  if (!(await ks.count())) {
    await label(page, CONN).dblclick();
    await ks.first().waitFor({ timeout: 20_000 });
  }
}

async function expand(page, lbl) {
  if ((await itemOf(lbl).getAttribute('aria-expanded')) !== 'true') {
    await lbl.click();
    await page.keyboard.press('ArrowRight');
    await page.waitForTimeout(800);
  }
}

export async function expandTables(page) {
  await openConnection(page);
  await expand(page, label(page, 'toinoi'));
  await expand(page, label(page, 'Bảng'));
}

export async function tableMenu(page, table, action) {
  await expandTables(page);
  await label(page, table).click({ button: 'right' });
  await page.getByRole('menuitem', { name: action }).click();
  await page.waitForTimeout(1500);
}

export async function keyspaceMenu(page, action) {
  await openConnection(page);
  await label(page, 'toinoi').click({ button: 'right' });
  await page.getByRole('menuitem', { name: action }).click();
  await page.waitForTimeout(2500);
}

/** Mở tab query mới gắn với keyspace toinoi, gõ CQL và chạy. `all` = chạy cả script. */
export async function runCql(page, cql, { all = false, wait = 2000 } = {}) {
  await openConnection(page);
  await label(page, 'toinoi').click({ button: 'right' });
  await page.getByRole('menuitem', { name: 'Mở trình soạn thảo' }).click();
  const ed = page.locator('.cm-content').last();
  await ed.waitFor();
  await ed.click();
  await page.keyboard.press('Control+a');
  await page.keyboard.press('Delete');
  await page.keyboard.insertText(cql);
  if (all) {
    await page.keyboard.press('Control+Shift+Enter');
  } else {
    // đặt con trỏ vào câu đầu tiên để "Chạy câu lệnh" chạy đúng câu đó
    await page.keyboard.press('Control+Home');
    await page.keyboard.press('Control+Enter');
  }
  await page.waitForTimeout(wait);
}

/** Chụp và lưu WebP (Edge tự mã hoá qua canvas). */
export async function shot(page, file, { clip, locator } = {}) {
  await page.mouse.move(0, 0);
  await page.waitForTimeout(300);
  const png = locator ? await locator.screenshot() : await page.screenshot({ clip });
  const dataUrl = await page.evaluate(async (b64) => {
    const img = new Image();
    img.src = `data:image/png;base64,${b64}`;
    await img.decode();
    const c = document.createElement('canvas');
    c.width = img.naturalWidth;
    c.height = img.naturalHeight;
    c.getContext('2d').drawImage(img, 0, 0);
    return c.toDataURL('image/webp', 0.86);
  }, png.toString('base64'));
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, Buffer.from(dataUrl.split(',')[1], 'base64'));
  return file;
}
