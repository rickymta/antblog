#!/usr/bin/env node
// Kiểm tra sản phẩm bàn giao của LNX-020 trên server lab ĐANG CHẠY (không tạo lại server):
//   node p02/kiem-tra.mjs
// Mỗi dòng PASS/FAIL là một yêu cầu trong đề bài. Mã thoát khác 0 nếu có FAIL.
// Bộ chấm chạy script dưới tài khoản an trong thư mục tạm /tmp/kiem-tra-p02 và dọn sau khi xong.

import { asUser, root } from '../lib/lab.mjs';

const DIR = '/srv/toinoi/bao-gia/scripts';
const TMP = '/tmp/kiem-tra-p02';
let pass = 0;
let fail = 0;

function check(name, ok, detail = '') {
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${ok || !detail ? '' : `\n      ${detail}`}`);
  ok ? pass++ : fail++;
}

const st = (p) => root(`stat -c '%a %U:%G' '${p}' 2>/dev/null || true`);

// 1. Thư mục và quyền
const perms = [
  [DIR, '750 an:toinoi'],
  [`${DIR}/bao-cao-log`, '750 an:toinoi'],
  [`${DIR}/sao-luu-log`, '750 an:toinoi'],
  [`${DIR}/hang-ngay`, '750 an:toinoi'],
  [`${DIR}/tom-tat.awk`, '640 an:toinoi'],
];
for (const [p, want] of perms) {
  const got = st(p);
  check(`${p.replace(DIR, 'scripts')} có quyền và chủ sở hữu ${want}`, got === want, `thực tế: ${got || 'không tồn tại'}`);
}

// 2. shellcheck
const sc = asUser(`cd ${DIR} && shellcheck bao-cao-log sao-luu-log hang-ngay`);
check('shellcheck không cảnh báo gì với ba script', sc.status === 0, sc.stdout.split('\n').slice(0, 4).join(' | '));

// 3. bao-cao-log
const bc = asUser(`${DIR}/bao-cao-log /var/log/toinoi/bao-gia.log /var/log/toinoi/bao-gia.log.2.gz`);
check(
  'bao-cao-log đếm đúng 362 dòng, 17 lỗi, 24 chậm cho bao-gia.log',
  /bao-gia\.log\s+362 dòng\s+17 lỗi\s+24 chậm/.test(bc.stdout),
  bc.stdout || bc.stderr,
);
check('bao-cao-log đọc được file .gz (302 dòng, 18 lỗi)', /bao-gia\.log\.2\.gz\s+302 dòng\s+18 lỗi/.test(bc.stdout), bc.stdout);
const bad = asUser(`${DIR}/bao-cao-log /var/log/toinoi/bao-gia.log /khong-co`);
check('bao-cao-log trả mã 1 và báo lỗi ra stderr khi có file không đọc được', bad.status === 1 && /khong-co/.test(bad.stderr), `mã ${bad.status}`);
const none = asUser(`${DIR}/bao-cao-log`);
check('bao-cao-log không có đối số thì trả mã 2', none.status === 2, `mã ${none.status}`);

// 4. tom-tat.awk
const tt = asUser(`zcat -f /var/log/toinoi/bao-gia.log* | awk -f ${DIR}/tom-tat.awk`);
check(
  'tom-tat.awk cho ba ngày, ngày 21/11 có 360 yêu cầu, tb 95.3 ms',
  tt.stdout.split('\n').length === 4 && /2026-11-21\s+360\s+17\s+24\s+95\.3/.test(tt.stdout),
  tt.stdout.replace(/\n/g, ' | '),
);

// 5. sao-luu-log: giữ N bản, file 640, nội dung đủ
asUser(`rm -rf ${TMP}`);
const runs = ['2026-11-19', '2026-11-20', '2026-11-21'].map((d) =>
  asUser(`NGAY=${d} ${DIR}/sao-luu-log -d ${TMP} -g 2`),
);
const left = asUser(`ls ${TMP}`).stdout.split(/\s+/).filter(Boolean).join(' ');
check('sao-luu-log -g 2 chỉ giữ hai bản mới nhất', left === 'log-2026-11-20.tar.gz log-2026-11-21.tar.gz', `còn lại: ${left}`);
check('sao-luu-log không để lại file tạm', asUser(`ls -A ${TMP} | grep -c '^\\.log-'`).stdout === '0');
check('bản sao lưu có quyền 640', st(`${TMP}/log-2026-11-21.tar.gz`).startsWith('640 '), st(`${TMP}/log-2026-11-21.tar.gz`));
const list = asUser(`tar -tzf ${TMP}/log-2026-11-21.tar.gz | sort | tr '\\n' ' '`).stdout.trim();
check('bản sao lưu chứa đủ ba file log', list === './ ./bao-gia.log ./bao-gia.log.1 ./bao-gia.log.2.gz', list);
check('sao-luu-log báo đã ghi file', runs.every((r) => r.status === 0 && /đã ghi/.test(r.stdout)));
const locked = asUser(`flock -n ${TMP}/.sao-luu.lock ${DIR}/sao-luu-log -d ${TMP}`);
check('sao-luu-log trả mã 1 khi đang có bản khác giữ khoá', locked.status === 1, `mã ${locked.status}: ${locked.stderr}`);
const g = asUser(`${DIR}/sao-luu-log -d ${TMP} -g abc`);
check('sao-luu-log -g abc bị từ chối với mã 2', g.status === 2, `mã ${g.status}`);
asUser(`rm -rf ${TMP}`);

// 6. cron
const tab = asUser('crontab -l');
check(
  'crontab của an chạy hang-ngay lúc 01:17 mỗi ngày',
  /^17 1 \* \* \* \/srv\/toinoi\/bao-gia\/scripts\/hang-ngay\b/m.test(tab.stdout),
  tab.stdout || tab.stderr,
);
const log = root('cat /srv/toinoi/bao-gia/shared/logs/hang-ngay.log 2>/dev/null || true');
check('cron đã chạy hang-ngay ít nhất một lần thành công', /hang-ngay: xong/.test(log), log || 'chưa có hang-ngay.log');

console.log(`\n${pass} PASS, ${fail} FAIL`);
process.exit(fail ? 1 : 0);
