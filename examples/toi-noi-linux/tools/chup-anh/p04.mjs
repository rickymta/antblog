#!/usr/bin/env node
// Chụp ảnh SSH Studio cho Phần 4 (LNX-031…LNX-040).
//
//   node tools/chup-anh/p04.mjs          đưa server về đầu LNX-031 (setup 31) rồi chụp cả phần
//   node tools/chup-anh/p04.mjs 36 37    chỉ vài bài; server phải đang ở đầu bài đầu tiên
//
// SSH Studio phải đang mở với cổng debug (xem README.md), có server toinoi-app-01 dùng SSH key
// an-laptop (trạng thái cuối Phần 3). Từ Phần 3 server bật xác thực hai bước, nên mỗi lần kết
// nối công cụ đợi sang khung 30 giây mới rồi nhập mã OTP tính từ bí mật của lab.
// Chạy xong, `npm run kiem-tra:p04` phải cho toàn PASS.

import { spawnSync } from 'node:child_process';
import { join } from 'node:path';

import { ROOT, root } from '../../lib/lab.mjs';
import { open } from './app.mjs';
import { chayPhan, shellIdle } from './phan.mjs';

const SLUG = {
  31: 'linux-tien-trinh-ps-top',
  32: 'linux-tin-hieu-kill',
  33: 'linux-apt-cai-phan-mem',
  34: 'linux-job-nen-nohup-tmux',
  35: 'linux-systemd-systemctl',
  36: 'linux-systemd-unit-bao-gia',
  37: 'linux-journalctl-doc-log',
  38: 'linux-cron-systemd-timer',
  39: 'linux-logrotate',
  40: 'linux-thuc-hanh-dich-vu-systemd',
};
const SERVER = 'toinoi-app-01';
const ADDRESS = '[127.0.0.1]:2222';

const only = process.argv.slice(2);
if (!only.length) {
  console.log('Đưa server về đầu LNX-031…');
  const r = spawnSync('node', [join(ROOT, 'run.mjs'), 'setup', '31'], { stdio: 'inherit' });
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

await chayPhan({
  part: 'p04',
  slugs: SLUG,
  only,
  gui: ({ a, c, sleep, shot }) => {
    /** Đóng tab phiên hiện tại (SSH đứt, shell nhận SIGHUP) rồi kết nối lại bằng tab mới. */
    async function reconnect() {
      await closeAllTabs(c, sleep);
      await sleep(1000);
      await connect(a);
    }
    return {
      // ---------- LNX-031 ----------
      async top() {
        await a.clear();
        await a.run('top -o %CPU', 3500);
        await shot('03'); // top tương tác, tinh-lai-bao-cao đứng đầu với ~100% CPU
        await c.type('q');
        await sleep(600);
      },

      // ---------- LNX-034 ----------
      async 'ngat-ket-noi'() {
        await reconnect();
      },
      async 'tmux-attach'() {
        await a.clear();
        await a.run('tmux attach -t xuat', 2500);
        await shot('03'); // đang ở trong tmux, tiến độ xuất vẫn chạy, thanh trạng thái màu xanh
        root('tmux -S /tmp/tmux-1000/default detach-client -s xuat');
        await sleep(800);
      },

      // ---------- LNX-036 ----------
      async 'ket-noi-lai'() {
        // Nhóm systemd-journal thêm ở LNX-035 chỉ có hiệu lực ở phiên đăng nhập mới.
        await reconnect();
      },

      // ---------- LNX-037 ----------
      async 'theo-doi-journal'() {
        await a.clear();
        await a.run('journalctl -u bao-gia -f -n 3', 1500);
        root("curl -s -o /dev/null 'http://127.0.0.1:8080/bao-gia?dich_vu=xe-tai&km=2'");
        await sleep(800);
        root("curl -s -o /dev/null 'http://127.0.0.1:8080/bao-gia?dich_vu=car4&km=-1'");
        await sleep(1500);
        await shot('03'); // journalctl -f đang chạy, hai dòng WARN mới hiện ra
        root('pkill -INT -u an -x journalctl');
        await sleep(800);
        for (let t = 0; t < 40 && !shellIdle(); t++) await sleep(250);
      },
    };
  },
});
console.log('Xong. Kiểm tra trạng thái cuối: npm run kiem-tra:p04');
