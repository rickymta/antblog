#!/usr/bin/env node
// Chụp ảnh SSH Studio cho Phần 2 (LNX-011…LNX-020).
//
//   node tools/chup-anh/p02.mjs          đưa server về đầu LNX-011 (setup 11) rồi chụp cả phần
//   node tools/chup-anh/p02.mjs 15 16    chỉ vài bài; server phải đang ở đầu bài đầu tiên
//
// SSH Studio phải đang mở với cổng debug (xem README.md) và đã có server toinoi-app-01 với
// host key đã tin cậy (từ lần chụp Phần 1). Chạy xong, `npm run kiem-tra:p02` phải cho 20 PASS.

import { spawnSync } from 'node:child_process';
import { join } from 'node:path';

import { ROOT } from '../../lib/lab.mjs';
import { SERVER } from './app.mjs';
import { attach } from './cdp.mjs';
import { chayPhan } from './phan.mjs';

const SLUG = {
  11: 'linux-bien-moi-truong-path-bashrc',
  12: 'linux-vim-hang-ngay',
  13: 'linux-cut-sort-uniq-tr',
  14: 'linux-sed-sua-van-ban',
  15: 'linux-awk-xu-ly-log',
  16: 'linux-shell-script-dau-tien',
  17: 'linux-bash-dieu-kien-vong-lap-ham',
  18: 'linux-tar-gzip-nen-luu-tru',
  19: 'linux-shell-script-an-toan',
  20: 'linux-thuc-hanh-script-sao-luu-log',
};

const only = process.argv.slice(2);
if (!only.length) {
  console.log('Đưa server về đầu LNX-011…');
  const r = spawnSync('node', [join(ROOT, 'run.mjs'), 'setup', '11'], { stdio: 'inherit' });
  if (r.status !== 0) process.exit(r.status);
}

// Kết nối (lại) phiên SSH: server vừa được tạo lại nên phiên cũ đã chết.
{
  const { open } = await import('./app.mjs');
  const a = await open();
  const { c, sleep } = a;
  await a.dismiss();
  if (await c.exists('button', 'Kết nối lại')) {
    await c.click('button', 'Kết nối lại');
  } else if (!(await c.exists('.terminal.xterm'))) {
    await a.nav('Servers');
    await c.click('button.connect', 'Kết nối');
  }
  await sleep(1500);
  if (await c.exists('input[type=password]')) {
    await a.password();
    await a.submitPassword();
  }
  await a.view('Terminal');
  c.close();
}

await chayPhan({
  part: 'p02',
  slugs: SLUG,
  only,
  gui: ({ a, c, sleep, shot, files, unlinkLocal }) => ({
    // ---------- LNX-011 ----------
    async 'vim-bashrc'() {
      await a.run('vim ~/.bashrc', 1200);
      await c.type('G');
      await c.type(':set paste');
      await c.enter();
      await c.type('o');
      await c.enter();
      await c.type('# Lối tắt cho dịch vụ bao-gia (LNX-011)');
      await c.enter();
      await c.type("alias cdbg='cd /srv/toinoi/bao-gia'");
      await c.enter();
      await c.type("alias loibg='grep -c ERROR /var/log/toinoi/bao-gia.log'");
      await c.esc();
      await c.type(':set nopaste');
      await c.enter();
      await sleep(300);
      await shot('03');
      await c.type(':wq');
      await c.enter();
      await sleep(600);
    },

    // ---------- LNX-012 ----------
    async 'vim-bang-gia'() {
      await a.run('vim bang-gia.csv', 1200);
      await c.type(':set number');
      await c.enter();
      await c.type('/car7');
      await c.enter();
      await c.type('yyp');
      await sleep(300);
      await shot('01'); // dòng car7 được nhân đôi
      await c.type('cw');
      await c.type('van16');
      await c.esc();
      await c.type(':s/7 chỗ/16 chỗ/');
      await c.enter();
      await c.type(':s/15000,5500,30000/25000,8000,45000/');
      await c.enter();
      await sleep(300);
      await shot('02'); // dòng van16 đã sửa
      await c.type(':%s/Ô tô/Xe/g');
      await c.enter();
      await sleep(300);
      await shot('03'); // thông báo 3 substitutions on 3 lines
      await c.type('G');
      await c.type('dd');
      await c.type('u');
      await c.type(':wq');
      await c.enter();
      await sleep(600);
    },
    async 'vim-appsettings'() {
      await a.run('vim appsettings.json', 1200);
      await c.type(':%s#/opt/toinoi/bao-gia#/srv/toinoi/bao-gia/current#');
      await c.enter();
      await c.type('/Information');
      await c.enter();
      await c.type('cw');
      await c.type('Warning');
      await c.esc();
      await c.type('/8080');
      await c.enter();
      await c.type('cw');
      await c.type('8081');
      await c.esc();
      await c.type(':w');
      await c.enter();
      await c.type(':!diff /opt/toinoi/bao-gia/appsettings.json %');
      await c.enter();
      await sleep(900);
      await shot('04'); // kết quả :!diff, vim chờ Enter
      await c.enter();
      await c.type(':q');
      await c.enter();
      await sleep(600);
    },
    async 'vim-vimrc-thu'() {
      await a.run('vim /var/log/toinoi/bao-gia.log', 1200);
      await c.type('/q-100057');
      await c.enter();
      await sleep(400);
      await shot('05'); // số dòng và tô sáng kết quả nhờ ~/.vimrc
      await c.type(':q');
      await c.enter();
      await sleep(500);
    },

    // ---------- LNX-018 ----------
    async 'download-sao-luu'() {
      unlinkLocal('log-ba-ngay.tar.gz');
      await files('Files', '/home/an/sao-luu');
      await a.select('remote', 'log-ba-ngay.tar.gz');
      await a.paneButton('remote', 'Download');
      await sleep(2000);
      await a.paneButton('local', 'Làm mới');
      await sleep(800);
      await shot('04');
      if (await c.exists('button, span, a', 'Dọn lịch sử')) await c.click('.statusbar button, footer button', 'Truyền file');
      unlinkLocal('log-ba-ngay.tar.gz');
      await a.view('Terminal');
    },
  }),
});

// Đóng phiên kết nối CDP còn mở (nếu có) để node thoát.
(await attach()).close();
console.log('Xong. Kiểm tra trạng thái cuối: npm run kiem-tra:p02');
