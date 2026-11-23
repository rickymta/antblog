#!/usr/bin/env node
// Chụp ảnh SSH Studio cho Phần 3 (LNX-021…LNX-030).
//
//   node tools/chup-anh/p03.mjs          đưa server về đầu LNX-021 (setup 21) rồi chụp cả phần
//   node tools/chup-anh/p03.mjs 25 26    chỉ vài bài; server phải đang ở đầu bài đầu tiên
//
// SSH Studio phải đang mở với cổng debug (xem README.md) và đã có server toinoi-app-01. Trước khi
// chụp, công cụ đưa SSH Studio về trạng thái đầu Phần 3: server dùng mật khẩu, không có key nào,
// không có hồ sơ của cuong, Known Hosts chỉ còn host key gốc. Chạy xong, `npm run kiem-tra:p03`
// phải cho 22 PASS.

import { spawn, spawnSync } from 'node:child_process';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

import { ROOT, root } from '../../lib/lab.mjs';
import { LOCAL, chayPhan } from './phan.mjs';
import { open } from './app.mjs';

const SLUG = {
  21: 'linux-quan-ly-nguoi-dung-nhom',
  22: 'linux-ssh-key-ssh-studio',
  23: 'linux-host-key-known-hosts',
  24: 'linux-sudo-sudoers',
  25: 'linux-sshd-config-an-toan',
  26: 'linux-setuid-setgid-sticky',
  27: 'linux-acl-setfacl',
  28: 'linux-ssh-agent-passphrase',
  29: 'linux-ssh-mfa-totp',
  30: 'linux-thuc-hanh-khoa-dang-nhap-ssh',
};
const ADDRESS = '[127.0.0.1]:2222';
const SERVER = 'toinoi-app-01';
const SERVER_CUONG = 'toinoi-app-01 (cuong)';
const KEY_FILE = join(LOCAL, 'khoa-lab', 'an-laptop');
const TOOLS = dirname(fileURLToPath(import.meta.url));

const only = process.argv.slice(2);
if (!only.length) {
  console.log('Đưa server về đầu LNX-021…');
  const r = spawnSync('node', [join(ROOT, 'run.mjs'), 'setup', '21'], { stdio: 'inherit' });
  if (r.status !== 0) process.exit(r.status);
}

/** Gọi thẳng một lệnh backend của SSH Studio (dùng để dọn trạng thái trước khi chụp). */
const invoke = (c, cmd, args = {}) =>
  c.eval(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args)})`);

/** Mã OTP hiện tại của an, tính trên server từ bí mật của lab. */

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

/**
 * Đóng mọi tab phiên. Tải lại cửa sổ KHÔNG đóng phiên: SSH Studio khôi phục các phiên cũ, và một
 * tab sót lại (ví dụ của cuong) sẽ tự kết nối lại và giành focus giữa lúc chụp.
 */
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

const otp = () => root('oathtool --totp -b "$(head -n 1 /usr/share/toinoi-lab/p03/google_authenticator-an)"');

// ---------- Đưa SSH Studio về trạng thái đầu Phần 3 (chỉ khi chụp từ LNX-021) ----------
const fromStart = !only.length || only[0] === '21';
if (fromStart || only[0] === '22') {
  const a = await open();
  const { c, sleep } = a;
  const data = await invoke(c, 'snapshot');
  if (fromStart) for (const h of data.hosts.filter((h) => h.name === SERVER_CUONG)) await invoke(c, 'delete_host', { id: h.id });
  const host = data.hosts.find((h) => h.name === SERVER);
  if (host.auth !== 'password') await invoke(c, 'save_host', { host: { ...host, auth: 'password', keyId: '' } });
  // Key đang được server dùng thì không xoá được: đổi server về mật khẩu trước.
  for (const k of data.keys) await invoke(c, 'delete_key', { id: k.id });
  const fpGoc = root('ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub').split(' ')[1];
  if (data.knownHosts[ADDRESS] && data.knownHosts[ADDRESS] !== fpGoc) await invoke(c, 'forget_host', { address: ADDRESS });
  await c.eval('location.reload()');
  await sleep(3000);
  await closeAllTabs(c, sleep);
  c.close();
}

// Kết nối (lại) phiên SSH bằng mật khẩu: server vừa được tạo lại.
if (fromStart || only[0] === '22') {
  const a = await open();
  const { c, sleep } = a;
  await a.nav('Servers');
  await sleep(500);
  const row = await c.eval(rowButton(SERVER, 'button.connect'));
  if (row) await c.clickAt(row.x, row.y);
  await sleep(1500);
  if (await c.exists('.button.primary', 'Tin cậy và kết nối')) await c.click('.button.primary', 'Tin cậy và kết nối');
  await a.password();
  await a.submitPassword();
  c.close();
} else {
  // Chụp tiếp từ giữa phần: `node run.mjs setup NN` vừa tạo lại server, mọi phiên cũ đã chết.
  // Tải lại cửa sổ (đóng hết tab) rồi kết nối bằng phương thức đang lưu của server (key từ LNX-022).
  const a = await open();
  const { c, sleep } = a;
  await c.eval('location.reload()');
  await sleep(3000);
  await closeAllTabs(c, sleep);
  await a.nav('Servers');
  await sleep(500);
  const row = await c.eval(rowButton(SERVER, 'button.connect'));
  if (!row) throw new Error('Không thấy nút Kết nối của ' + SERVER);
  // Từ LNX-030, server đã bật MFA (LNX-029): đợi sang khung 30 giây mới để mã OTP chưa bị dùng.
  if (Number(only[0]) >= 30) await sleep((31 - (Math.floor(Date.now() / 1000) % 30)) * 1000);
  await c.clickAt(row.x, row.y);
  await sleep(2500);
  if (await c.exists('.button.primary', 'Tin cậy và kết nối')) await c.click('.button.primary', 'Tin cậy và kết nối');
  await sleep(1500);
  if (await c.exists('.modal input[type=password]')) {
    await c.click('.modal input[type=password]');
    await c.type(otp());
    await c.click('.modal .button.primary', 'Tiếp tục');
    await sleep(2500);
  }
  c.close();
}

await chayPhan({
  part: 'p03',
  slugs: SLUG,
  only,
  gui: ({ a, c, sleep, shot }) => {
    /** Nút "Kết nối" ở dòng của một server trên trang Servers. */
    async function connectRow(name) {
      await a.nav('Servers');
      await sleep(400);
      const p = await c.eval(rowButton(name, 'button.connect'));
      if (!p) throw new Error('Không thấy server ' + name);
      await c.clickAt(p.x, p.y);
      await sleep(1500);
    }
    /** Mở hộp thoại sửa server từ trang Servers. */
    async function editRow(name) {
      await a.nav('Servers');
      await sleep(400);
      const p = await c.eval(rowButton(name, 'button[title="Sửa server"]'));
      if (!p) throw new Error('Không thấy nút sửa của ' + name);
      await c.clickAt(p.x, p.y);
      await sleep(600);
    }
    /** Chọn một <option> theo chữ hiển thị trong <select> nằm trong label `label`. */
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
    /** Mở thêm một tab phiên tới server đang mở. */
    /** Mở thêm một phiên tới toinoi-app-01: menu ⋯ của dòng server → "Kết nối phiên mới". */
    async function newTab(name = SERVER) {
      await a.nav('Servers');
      await sleep(400);
      const p = await c.eval(rowButton(name, 'button[aria-label="Thêm thao tác"]'));
      if (!p) throw new Error('Không thấy menu thao tác của ' + name);
      await c.clickAt(p.x, p.y);
      await sleep(400);
      const label = (await c.exists('[role=menuitem]', 'Kết nối phiên mới')) ? 'Kết nối phiên mới' : 'Kết nối';
      await c.click('[role=menuitem]', label, { exact: true });
      await sleep(1800);
    }
    /** Trả lời hộp thoại mật khẩu hoặc OTP nếu có. */
    async function answerPrompts() {
      for (let i = 0; i < 3; i++) {
        await sleep(900);
        const title = await c.eval(`document.querySelector('.modal h2')?.innerText || ''`);
        if (!title) return;
        if (/Xác nhận danh tính/.test(title)) await c.click('.button.primary', 'Tin cậy và kết nối');
        else if (await c.exists('.modal input[type=password]')) {
          const label = await c.eval(`document.querySelector('.modal label')?.innerText || ''`);
          await c.click('.modal input[type=password]');
          await c.type(/code|mã|verification/i.test(label + title) ? otp() : 'toinoi-lab');
          await c.click('.modal .button.primary', 'Tiếp tục');
        } else return;
      }
    }
    /** Đóng tab phiên cuối cùng (tab vừa mở), xác nhận nếu app hỏi. */
    async function closeTab() {
      const p = await c.eval(`(() => {
        const bs = [...document.querySelectorAll('button')].filter((b) => b.title === 'Đóng phiên' && b.getBoundingClientRect().width);
        const b = bs[bs.length - 1];
        if (!b) return null;
        const r = b.getBoundingClientRect();
        return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
      })()`);
      if (!p) return;
      await c.clickAt(p.x, p.y);
      await sleep(600);
      if (await c.exists('.confirm-actions button', 'Ngắt kết nối')) {
        await c.click('.confirm-actions button', 'Ngắt kết nối');
        await sleep(500);
      }
    }
    /** Chuyển sang phiên của `user` (cột trái ghi user@127.0.0.1); lấy phiên đầu tiên nếu có nhiều. */
    async function goTab(user = 'an') {
      await c.click('button.session-nav', `${user}@127.0.0.1`);
      await sleep(600);
    }

    return {
      // ---------- LNX-021 ----------
      async 'dang-nhap-cuong'() {
        await a.nav('Servers');
        await c.click('button.primary', 'Thêm server', { exact: true });
        await sleep(500);
        await a.fill('Tên hiển thị', SERVER_CUONG);
        await a.fill('Địa chỉ server', '127.0.0.1');
        await a.fill('Port', '2222');
        await a.fill('Username', 'cuong');
        await a.fill('Nhóm', 'Lab');
        await c.click('form .button.primary', 'Lưu server');
        await sleep(800);
        await a.dismiss();
        await connectRow(SERVER_CUONG);
        await c.waitFor('input[type=password]');
        await c.click('input[type=password]');
        await c.type('Tam-thoi-2611');
        await c.click('.modal .button.primary', 'Tiếp tục');
        await sleep(2500);
        await a.focusTerminal();
        await c.type('Tam-thoi-2611');
        await c.enter();
        await sleep(800);
        await c.type('Cuong-doi-mat-khau-2026');
        await c.enter();
        await sleep(800);
        await c.type('Cuong-doi-mat-khau-2026');
        await c.enter();
        await sleep(2500);
        await shot('02'); // đổi mật khẩu bắt buộc ở lần đăng nhập đầu
        await closeTab();
        await goTab('an');
      },

      // ---------- LNX-022 ----------
      async 'tao-va-nhap-key'() {
        await a.nav('SSH Keys');
        await c.click('button', 'Nhập key', { exact: true });
        await sleep(500);
        await a.fill('Tên key', 'an-laptop');
        // Hộp thoại chọn file của Windows: gõ đường dẫn key của lab vào ô tên file rồi Enter.
        const chon = spawn('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', join(TOOLS, 'chon-file.ps1'), '-Path', KEY_FILE]);
        await c.click('.modal button', 'Chọn file private key');
        await new Promise((ok) => chon.on('exit', ok));
        await sleep(800);
        await shot('01'); // hộp thoại Nhập SSH key với đường dẫn
        await c.click('.modal .button.primary', 'Nhập key');
        await sleep(1200);
        await a.dismiss();
        await c.click('button', 'Public key', { exact: true });
        await sleep(500);
        await shot('02'); // thư viện key với fingerprint, thông báo đã sao chép public key
        await a.dismiss();
        await goTab('an');
      },
      async 'chuyen-sang-key'() {
        await editRow(SERVER);
        await c.click('.modal button', 'SSH Key', { exact: true });
        await chooseOption('SSH key', 'an-laptop');
        await sleep(300);
        await shot('04'); // hộp thoại sửa server: phương thức SSH Key, chọn an-laptop
        await c.click('form .button.primary', 'Lưu');
        await sleep(800);
        await a.dismiss();
        await goTab('an');
        await newTab();
        await answerPrompts();
        await sleep(1500);
        await shot('05'); // phiên mới đăng nhập bằng key, không hỏi mật khẩu
      },

      // ---------- LNX-023 ----------
      async 'host-key-doi'() {
        await newTab();
        await sleep(1500);
        await shot('03'); // kết nối bị chặn vì host key đổi
        await closeTab();
        await a.nav('Known Hosts');
        await shot('04');
        await c.click('button', 'Quên host', { exact: true });
        await sleep(500);
        await c.click('.confirm-actions button', 'Quên host');
        await sleep(600);
        await goTab('an');
        await newTab();
        await c.waitFor('.modal', 'Xác nhận danh tính server');
        await sleep(400);
        await shot('05'); // fingerprint mới
        await c.click('.button.primary', 'Tin cậy và kết nối');
        await answerPrompts();
        await sleep(1200);
        await closeTab();
      },
      async 'host-key-tra-lai'() {
        await newTab();
        await sleep(1200);
        await closeTab();
        await a.nav('Known Hosts');
        await c.click('button', 'Quên host', { exact: true });
        await sleep(500);
        await c.click('.confirm-actions button', 'Quên host');
        await sleep(600);
        await goTab('an');
        await newTab();
        await c.waitFor('.modal', 'Xác nhận danh tính server');
        await c.click('.button.primary', 'Tin cậy và kết nối');
        await answerPrompts();
        await sleep(1200);
        await closeTab();
        await goTab('an');
      },

      // ---------- LNX-025 ----------
      async 'thu-phien-thu-hai'() {
        await newTab();
        await answerPrompts();
        await sleep(1200);
        await a.run('whoami; id -nG');
        await shot('04'); // phiên thứ hai vẫn vào được bằng key
        await closeTab();
        await connectRow(SERVER_CUONG);
        await sleep(2500);
        // Hồ sơ của cuong không lưu mật khẩu: nhập mật khẩu Cường đã đổi ở LNX-021.
        if (await c.exists('.modal input[type=password]')) {
          await c.click('.modal input[type=password]');
          await c.type('Cuong-doi-mat-khau-2026');
          await c.click('.modal .button.primary', 'Tiếp tục');
          await sleep(3000);
        }
        await shot('05'); // cuong bị từ chối
        await closeTab().catch(() => {});
        await goTab('an');
      },

      // ---------- LNX-028 ----------
      async 'nhap-passphrase'() {
        await sleep(600);
        await c.type('mot-cum-tu-chi-dung-cho-lab');
        await c.enter();
        await sleep(800);
      },
      async 'agent-ssh-studio'() {
        await editRow(SERVER);
        await c.click('.modal button', 'Agent', { exact: true });
        await sleep(300);
        await c.click('form .button.primary', 'Lưu');
        await sleep(800);
        await a.dismiss();
        await goTab('an');
        await newTab();
        await sleep(1500);
        await shot('03'); // không tìm thấy SSH agent trên Windows
        await closeTab();
        await editRow(SERVER);
        await c.click('.modal button', 'SSH Key', { exact: true });
        await chooseOption('SSH key', 'an-laptop');
        await c.click('form .button.primary', 'Lưu');
        await sleep(800);
        await a.dismiss();
        await goTab('an');
      },

      // ---------- LNX-029 ----------
      async 'dang-nhap-mfa'() {
        // Mã OTP chỉ dùng một lần: đợi sang khung 30 giây mới.
        await sleep((31 - (Math.floor(Date.now() / 1000) % 30)) * 1000);
        await newTab();
        await c.waitFor('.modal input[type=password]', null, 15000);
        await sleep(400);
        await shot('03'); // hộp thoại hỏi mã xác minh sau khi key đã được chấp nhận
        await c.click('.modal input[type=password]');
        await c.type(otp());
        await c.click('.modal .button.primary', 'Tiếp tục');
        await sleep(2500);
        await shot('04'); // đã vào phiên
        await closeTab();
        await goTab('an');
      },
    };
  },
});
console.log('Xong. Kiểm tra trạng thái cuối: npm run kiem-tra:p03');
