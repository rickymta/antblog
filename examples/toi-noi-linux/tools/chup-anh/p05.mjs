#!/usr/bin/env node
// Chụp ảnh SSH Studio cho Phần 5 (LNX-041…LNX-050).
//
//   node tools/chup-anh/p05.mjs          đưa server về đầu LNX-041 (setup 41) rồi chụp cả phần
//   node tools/chup-anh/p05.mjs 46 47    chỉ vài bài; server phải đang ở đầu bài đầu tiên
//
// Như p04.mjs: SSH Studio có server toinoi-app-01 dùng key an-laptop, mỗi lần kết nối cần mã OTP.
// Các đoạn `#may app02` của kịch bản chạy ngầm (SSH Studio chỉ mở app01).
// Chạy xong, `npm run kiem-tra:p05` phải cho toàn PASS.

import { spawnSync } from 'node:child_process';
import { join } from 'node:path';

import { ROOT, root } from '../../lib/lab.mjs';
import { open } from './app.mjs';
import { chayPhan, shellIdle } from './phan.mjs';

const SLUG = {
  41: 'linux-ip-cong-ss',
  42: 'linux-dns-ping-curl',
  43: 'linux-df-du-dung-luong',
  44: 'linux-ssh-port-forwarding',
  45: 'linux-tuong-lua-ufw-nftables',
  46: 'linux-dong-bo-rsync-ssh-studio',
  47: 'linux-dia-phan-vung-mount',
  48: 'linux-bo-nho-tai-he-thong',
  49: 'linux-nginx-reverse-proxy-tls',
  50: 'linux-thuc-hanh-mang-giam-sat',
};
const SERVER = 'toinoi-app-01';
const ADDRESS = '[127.0.0.1]:2222';

const only = process.argv.slice(2);
if (!only.length) {
  console.log('Đưa server về đầu LNX-041…');
  const r = spawnSync('node', [join(ROOT, 'run.mjs'), 'setup', '41'], { stdio: 'inherit' });
  if (r.status !== 0) process.exit(r.status);
  // LNX-029 đặt RATE_LIMIT 3 30 cho mã OTP: tối đa 3 lần đăng nhập của an trong 30 giây. Setup vừa
  // chạy các lần đăng nhập thật của LNX-029/030, nên đợi hết cửa sổ đó rồi mới kết nối.
  console.log('Đợi 35 giây cho hết giới hạn đăng nhập của mã OTP…');
  await new Promise((ok) => setTimeout(ok, 35000));
}

/** Biểu thức JS chạy trong trang: toạ độ nút `sel` nằm trong dòng của server `name`. */
const rowButton = (name, sel) => `(() => {
  const NL = String.fromCharCode(10);
  for (const b of document.querySelectorAll(${JSON.stringify(sel)})) {
    let p = b.parentElement;
    while (p && p.querySelectorAll('button[aria-label="Thêm thao tác"]').length <= 1) {
      if ((p.innerText || '').split(NL).some((l) => l.trim() === ${JSON.stringify(name)})) {
        const x = b.getBoundingClientRect();
        return { x: x.left + x.width / 2, y: x.top + x.height / 2 };
      }
      p = p.parentElement;
    }
  }
  return null;
})()`;

const otp = () => root('oathtool --totp -b "$(head -n 1 /usr/share/toinoi-lab/p03/google_authenticator-an)"');
const nextOtpWindow = (sleep) => sleep((31 - (Math.floor(Date.now() / 1000) % 30)) * 1000);

/** Đóng mọi tab phiên (tải lại cửa sổ không đóng phiên: SSH Studio khôi phục chúng). */
async function closeAllTabs(c, sleep) {
  for (let i = 0; i < 20; i++) {
    const p = await c.eval(`(() => {
      const b = [...document.querySelectorAll('button[title="Đóng phiên"]')].filter((b) => b.getBoundingClientRect().width)[0];
      if (!b) return null;
      const r = b.getBoundingClientRect();
      return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
    })()`);
    if (!p) return;
    await c.clickAt(p.x, p.y);
    await sleep(600);
    if (await c.exists('.confirm-actions button', 'Ngắt kết nối')) {
      await c.click('.confirm-actions button', 'Ngắt kết nối');
      await sleep(600);
    }
  }
  throw new Error('Không đóng hết được các tab phiên');
}

/** Kết nối mới tới toinoi-app-01 (key + mã OTP) và chờ tới dấu nhắc. */
async function connect(a) {
  const { c, sleep } = a;
  await a.nav('Servers');
  await sleep(500);
  const row = await c.eval(rowButton(SERVER, 'button.connect'));
  if (!row) throw new Error('Không thấy nút Kết nối của ' + SERVER);
  await nextOtpWindow(sleep);
  await c.clickAt(row.x, row.y);
  for (let i = 0; i < 20; i++) {
    await sleep(500);
    if (await c.exists('.button.primary', 'Tin cậy và kết nối')) await c.click('.button.primary', 'Tin cậy và kết nối');
    if (await c.exists('.modal input[type=password]')) {
      await c.click('.modal input[type=password]');
      await c.type(otp());
      await c.click('.modal .button.primary', 'Tiếp tục');
      break;
    }
  }
  await c.waitFor('.terminal.xterm', null, 15000);
  await sleep(2500);
  const loi = await c.eval(`document.querySelector('main')?.innerText.match(/Lỗi kết nối[\\s\\S]{0,240}/)?.[0] || ''`);
  if (loi) throw new Error('Kết nối thất bại: ' + loi.replace(/\s+/g, ' '));
}

// ---------- Đầu lượt chụp: một tab duy nhất, phiên mới ----------
{
  const a = await open();
  const { c, sleep } = a;
  // Host key sinh lúc build ảnh lab: dựng lại ảnh là fingerprint đổi. Known Hosts còn giữ bản
  // cũ thì SSH Studio (đúng như nó phải làm) chặn kết nối, nên quên bản cũ trước khi chụp.
  const data = await c.eval(`window.__TAURI_INTERNALS__.invoke('snapshot')`);
  const fp = root('ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub').split(' ')[1];
  if (data.knownHosts[ADDRESS] && data.knownHosts[ADDRESS] !== fp) {
    console.log(`Quên host key cũ của ${ADDRESS}: ${data.knownHosts[ADDRESS]} (server giờ là ${fp})`);
    await c.eval(`window.__TAURI_INTERNALS__.invoke('forget_host', ${JSON.stringify({ address: ADDRESS })})`);
  }
  await c.eval('location.reload()');
  await sleep(3000);
  await closeAllTabs(c, sleep);
  await connect(a);
  c.close();
}

import { writeFileSync } from 'node:fs';
import { connect as tcpConnect } from 'node:net';

/** Kết quả gọi qua đường hầm từ chính máy Windows (đưa vào bài viết, không bịa). */
const KET_QUA = join(ROOT, 'p05', 'tu-may-ban.txt');

await chayPhan({
  part: 'p05',
  slugs: SLUG,
  only,
  gui: ({ a, c, sleep, shot, files, LOCAL }) => {
    let giuKetNoi = null;
    /** Chọn <option> theo chữ hiển thị trong <select> nằm trong label `label`. */
    async function chooseOption(label, text) {
      const ok = await c.eval(`(() => {
        const l = [...document.querySelectorAll('label')].find((x) => x.innerText.trim().startsWith(${JSON.stringify(label)}));
        const s = l && l.querySelector('select');
        if (!s) return false;
        const o = [...s.options].find((x) => x.text.includes(${JSON.stringify(text)}));
        if (!o) return false;
        Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set.call(s, o.value);
        s.dispatchEvent(new Event('change', { bubbles: true }));
        return true;
      })()`);
      if (!ok) throw new Error(`Không chọn được ${text} trong ${label}`);
      await sleep(300);
    }
    return {
      // ---------- LNX-044 ----------
      async 'forward-local'() {
        await a.view('Ports');
        await a.fill('Lắng nghe tại', '127.0.0.1');
        await a.fill('Cổng', '18080');
        await a.fill('Đích (từ server)', '127.0.0.1');
        await a.fill('Cổng đích', '8080');
        // Không lưu vào cấu hình server: đường hầm chỉ cho bài này.
        if (await c.eval(`[...document.querySelectorAll('label.check input')].some((i) => i.checked)`)) {
          await c.click('label.check', 'Lưu vào server');
        }
        await sleep(300);
        await shot('02'); // form Local 127.0.0.1:18080 → 127.0.0.1:8080
        await c.click('.ports-form .button.primary', 'Bắt đầu');
        await sleep(1500);
        // Từ máy Windows: gọi qua đường hầm, ghi lại kết quả thật cho bài viết.
        const r = spawnSync('curl.exe', ['-s', '-w', '\nHTTP %{http_code}', 'http://127.0.0.1:18080/suc-khoe'], { encoding: 'utf8' });
        writeFileSync(KET_QUA, `PS> curl.exe http://127.0.0.1:18080/suc-khoe\n${r.stdout.trim()}\n`);
        console.log('  từ máy bạn:', r.stdout.trim().split('\n')[0]);
        // Giữ một kết nối mở để phía server thấy nó trong `ss`.
        giuKetNoi = tcpConnect(18080, '127.0.0.1');
        await sleep(1500);
        await shot('03'); // bảng đường hầm: Đang chạy, Kết nối 1 / 2
        await a.view('Terminal');
      },
      async 'forward-xong'() {
        await shot('04'); // terminal: ss thấy sshd nối tới 127.0.0.1:8080
        giuKetNoi?.destroy();
        await a.view('Ports');
        await c.click('.ports-actions button[title="Dừng"]');
        await sleep(800);
        await chooseOption('Kiểu', 'Dynamic');
        await a.fill('Lắng nghe tại', '127.0.0.1');
        await a.fill('Cổng', '1080');
        await sleep(400);
        await shot('05'); // form Dynamic (SOCKS5) và đường hầm Local đã dừng
        await chooseOption('Kiểu', 'Local');
        await a.view('Terminal');
      },

      // ---------- LNX-046 ----------
      async 'dong-bo-len'() {
        await files('Files', '/home/an/bang-gia-moi');
        await a.path('local', join(LOCAL, 'bang-gia-moi'));
        await sleep(600);
        await c.click('button.text-button', 'Đồng bộ');
        await c.waitFor('.modal', 'Đồng bộ thư mục');
        await sleep(1500);
        await shot('01'); // so sánh: hai file chỉ có ở local, chiều Local → Remote
        await c.click('.modal .button.primary', 'Đồng bộ');
        await sleep(2500);
        if (await c.exists('.modal', 'Đồng bộ thư mục')) await c.click('.modal button', 'Hủy').catch(() => {});
        await sleep(500);
        await shot('02'); // khung remote đã có bang-gia.csv và ghi-chu.txt
        await a.view('Terminal');
      },
    };
  },
});
console.log('Xong. Kiểm tra trạng thái cuối: npm run kiem-tra:p05');
