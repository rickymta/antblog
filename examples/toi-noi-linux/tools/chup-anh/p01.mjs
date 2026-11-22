#!/usr/bin/env node
// Chụp ảnh SSH Studio cho Phần 1: gõ TỪNG lệnh của kịch bản p01/NN-*.sh vào terminal của
// SSH Studio (đúng như người đọc làm theo bài), thao tác trên giao diện ở các dòng `#gui`,
// và chụp ở các dòng `#shot`. Chạy xong, server lab ở đúng trạng thái cuối Phần 1, nên
// `node p01/kiem-tra.mjs` phải cho 15 PASS: bằng chứng rằng ảnh và bản ghi cùng một quy trình.
//
//   node tools/chup-anh/p01.mjs            cả Phần 1 (tự tạo lại server lab trước)
//   node tools/chup-anh/p01.mjs 05 06      chỉ vài bài (server phải đang ở đầu bài đầu tiên)
//
// Chỉ dẫn trong file kịch bản (runner của lab coi chúng là ghi chú):
//   #clear       gõ `clear`
//   #shot NN     chụp ảnh ssh-studio-NN của bài
//   #gui tên     gọi thao tác giao diện cùng tên ở dưới
//   #@ lệnh      runner chạy thay cho thao tác giao diện; ở đây bỏ qua vì đã làm thật

import { mkdirSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { ROOT, reset, scripts } from '../../lib/lab.mjs';
import { SERVER, open } from './app.mjs';

const REPO = join(ROOT, '..', '..');
const sharp = createRequire(join(REPO, 'package.json'))('sharp');
const LOCAL = join(ROOT, 'may-cua-ban');

const SLUG = {
  '01': 'linux-la-gi-va-ssh-studio',
  '02': 'linux-dung-lab-docker-ket-noi-ssh-studio',
  '03': 'linux-shell-dong-lenh-tro-giup',
  '04': 'linux-cay-thu-muc-duong-dan',
  '05': 'linux-tao-sao-chep-di-chuyen-xoa-file',
  '06': 'linux-xem-sua-file-van-ban',
  '07': 'linux-nguoi-dung-nhom-quyen-file',
  '08': 'linux-chuyen-huong-pipe',
  '09': 'linux-tim-kiem-find-grep',
  '10': 'linux-thuc-hanh-thu-muc-trien-khai',
};

const only = process.argv.slice(2);
const a = await open();
const { c, sleep } = a;
let slug = '';

/** Chụp cửa sổ (2400×1350 điểm ảnh cho 1600×900 điểm CSS) và lưu WebP vào thư mục ảnh của bài. */
async function shot(n) {
  await sleep(350);
  const png = join(tmpdir(), `ssh-studio-${slug}-${n}.png`);
  await c.png(png);
  const dir = join(REPO, 'apps', 'web', 'public', 'images', 'articles', slug);
  mkdirSync(dir, { recursive: true });
  const out = join(dir, `ssh-studio-${n}.webp`);
  await sharp(png).webp({ quality: 90, effort: 6 }).toFile(out);
  console.log('  ảnh', `${slug}/ssh-studio-${n}.webp`);
}

/** Mở một chế độ có khung Files, trỏ khung local vào thư mục của lab và khung remote vào `remote`. */
async function files(view, remote) {
  await a.view(view);
  await a.path('local', LOCAL);
  await a.path('remote', remote);
  // Bỏ focus khỏi ô đường dẫn để ảnh không có viền nhập liệu.
  await c.click('.file-pane *', 'Remote server', { exact: true }).catch(() => {});
  await sleep(300);
}

const modalButton = (text) => c.click('.modal .button', text, { exact: true });

const gui = {
  // ---------- LNX-002 ----------
  async 'them-server-va-ket-noi'() {
    await a.nav('Servers');
    await shot('01'); // workspace trống
    await c.click('button', 'Thêm server đầu tiên');
    await sleep(500);
    await a.fill('Tên hiển thị', SERVER);
    await a.fill('Địa chỉ server', '127.0.0.1');
    await a.fill('Port', '2222');
    await a.fill('Username', 'an');
    await a.fill('Nhóm', 'Lab');
    await c.move(1500, 500);
    await shot('02'); // form Thêm server
    await c.click('form .button.primary', 'Lưu server');
    await sleep(900);
    await a.dismiss();
    await c.click('button.connect', 'Kết nối');
    await c.waitFor('.modal', 'Xác nhận danh tính server');
    await sleep(500);
    await shot('03'); // fingerprint
    await c.click('.button.primary', 'Tin cậy và kết nối');
    await a.password();
    await shot('04'); // hộp thoại mật khẩu
    await a.submitPassword();
    await shot('05'); // lần đăng nhập đầu tiên: MOTD và dấu nhắc
    await a.clear();
  },
  async 'known-hosts'() {
    await a.nav('Known Hosts');
    await shot('07');
    await c.click('button.session-nav', SERVER);
    await sleep(500);
  },

  // ---------- LNX-001 ----------
  async 'song-song-bao-gia'() {
    await a.clear();
    await a.run('ls -l /opt/toinoi/bao-gia');
    await files('Song song', '/opt/toinoi/bao-gia');
    await shot('02');
    await a.path('remote', '/home/an');
    await a.view('Terminal');
  },

  // ---------- LNX-003 ----------
  async 'man-ls'() {
    await a.run('man ls', 900);
    await shot('02');
    await a.typeOnly('q');
    await sleep(400);
  },
  async 'phim-tat'() {
    await c.click('button', 'Trợ giúp');
    await sleep(600);
    await shot('04');
    await c.esc();
    await sleep(300);
    if (await c.exists('.modal')) await c.click('.modal .icon-button', 'Đóng');
  },

  // ---------- LNX-004 ----------
  async 'files-goc'() {
    await files('Files', '/');
    await shot('03');
    await a.path('remote', '/home/an');
    await a.view('Terminal');
  },

  // ---------- LNX-005 ----------
  async 'upload-lich-truc'() {
    await files('Files', '/home/an/tai-lieu');
    await a.select('local', 'lich-truc.txt');
    await shot('03'); // đã chọn file local, sắp bấm Upload
    await a.paneButton('local', 'Upload');
    await sleep(2000);
    await a.paneButton('remote', 'Làm mới');
    await sleep(800);
    await shot('04'); // file đã nằm trên server, bảng Truyền file báo hoàn tất
    // Thu bảng Truyền file lại để các ảnh sau không bị che.
    if (await c.exists('button, span, a', 'Dọn lịch sử')) await c.click('.statusbar button, footer button', 'Truyền file');
    await a.view('Terminal');
    await a.clear();
  },

  // ---------- LNX-006 ----------
  async 'less-log'() {
    await a.run('less /var/log/toinoi/bao-gia.log', 800);
    await a.typeOnly('/ERROR');
    await c.enter();
    await sleep(500);
    await shot('02');
    await a.typeOnly('q');
    await sleep(400);
  },
  async 'sua-bang-gia'() {
    await files('Files', '/home/an/tai-lieu');
    await a.openRow('remote', 'bang-gia.csv');
    await c.waitFor('.editor-text');
    // Bôi đen "8000" ở dòng xe máy rồi gõ đè 9000.
    await c.click('.editor-text');
    await c.eval(`(() => {
      const t = document.querySelector('.editor-text');
      const i = t.value.indexOf('bike,Xe máy,8000,') + 'bike,Xe máy,'.length;
      t.focus();
      t.setSelectionRange(i, i + 4);
    })()`);
    await c.type('9000');
    await sleep(300);
    await shot('03'); // trình sửa file, chưa lưu
    await c.click('.modal .button.primary', 'Lưu');
    await sleep(1200);
    await a.dismiss();
    if (await c.exists('.modal .button', 'Đóng')) await modalButton('Đóng');
    await sleep(400);
    await a.view('Terminal');
  },
  async 'vim-ghi-chu'() {
    await a.run('vim tai-lieu/ghi-chu.txt', 1000);
    await a.typeOnly('i'); // vào chế độ Insert
    await c.type('Ghi chú của An');
    await c.enter();
    await c.type('- 22/11: tăng giá mở cửa xe máy lên 9.000 đ ở bản nháp');
    await sleep(400);
    await shot('04'); // vim ở chế độ Insert
    await c.esc(); // về chế độ Normal
    await sleep(300);
    await c.type(':wq');
    await sleep(300);
    await shot('05'); // dòng lệnh :wq, chưa Enter
    await c.enter();
    await sleep(600);
  },

  // ---------- LNX-007 ----------
  async 'chmod-kiem-tra'() {
    await files('Files', '/home/an/du-an/bao-gia');
    await a.select('remote', 'kiem-tra-cau-hinh.sh');
    await a.paneButton('remote', 'Đổi quyền');
    await sleep(500);
    await shot('03');
    await modalButton('Hủy');
    await sleep(300);
    await a.view('Terminal');
  },
  async 'sudo-mat-khau'() {
    await a.sudoPassword();
  },

  // ---------- LNX-008 ----------
  async 'dang-nhap-lai'() {
    await a.clear();
    await a.run('cd', 200);
    await a.run('id', 300);
    await a.run('exit', 1500);
    await c.waitFor('button', 'Kết nối lại');
    await shot('01'); // phiên đã kết thúc
    await c.click('button', 'Kết nối lại');
    await a.password();
    await a.submitPassword();
    await a.clear();
  },
  async 'chia-doi-tai-lieu'() {
    await files('Chia đôi', '/home/an/tai-lieu');
    await shot('04');
    await a.view('Terminal');
  },

  // ---------- LNX-009 ----------
  async 'tim-trong-terminal'() {
    await a.focusTerminal();
    await c.ctrl('f', { shift: true });
    await sleep(500);
    await c.click('input[placeholder="Tìm trong terminal"]');
    await c.type('delivery');
    await c.enter();
    await sleep(500);
    await shot('04');
    await c.click('button', 'Đóng (Esc)');
    await sleep(300);
  },

  // ---------- LNX-010 ----------
  async 'song-song-trien-khai'() {
    await files('Song song', '/srv/toinoi/bao-gia');
    await shot('03');
    await a.path('remote', '/home/an');
    await a.view('Terminal');
  },
};

async function play(script) {
  slug = SLUG[script.id];
  console.log(`${script.name} → ${slug}`);
  const lines = readFileSync(script.path, 'utf8').replace(/\r\n/g, '\n').split('\n');
  for (const line of lines) {
    if (!line.trim()) continue;
    if (line === '#clear') await a.clear();
    else if (line.startsWith('#shot ')) await shot(line.slice(6).trim());
    else if (line.startsWith('#gui ')) {
      const name = line.slice(5).trim();
      if (!gui[name]) throw new Error('Chưa có thao tác giao diện: ' + name);
      await gui[name]();
    } else if (line.startsWith('#')) continue;
    else {
      if (line.endsWith('\\') || /<</.test(line)) throw new Error('Trình chụp chưa hỗ trợ lệnh nhiều dòng: ' + line);
      const slow = /^(sudo|find|tree|grep -r|man|cp -r)/.test(line);
      await a.run(line, slow ? 900 : 450);
    }
  }
  // Mỗi bài bắt đầu ở thư mục home, như một phiên đăng nhập mới.
  await a.run('cd', 250);
}

const all = scripts('p01');
// Bài 2 dựng kết nối nên chạy trước bài 1; cả hai không đổi gì trên server.
const order = ['02', '01', '03', '04', '05', '06', '07', '08', '09', '10'];
const todo = (only.length ? only : order).map((id) => all.find((s) => s.id === id));
if (!only.length) {
  console.log('Tạo lại server lab…');
  reset();
}
for (const s of todo) await play(s);
c.close();
console.log('Xong. Kiểm tra trạng thái cuối: node p01/kiem-tra.mjs');
