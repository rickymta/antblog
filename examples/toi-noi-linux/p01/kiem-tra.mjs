#!/usr/bin/env node
// Kiểm tra sản phẩm bàn giao của LNX-010 trên server lab ĐANG CHẠY (không tạo lại server):
//   node p01/kiem-tra.mjs
// Mỗi dòng PASS/FAIL là một yêu cầu trong đề bài. Mã thoát khác 0 nếu có FAIL.

import { asUser, root } from '../lib/lab.mjs';

const BASE = '/srv/toinoi/bao-gia';
let fail = 0;

function check(name, ok, detail = '') {
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${ok || !detail ? '' : `\n      ${detail}`}`);
  if (!ok) fail++;
}

/** "quyền chủ:nhóm" của một đường dẫn, hoặc chuỗi rỗng nếu không tồn tại. */
function st(path) {
  try {
    return root(`stat -c '%a %U:%G' '${path}' 2>/dev/null || true`);
  } catch {
    return '';
  }
}

const expect = [
  ['releases/1.4.2', '755 an:toinoi'],
  ['releases/1.4.2/kiem-tra-cau-hinh.sh', '750 an:toinoi'],
  ['releases/1.4.2/bang-gia.csv', '640 an:toinoi'],
  ['shared/config', '750 an:toinoi'],
  ['shared/config/appsettings.json', '640 an:toinoi'],
  ['shared/config/bi-mat.env', '600 an:toinoi'],
  ['shared/logs', '770 an:toinoi'],
];
for (const [path, want] of expect) {
  const got = st(`${BASE}/${path}`);
  check(`${path} có quyền và chủ sở hữu ${want}`, got === want, `thực tế: ${got || 'không tồn tại'}`);
}

const link = root(`readlink ${BASE}/current 2>/dev/null || true`);
check('current là symlink tương đối tới releases/1.4.2', link === 'releases/1.4.2', `thực tế: ${link || 'không có'}`);

const run = asUser(`${BASE}/current/kiem-tra-cau-hinh.sh`);
check('an chạy được current/kiem-tra-cau-hinh.sh', run.status === 0 && run.stdout.includes('cau hinh hop le'), run.stderr);

const errors = root(`wc -l < ${BASE}/shared/logs/loi-2026-11-21.txt 2>/dev/null || echo 0`);
check('loi-2026-11-21.txt có đúng 17 dòng ERROR', errors === '17', `thực tế: ${errors} dòng`);

const top = root(`head -n 1 ${BASE}/shared/logs/loi-theo-dich-vu.txt 2>/dev/null || true`).trim();
check('loi-theo-dich-vu.txt xếp car4 (6 lỗi) lên đầu', top === '6 dich_vu=car4', `dòng đầu: ${top}`);

const binhConfig = asUser(`cat ${BASE}/shared/config/appsettings.json`, 'binh');
check('binh (nhóm toinoi) đọc được appsettings.json', binhConfig.status === 0);

const binhSecret = asUser(`cat ${BASE}/shared/config/bi-mat.env`, 'binh');
check('binh KHÔNG đọc được bi-mat.env', binhSecret.status !== 0);

const other = root(`su -s /bin/sh nobody -c 'ls ${BASE}/shared/config' >/dev/null 2>&1 && echo ok || echo denied`);
check('người ngoài nhóm KHÔNG liệt kê được shared/config', other === 'denied');

const groups = root('id -nG an');
check('an thuộc nhóm toinoi', groups.split(' ').includes('toinoi'), `nhóm của an: ${groups}`);

console.log(`\n${expect.length + 8 - fail} PASS, ${fail} FAIL`);
process.exit(fail ? 1 : 0);
