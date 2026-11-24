// Client CDP tối thiểu (Node 22, không dependency) để điều khiển cửa sổ WebView2 của SSH Studio
// và chụp ảnh. SSH Studio phải được chạy với cổng debug, xem README.md cùng thư mục.

import { writeFileSync } from 'node:fs';

const PORT = process.env.CDP_PORT || 9344;
export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

export async function attach() {
  const list = await (await fetch(`http://127.0.0.1:${PORT}/json/list`)).json();
  const page = list.find((t) => t.type === 'page');
  if (!page) throw new Error('Không thấy cửa sổ SSH Studio ở cổng debug ' + PORT);
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((ok, fail) => {
    ws.onopen = ok;
    ws.onerror = fail;
  });
  let seq = 0;
  const pending = new Map();
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (!msg.id || !pending.has(msg.id)) return;
    const { ok, fail } = pending.get(msg.id);
    pending.delete(msg.id);
    msg.error ? fail(new Error(JSON.stringify(msg.error))) : ok(msg.result);
  };
  const send = (method, params = {}) =>
    new Promise((ok, fail) => {
      const id = ++seq;
      pending.set(id, { ok, fail });
      ws.send(JSON.stringify({ id, method, params }));
    });

  const c = {
    send,
    close: () => ws.close(),

    async eval(expression) {
      const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
      if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || r.exceptionDetails.text);
      return r.result.value;
    },

    /** Khung nhìn 1600×900 điểm CSS, mỗi điểm `scale` pixel: scale 2 cho ảnh 3200×1800. */
    async viewport(scale = 2, width = 1600, height = 900) {
      await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: scale, mobile: false });
      await sleep(600);
    },

    async png(file) {
      const r = await send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
      writeFileSync(file, Buffer.from(r.data, 'base64'));
      return file;
    },

    /**
     * Tâm của phần tử đầu tiên khớp selector (và chứa `text` nếu có). Khi lọc theo chữ, lấy phần
     * tử trong cùng. `exact` đòi chữ khớp nguyên văn, hoặc khớp dòng đầu (nút có số đếm như «Ports
1»).
     */
    async locate(sel, text = null, { exact = false, nth = 0 } = {}) {
      const pos = await c.eval(`(() => {
        const sel = ${JSON.stringify(sel)}, text = ${JSON.stringify(text)}, exact = ${exact};
        const label = (e) => (e.innerText || e.textContent || e.title || e.getAttribute('aria-label') || '').trim();
        let els = [...document.querySelectorAll(sel)].filter((e) => {
          const r = e.getBoundingClientRect();
          if (!r.width || !r.height) return false;
          if (text === null) return true;
          const l = label(e), t = e.title || e.getAttribute('aria-label') || '';
          return exact ? l === text || t === text || l.split(/\\s*\\n/)[0] === text : l.includes(text) || t.includes(text);
        });
        if (text !== null) els = els.filter((e) => !els.some((o) => o !== e && e.contains(o)));
        const el = els[${nth}];
        if (!el) return null;
        el.scrollIntoView({ block: 'nearest', inline: 'nearest' });
        const r = el.getBoundingClientRect();
        return { x: r.left + r.width / 2, y: r.top + r.height / 2, left: r.left, top: r.top, w: r.width, h: r.height };
      })()`);
      if (!pos) throw new Error(`Không tìm thấy: ${sel} ${text ?? ''}`);
      return pos;
    },

    exists: (sel, text = null) =>
      c.locate(sel, text).then(
        () => true,
        () => false,
      ),

    async waitFor(sel, text = null, timeout = 8000) {
      const t0 = Date.now();
      while (Date.now() - t0 < timeout) {
        if (await c.exists(sel, text)) return;
        await sleep(150);
      }
      throw new Error(`Hết thời gian chờ: ${sel} ${text ?? ''}`);
    },

    async mouse(type, x, y, extra = {}) {
      await send('Input.dispatchMouseEvent', { type, x, y, button: 'left', clickCount: 1, ...extra });
    },
    async move(x, y) {
      await c.mouse('mouseMoved', x, y, { button: 'none' });
    },
    async clickAt(x, y, extra = {}) {
      await c.move(x, y);
      await c.mouse('mousePressed', x, y, extra);
      await c.mouse('mouseReleased', x, y, extra);
      await sleep(150);
    },
    async click(sel, text, opts) {
      const p = await c.locate(sel, text, opts);
      await c.clickAt(p.x, p.y);
      return p;
    },
    async dblclick(sel, text, opts) {
      const p = await c.locate(sel, text, opts);
      await c.clickAt(p.x, p.y);
      await c.mouse('mousePressed', p.x, p.y, { clickCount: 2 });
      await c.mouse('mouseReleased', p.x, p.y, { clickCount: 2 });
      await sleep(200);
    },
    async rightClick(sel, text, opts) {
      const p = await c.locate(sel, text, opts);
      await c.clickAt(p.x, p.y, { button: 'right' });
    },
    async hover(sel, text, opts) {
      const p = await c.locate(sel, text, opts);
      await c.move(p.x, p.y);
      await sleep(150);
    },

    /** Chèn chữ vào phần tử đang focus (dùng được cho ô nhập và cho terminal xterm). */
    async type(text) {
      await send('Input.insertText', { text });
      await sleep(80);
    },
    async key(key, { code, keyCode, modifiers = 0, text } = {}) {
      const base = { key, code: code || key, windowsVirtualKeyCode: keyCode, nativeVirtualKeyCode: keyCode, modifiers };
      await send('Input.dispatchKeyEvent', { type: text ? 'keyDown' : 'rawKeyDown', ...base, text });
      await send('Input.dispatchKeyEvent', { type: 'keyUp', ...base });
      await sleep(80);
    },
    enter: () => c.key('Enter', { keyCode: 13, text: '\r' }),
    esc: () => c.key('Escape', { keyCode: 27 }),
    /** Ctrl+<chữ>; thêm shift: true cho Ctrl+Shift+<chữ>. */
    ctrl: (letter, { shift = false } = {}) =>
      c.key(shift ? letter.toUpperCase() : letter.toLowerCase(), {
        code: 'Key' + letter.toUpperCase(),
        keyCode: letter.toUpperCase().charCodeAt(0),
        modifiers: 2 | (shift ? 8 : 0),
      }),
  };
  return c;
}
