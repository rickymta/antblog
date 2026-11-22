// Thao tác mức "người dùng" trên SSH Studio: kết nối, gõ lệnh, đổi chế độ xem, khung Files.

import { PASSWORD } from '../../lib/lab.mjs';
import { attach, sleep } from './cdp.mjs';

export const SERVER = 'toinoi-app-01';

// Khung file theo loại: 'local' (Máy của bạn) hoặc 'remote' (Remote server).
const PANE_FN = `const PANE = (kind) => [...document.querySelectorAll('.file-pane')].find((p) => p.innerText.includes(kind === 'local' ? 'Máy của bạn' : 'Remote server'));`;

export async function open() {
  const c = await attach();
  // Không giả lập tỉ lệ điểm ảnh: terminal vẽ bằng WebGL nên chữ sẽ sai cỡ. Cửa sổ phải được
  // mở sẵn ở 1600×900 điểm CSS với tỉ lệ thật 1,5 (xem README.md), ảnh chụp ra 2400×1350.
  await c.send('Emulation.clearDeviceMetricsOverride');
  await c.waitFor('button.nav-item', 'Servers', 30000);
  const size = await c.eval('({ w: innerWidth, h: innerHeight, dpr: devicePixelRatio })');
  if (size.w !== 1600 || size.h !== 900 || size.dpr < 1)
    throw new Error(`Cửa sổ SSH Studio phải là 1600×900 điểm CSS, đang là ${size.w}×${size.h} @${size.dpr}`);

  const app = {
    c,
    sleep,

    /** Đóng mọi hộp thoại, thông báo đang mở. */
    async dismiss() {
      for (let i = 0; i < 3; i++) {
        if (await c.exists('button', 'Đóng thông báo')) await c.click('button', 'Đóng thông báo');
        else break;
      }
    },

    async nav(name) {
      await c.click('button.nav-item', name);
      await sleep(400);
    },

    /** Chế độ xem của phiên: Terminal | Files | Chia đôi | Song song | Ports. */
    async view(name) {
      await c.click('button', name, { exact: true });
      await sleep(name === 'Terminal' ? 500 : 1200);
    },

    /** Điền ô nhập nằm trong <label> bắt đầu bằng `label`. */
    async fill(label, value) {
      const pos = await c.eval(`(() => {
        const l = [...document.querySelectorAll('label')].find((x) => x.innerText.trim().startsWith(${JSON.stringify(label)}));
        const i = l && l.querySelector('input, textarea, select');
        if (!i) return null;
        i.scrollIntoView({ block: 'nearest' });
        const r = i.getBoundingClientRect();
        return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
      })()`);
      if (!pos) throw new Error('Không có ô: ' + label);
      await c.clickAt(pos.x, pos.y);
      await c.ctrl('a');
      await c.type(String(value));
    },

    /** Gõ mật khẩu lab vào hộp thoại đăng nhập (nếu đang hiện) rồi tiếp tục. */
    async password({ remember = false } = {}) {
      await c.waitFor('input[type=password]');
      await c.click('input[type=password]');
      await c.type(PASSWORD);
      if (remember) await c.click('label', 'Lưu mật khẩu');
    },
    async submitPassword() {
      await c.click('.button.primary', 'Tiếp tục');
      await c.waitFor('.terminal.xterm');
      await sleep(2200);
    },

    // ---------- Terminal ----------

    async focusTerminal() {
      const p = await c.locate('.terminal.xterm');
      // Bấm vào vùng trống phía dưới bên phải để không chọn chữ.
      await c.clickAt(p.left + p.w - 60, p.top + p.h - 24);
    },

    /** Gõ một lệnh vào terminal rồi Enter. */
    async run(cmd, wait = 450) {
      await app.focusTerminal();
      await c.type(cmd);
      await c.enter();
      await sleep(wait);
    },

    /** Gõ nhưng chưa Enter (để chụp lệnh đang gõ). */
    async typeOnly(text) {
      await app.focusTerminal();
      await c.type(text);
      await sleep(200);
    },

    async clear() {
      await app.run('clear', 300);
    },

    /** Trả lời dấu nhắc "[sudo] password for an:" trong terminal. */
    async sudoPassword() {
      await sleep(500);
      await c.type(PASSWORD);
      await c.enter();
      await sleep(900);
    },

    // ---------- Khung Files ----------

    /** Đặt thư mục của khung 'local' hoặc 'remote'. */
    async path(pane, value) {
      const pos = await c.eval(`(() => { ${PANE_FN}
        const i = PANE(${JSON.stringify(pane)}).querySelector('.pathbar input');
        const r = i.getBoundingClientRect();
        return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
      })()`);
      await c.clickAt(pos.x, pos.y);
      await c.ctrl('a');
      await c.type(value);
      await c.enter();
      await sleep(1000);
    },

    /** Vị trí một dòng file theo tên trong khung local (0) hoặc remote (1). */
    row(pane, name) {
      return c.eval(`(() => { ${PANE_FN}
        const p = PANE(${JSON.stringify(pane)});
        const el = [...p.querySelectorAll('.file-name')].find((e) => e.innerText.trim() === ${JSON.stringify(name)});
        if (!el) return null;
        el.scrollIntoView({ block: 'nearest' });
        const r = el.getBoundingClientRect();
        return { x: r.left + 40, y: r.top + r.height / 2 };
      })()`);
    },
    async select(pane, name) {
      const p = await app.row(pane, name);
      if (!p) throw new Error(`Không có ${name} trong khung ${pane}`);
      await c.clickAt(p.x, p.y);
      await sleep(250);
    },
    async openRow(pane, name) {
      const p = await app.row(pane, name);
      if (!p) throw new Error(`Không có ${name} trong khung ${pane}`);
      await c.clickAt(p.x, p.y);
      await c.mouse('mousePressed', p.x, p.y, { clickCount: 2 });
      await c.mouse('mouseReleased', p.x, p.y, { clickCount: 2 });
      await sleep(1200);
    },
    /** Bấm nút trên thanh công cụ của một khung, tìm theo chữ hoặc title. */
    async paneButton(pane, label) {
      const pos = await c.eval(`(() => { ${PANE_FN}
        const p = PANE(${JSON.stringify(pane)});
        const b = [...p.querySelectorAll('button')].find((x) => ((x.innerText || '').trim() || x.title || x.getAttribute('aria-label') || '').startsWith(${JSON.stringify(label)}));
        if (!b) return null;
        const r = b.getBoundingClientRect();
        return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
      })()`);
      if (!pos) throw new Error('Không có nút ' + label);
      await c.clickAt(pos.x, pos.y);
      await sleep(500);
    },

    text: () => c.eval('document.body.innerText'),
    /** Liệt kê nút, ô nhập đang hiển thị: dùng khi dò giao diện. */
    dump: (root = 'body') =>
      c.eval(`[...document.querySelector(${JSON.stringify(root)}).querySelectorAll('button, input, select, textarea')]
        .filter((b) => b.getBoundingClientRect().width)
        .map((b) => [b.tagName, b.type || '', ((b.tagName === 'INPUT' && b.value) || b.innerText || b.title || b.getAttribute('aria-label') || b.placeholder || '?').trim().split(String.fromCharCode(10)).join('/').slice(0, 50), b.className].join('|'))
        .join(String.fromCharCode(10))`),
  };
  return app;
}
