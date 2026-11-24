#!/usr/bin/env node
// Chụp ảnh SSH Studio cho Phần 6 (LNX-051…LNX-060).
//
//   node tools/chup-anh/p06.mjs          đưa lab về đầu LNX-051 (setup 51) rồi chụp cả phần
//   node tools/chup-anh/p06.mjs 55 56    chỉ vài bài; lab phải đang ở đầu bài đầu tiên
//
// Như p05.mjs: toinoi-app-01 dùng key an-laptop và mã OTP. Phần 6 thêm vào SSH Studio hai server:
// toinoi-bastion (127.0.0.1:2200, key an-laptop) và toinoi-app-02 (đi qua bastion). Dòng `#phien TÊN`
// của kịch bản chuyển sang gõ trong tab của máy đó; `#may TÊN` vẫn chạy ngầm.
// Chạy xong, `npm run kiem-tra:p06` phải cho toàn PASS.

import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, unlinkSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { ROOT, root, rootOn } from '../../lib/lab.mjs';
import { open } from './app.mjs';
import { LOCAL, chayPhan } from './phan.mjs';

const TOOLS = dirname(fileURLToPath(import.meta.url));
const SLUG = {
  51: 'linux-workspace-launchpad-nhieu-server',
  52: 'linux-bastion-jump-host',
  53: 'linux-sao-luu-khoi-phuc-ssh-studio',
  54: 'linux-cap-nhat-he-thong-an-toan',
  55: 'linux-gia-co-server-fail2ban',
  56: 'linux-nhieu-server-ansible-toi-thieu',
  57: 'linux-trien-khai-release-symlink',
  58: 'linux-su-co-dia-day-oom',
  59: 'linux-su-co-khong-ssh-duoc',
  60: 'linux-thuc-hanh-runbook-van-hanh',
};
const TEN = { app01: 'toinoi-app-01', bastion: 'toinoi-bastion', app02: 'toinoi-app-02' };
const DIA_CHI = { app01: '[127.0.0.1]:2222', bastion: '[127.0.0.1]:2200', app02: '[toinoi-app-02]:22' };
const MAT_KHAU_SAO_LUU = 'Sao-luu-ToiNoi-2026';
const FILE_SAO_LUU = join(LOCAL, 'sao-luu', 'ssh-studio-2026-11-24.sshstudio-backup');
const KET_QUA = join(ROOT, 'p06', 'tu-may-ban.txt');

const only = process.argv.slice(2);
if (!only.length) {
  console.log('Đưa lab về đầu LNX-051…');
  const r = spawnSync('node', [join(ROOT, 'run.mjs'), 'setup', '51'], { stdio: 'inherit' });
  if (r.status !== 0) process.exit(r.status);
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
const fpOf = (container) => rootOn(container, 'ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub').split(' ')[1];
const invoke = (c, cmd, args = {}) => c.eval(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args)})`);

/** Đóng mọi tab phiên. */
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

/** Bấm Kết nối của server `name` trên trang Servers. */
async function clickConnect(a, name) {
  const { c, sleep } = a;
  await a.nav('Servers');
  await sleep(500);
  const row = await c.eval(rowButton(name, 'button.connect'));
  if (!row) throw new Error('Không thấy nút Kết nối của ' + name);
  if (name === TEN.app01) await nextOtpWindow(sleep);
  await c.clickAt(row.x, row.y);
}

/**
 * Trả lời các hộp thoại của một lần kết nối: tin host key (khi `tin`), mã OTP của app01. Dừng khi
 * có terminal mới, hoặc khi `dungTruocTin` và hộp thoại xác nhận host key hiện ra (để chụp nó).
 */
async function answerPrompts(a, { tin = true, dungTruocTin = false, soTab } = {}) {
  const { c, sleep } = a;
  for (let i = 0; i < 60; i++) {
    await sleep(500);
    if (await c.exists('.button.primary', 'Tin cậy và kết nối')) {
      if (dungTruocTin) return 'host-key';
      if (!tin) return 'host-key';
      await c.click('.button.primary', 'Tin cậy và kết nối');
      continue;
    }
    if (await c.exists('.modal input[type=password]')) {
      await c.click('.modal input[type=password]');
      await c.type(otp());
      await c.click('.modal .button.primary', 'Tiếp tục');
      continue;
    }
    const n = await c.eval(`document.querySelectorAll('button.session-nav').length`);
    if (soTab !== undefined && n > soTab) break;
  }
  await sleep(2500);
}

/** Kết nối tới `may` và chờ dấu nhắc. */
async function connect(a, may) {
  const { c } = a;
  const soTab = await c.eval(`document.querySelectorAll('button.session-nav').length`);
  await clickConnect(a, TEN[may]);
  await answerPrompts(a, { soTab });
  const loi = await c.eval(`document.querySelector('main')?.innerText.match(/Lỗi kết nối[\\s\\S]{0,240}/)?.[0] || ''`);
  if (loi) throw new Error('Kết nối thất bại: ' + loi.replace(/\s+/g, ' '));
}

// ---------- Đầu lượt chụp ----------
{
  const a = await open();
  const { c, sleep } = a;
  const data = await invoke(c, 'snapshot');
  // Host key sinh lúc build ảnh lab: dựng lại ảnh là fingerprint của app01 đổi.
  const fp = fpOf('toinoi-app-01');
  if (data.knownHosts[DIA_CHI.app01] && data.knownHosts[DIA_CHI.app01] !== fp) {
    console.log(`Quên host key cũ của ${DIA_CHI.app01}`);
    await invoke(c, 'forget_host', { address: DIA_CHI.app01 });
  }
  await c.eval('location.reload()');
  await sleep(3000);
  await closeAllTabs(c, sleep);
  if (!only.length || only[0] === '51') {
    // Chụp từ đầu phần: SSH Studio về trạng thái cuối Phần 5 (chưa có bastion, app02, thư mục).
    const d = await invoke(c, 'snapshot');
    // Hồ sơ thử của cuong (Phần 3, đăng nhập bằng mật khẩu) không thuộc hệ thống của Tới Nơi:
    // bỏ đi, để "Kết nối tất cả" chỉ mở các server thật.
    for (const ten of [TEN.app02, TEN.bastion, 'toinoi-app-01 (cuong)']) {
      const h = d.hosts.find((x) => x.name === ten);
      if (h) await invoke(c, 'delete_host', { id: h.id });
    }
    for (const dc of [DIA_CHI.bastion, DIA_CHI.app02]) if (d.knownHosts[dc]) await invoke(c, 'forget_host', { address: dc });
    const ws = d.workspaces.find((w) => w.id === d.activeWorkspace) || d.workspaces[0];
    for (const f of ws.folders.filter((f) => !f.parent)) await invoke(c, 'delete_folder', { workspaceId: ws.id, id: f.id });
    for (const h of (await invoke(c, 'snapshot')).hosts) {
      if (h.group !== 'Lab' || h.folderId) await invoke(c, 'save_host', { host: { ...h, group: 'Lab', folderId: '' } });
    }
    await c.eval('location.reload()');
    await sleep(3000);
  } else if (only.length) {
    // Chụp lại vài bài: `run.mjs setup` dựng lại host key gốc của bastion và app02, còn SSH Studio
    // có thể đang nhớ key mà một bài sau đã đổi (LNX-059). Quên key lệch rồi kết nối một lần để
    // tin lại key gốc, nếu không thì hộp thoại host key chen vào các lỗi mà bài muốn chụp.
    const d = await invoke(c, 'snapshot');
    for (const may of ['bastion', 'app02']) {
      const dc = DIA_CHI[may];
      if (!d.knownHosts[dc] || d.knownHosts[dc] === fpOf(TEN[may])) continue;
      console.log(`Tin lại host key gốc của ${dc}`);
      await invoke(c, 'forget_host', { address: dc });
      await connect(a, may);
      await closeAllTabs(c, sleep);
    }
  }
  await connect(a, 'app01');
  c.close();
}

await chayPhan({
  part: 'p06',
  slugs: SLUG,
  only,
  gui: ({ a, c, sleep, shot }) => {
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
    async function editRow(name) {
      await a.nav('Servers');
      await sleep(400);
      const p = await c.eval(rowButton(name, 'button[title="Sửa server"]'));
      if (!p) throw new Error('Không thấy nút sửa của ' + name);
      await c.clickAt(p.x, p.y);
      await sleep(600);
    }
    async function luuServer() {
      await c.click('form .button.primary', 'Lưu server');
      await sleep(800);
      await a.dismiss();
    }
    /** Chuyển sang tab phiên của server `ten`; trả về false nếu chưa có tab. */
    async function goTab(ten) {
      if (!(await c.exists('button.session-nav', ten))) return false;
      await c.click('button.session-nav', ten);
      await sleep(700);
      return true;
    }
    async function closeTab(ten) {
      if (!(await goTab(ten))) return;
      // Nút đóng của tab đang chọn (tab của `ten`), không phải tab cuối cùng.
      const p = await c.eval(`(() => {
        const b = document.querySelector('.tab.selected button[title="Đóng phiên"]');
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
    /** Hộp thoại chọn/lưu file của Windows: dán `path` vào ô tên file (chon-file.ps1). */
    function chonFile(path) {
      return spawn('powershell.exe', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', join(TOOLS, 'chon-file.ps1'), '-Path', path]);
    }
    /** Lỗi đang hiển thị (toast hoặc băng lỗi của phiên đang xem); phiên ẩn vẫn còn trong DOM nên lọc. */
    const LOI_HIEN = `[...document.querySelectorAll('.toast.error, .notice.error')].filter((e) => e.getBoundingClientRect().width > 0)`;
    /** Đánh dấu các lỗi đang có, để choLoi chỉ nhận lỗi mới xuất hiện sau đó. */
    const danhDauLoi = () => c.eval(`${LOI_HIEN}.forEach((e) => (e.dataset.cu = '1'))`);
    /** Chờ một lỗi kết nối MỚI hiện ra, tối đa `giay` giây. */
    async function choLoi(giay = 30) {
      for (let i = 0; i < giay * 2; i++) {
        await sleep(500);
        const t = await c.eval(`(() => { const e = ${LOI_HIEN}.find((e) => !e.dataset.cu); return e ? e.innerText : ''; })()`);
        if (t) return t;
      }
      throw new Error('Không thấy lỗi kết nối');
    }
    /** Mở server `ten` (đóng phiên cũ nếu có) và chờ lỗi. */
    async function ketNoiLoi(ten, giay) {
      await closeTab(ten);
      await a.dismiss();
      await danhDauLoi();
      await clickConnect(a, ten);
      const t = await choLoi(giay);
      console.log('  lỗi:', t.replace(/\s+/g, ' ').slice(0, 200));
      await sleep(600);
    }

    return {
      /** `#phien TÊN`: sang tab của máy đó, mở kết nối nếu chưa có. */
      async phien(may) {
        // Toast lỗi của bước trước nằm đúng chỗ focusTerminal bấm vào: tắt đi để lệnh gõ vào terminal.
        await a.dismiss();
        if (await goTab(TEN[may])) return;
        await connect(a, may);
      },

      // ---------- LNX-051 ----------
      async 'them-bastion'() {
        await editRow(TEN.app01);
        await a.fill('Nhóm', 'Production');
        await luuServer();
        await a.nav('Servers');
        await c.click('button.primary', 'Thêm server', { exact: true });
        await sleep(500);
        await a.fill('Tên hiển thị', TEN.bastion);
        await a.fill('Địa chỉ server', '127.0.0.1');
        await a.fill('Port', '2200');
        await a.fill('Username', 'an');
        await a.fill('Nhóm', 'Hạ tầng');
        await c.click('.modal button', 'SSH Key', { exact: true });
        await chooseOption('SSH key', 'an-laptop');
        await sleep(400);
        await shot('01'); // form Thêm server: toinoi-bastion 127.0.0.1:2200, nhóm Hạ tầng, key an-laptop
        await luuServer();
      },
      async 'thu-muc'() {
        await a.nav('Servers');
        await c.click('button.tree-hint');
        await sleep(600);
        await c.click('.confirm-actions .button.primary', 'Tạo thư mục');
        await sleep(1200);
        await a.dismiss();
        await shot('02'); // cây thư mục Production / Hạ tầng / Lab, danh sách theo thư mục
      },
      async 'ket-noi-tat-ca'() {
        await a.nav('Servers');
        await c.click('button', 'Kết nối tất cả', { exact: true });
        const r = await answerPrompts(a, { dungTruocTin: true });
        if (r !== 'host-key') throw new Error('Không thấy hộp thoại xác nhận host key của bastion');
        await shot('03'); // xác nhận danh tính toinoi-bastion lần đầu
        const soTab = await c.eval(`document.querySelectorAll('button.session-nav').length`);
        await c.click('.button.primary', 'Tin cậy và kết nối');
        await answerPrompts(a, { soTab });
      },
      async launchpad() {
        await a.nav('Launchpad');
        await sleep(600);
        await c.click('.lp-tabs [role=tab]', 'Phiên đang mở');
        await sleep(600);
        await shot('05'); // Launchpad: hai phiên đang mở, app01 và bastion
        await goTab(TEN.app01);
      },

      // ---------- LNX-052 ----------
      async 'them-app02'() {
        await a.nav('Servers');
        await c.click('button.primary', 'Thêm server', { exact: true });
        await sleep(500);
        await a.fill('Tên hiển thị', TEN.app02);
        await a.fill('Địa chỉ server', 'toinoi-app-02');
        await a.fill('Port', '22');
        await a.fill('Username', 'an');
        await a.fill('Nhóm', 'Production');
        await c.click('.modal button', 'SSH Key', { exact: true });
        await chooseOption('SSH key', 'an-laptop');
        await c.click('.modal button', 'Jump host', { exact: true });
        await sleep(300);
        await chooseOption('Jump host (ProxyJump)', TEN.bastion);
        await sleep(400);
        await shot('02'); // form: toinoi-app-02, Kết nối qua Jump host toinoi-bastion
        await luuServer();
      },
      async 'ket-noi-app02'() {
        const soTab = await c.eval(`document.querySelectorAll('button.session-nav').length`);
        await clickConnect(a, TEN.app02);
        const r = await answerPrompts(a, { dungTruocTin: true });
        if (r !== 'host-key') throw new Error('Không thấy hộp thoại xác nhận host key của app02');
        await shot('03'); // xác nhận danh tính toinoi-app-02 (kết nối qua bastion)
        await c.click('.button.primary', 'Tin cậy và kết nối');
        await answerPrompts(a, { soTab });
      },

      // ---------- LNX-053 ----------
      async 'sao-luu'() {
        await a.nav('SSH Keys');
        await c.click('button', 'Sao lưu', { exact: true });
        await sleep(500);
        await a.fill('Mật khẩu sao lưu', MAT_KHAU_SAO_LUU);
        await a.fill('Nhập lại mật khẩu', MAT_KHAU_SAO_LUU);
        await sleep(300);
        await shot('01'); // hộp thoại Sao lưu workspace
      },
      async 'sao-luu-xong'() {
        mkdirSync(dirname(FILE_SAO_LUU), { recursive: true });
        if (existsSync(FILE_SAO_LUU)) unlinkSync(FILE_SAO_LUU);
        const chon = chonFile(FILE_SAO_LUU);
        await c.click('.modal button', 'Chọn nơi lưu…');
        await new Promise((ok) => chon.on('exit', ok));
        await c.waitFor('.toast', null, 15000);
        await sleep(600);
        if (!existsSync(FILE_SAO_LUU)) throw new Error('Không thấy file sao lưu ' + FILE_SAO_LUU);
        await shot('02'); // thông báo: đã sao lưu N server, N key, N known host
        await a.dismiss();
      },
      async 'workspace-json'() {
        // Từ máy Windows: workspace.json của SSH Studio (bản chụp) chỉ chứa bí mật đã mã hoá bằng
        // khoá trong Credential Manager. Ghi phần đầu của một host và một key để bài viết dẫn lại.
        const file = join(process.env.APPDATA, 'dev.sshstudio.blog', 'workspace.json');
        const j = JSON.parse(readFileSync(file, 'utf8'));
        const rut = (v) => (typeof v === 'string' && v.length > 48 ? v.slice(0, 44) + '…' : v);
        const key = Object.fromEntries(Object.entries((j.keys || [])[0] || {}).map(([k, v]) => [k, rut(v)]));
        const head = JSON.stringify({ keys: [key] }, null, 2);
        const sl = JSON.parse(readFileSync(FILE_SAO_LUU, 'utf8'));
        const env = JSON.stringify({ ...sl, nonce: rut(sl.nonce), data: rut(sl.data) }, null, 2);
        writeFileSync(KET_QUA, `PS> Get-Content $env:APPDATA\\dev.sshstudio.desktop\\workspace.json  # rút gọn: chỉ key đầu tiên\n${head}\n\nPS> Get-Content .\\ssh-studio-2026-11-24.sshstudio-backup  # rút gọn\n${env}\n`);
        console.log('  ghi', KET_QUA);
      },
      async 'mat-server'() {
        // "Máy mới" thiếu toinoi-app-02 và host key của nó: xoá chúng khỏi SSH Studio.
        await closeTab(TEN.app02);
        const d = await invoke(c, 'snapshot');
        const h = d.hosts.find((x) => x.name === TEN.app02);
        if (h) await invoke(c, 'delete_host', { id: h.id });
        await invoke(c, 'forget_host', { address: DIA_CHI.app02 });
        await c.eval('location.reload()');
        await sleep(3000);
        await a.nav('Servers');
        await sleep(600);
        await shot('03'); // danh sách server không còn toinoi-app-02
      },
      async 'khoi-phuc'() {
        await a.nav('SSH Keys');
        await c.click('button', 'Khôi phục', { exact: true });
        await sleep(500);
        const chon = chonFile(FILE_SAO_LUU);
        await c.click('.modal button', 'Chọn…');
        await new Promise((ok) => chon.on('exit', ok));
        await sleep(600);
        await a.fill('Mật khẩu sao lưu', MAT_KHAU_SAO_LUU);
        await c.click('.modal .button.primary', 'Xem trước');
        await c.waitFor('table.restore-summary', null, 15000);
        await sleep(500);
        await shot('04'); // xem trước: 1 server mới, known host mới, key đã có
      },
      async 'khoi-phuc-xong'() {
        await c.click('.modal .button.primary', 'Khôi phục');
        await c.waitFor('.toast', null, 15000);
        await sleep(600);
        await a.nav('Servers');
        await sleep(600);
        await shot('05'); // toinoi-app-02 trở lại, thông báo đã khôi phục
        await a.dismiss();
        // Tải lại cửa sổ ở bước trước đã đóng mọi phiên: mở lại app01 cho phần còn lại của bài.
        if (!(await goTab(TEN.app01))) await connect(a, 'app01');
      },

      // ---------- LNX-059 ----------
      async 'sai-cong'() {
        await editRow(TEN.bastion);
        await a.fill('Port', '2201');
        await luuServer();
        await closeTab(TEN.bastion);
        await danhDauLoi();
        await clickConnect(a, TEN.bastion);
        const t = await choLoi(30);
        console.log('  lỗi:', t.replace(/\s+/g, ' ').slice(0, 200));
        await sleep(600);
        await shot('01'); // cổng 2201: máy từ chối kết nối (lỗi của hệ điều hành)
        await a.dismiss();
        // Tab lỗi của bastion không dùng nữa: đóng, để băng lỗi của nó không lẫn vào các tình huống sau.
        await closeTab(TEN.bastion);
        await editRow(TEN.bastion);
        await a.fill('Port', '2200');
        await luuServer();
        await goTab(TEN.app01);
      },
      async 'ket-noi-app02-loi'() {
        await ketNoiLoi(TEN.app02, 170);
      },
      async 'tin-host-key-moi'() {
        await a.dismiss();
        await closeTab(TEN.app02);
        await a.nav('Known Hosts');
        await sleep(500);
        await invoke(c, 'forget_host', { address: DIA_CHI.app02 });
        const soTab = await c.eval(`document.querySelectorAll('button.session-nav').length`);
        await clickConnect(a, TEN.app02);
        const r = await answerPrompts(a, { dungTruocTin: true });
        if (r !== 'host-key') throw new Error('Không thấy hộp thoại host key mới của app02');
        await shot('07'); // fingerprint mới của app02, đối chiếu với console
        await c.click('.button.primary', 'Tin cậy và kết nối');
        await answerPrompts(a, { soTab });
        await goTab(TEN.app01);
      },
    };
  },
});
