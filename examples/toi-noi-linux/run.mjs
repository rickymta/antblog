#!/usr/bin/env node
// Runner của lab "Linux qua SSH Studio và Tới Nơi".
//
//   node run.mjs up            dựng và khởi động server lab
//   node run.mjs down          xoá server lab
//   node run.mjs reset         tạo lại server lab từ ảnh (xoá mọi thay đổi)
//   node run.mjs p01           tạo lại server rồi chạy kịch bản của cả Phần 1, in bản ghi
//   node run.mjs p02 15        tạo lại server, chạy im lặng các bài trước, chỉ in bản ghi bài 15
//   node run.mjs setup 15      đưa server về đúng trạng thái ĐẦU bài LNX-015 để bạn tự làm theo
//   node run.mjs info          in thông tin kết nối và fingerprint của server lab
//
// Bài được đánh số liên tục qua các phần: p01/01…10, p02/11…20. Mỗi bài bắt đầu từ trạng thái
// cuối của bài trước nó.

import { existsSync } from 'node:fs';
import { join } from 'node:path';

import { CONTAINER, PASSWORD, ROOT, USER, down, reset, root, runScript, scripts, up } from './lib/lab.mjs';

const PARTS = ['p01', 'p02', 'p03'];
const [cmd, arg] = process.argv.slice(2);

function info() {
  const fp = root('ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub').split(' ')[1];
  console.log(`Server lab : ${CONTAINER} (Ubuntu ${root('. /etc/os-release && echo $VERSION_ID')})`);
  console.log('Địa chỉ    : 127.0.0.1   Cổng: 2222');
  console.log(`Người dùng : ${USER}   Mật khẩu: ${PASSWORD} (chỉ dùng cho lab)`);
  console.log(`Fingerprint: ${fp} (ED25519)`);
}

/** Mọi kịch bản của các phần, theo thứ tự bài. */
const all = () =>
  PARTS.filter((p) => existsSync(join(ROOT, p))).flatMap((p) => scripts(p).map((s) => ({ ...s, part: p })));

function part(name, only) {
  if (!PARTS.includes(name)) throw new Error('Không có phần ' + name);
  reset();
  const out = [];
  for (const s of all()) {
    const mine = s.part === name;
    if (PARTS.indexOf(s.part) > PARTS.indexOf(name) || (only && s.id > only)) break;
    const show = mine && (!only || s.id === only);
    const text = runScript(s.path, { echo: show });
    if (show) out.push(`==== ${name}/${s.name} ====\n${text.trimEnd()}\n`);
  }
  process.stdout.write(out.join('\n'));
}

switch (cmd) {
  case 'up':
    up();
    info();
    break;
  case 'down':
    down();
    break;
  case 'reset':
    reset();
    info();
    break;
  case 'info':
    info();
    break;
  case 'setup': {
    if (!/^\d\d$/.test(arg || '')) throw new Error('Dùng: node run.mjs setup <số bài, ví dụ 05 hoặc 15>');
    reset();
    for (const s of all()) {
      if (s.id >= arg) break;
      runScript(s.path, { echo: false });
    }
    console.log(`Server lab đang ở trạng thái đầu bài LNX-0${arg}.`);
    info();
    break;
  }
  default:
    if (PARTS.includes(cmd)) {
      part(cmd, arg);
      break;
    }
    console.log(`Dùng: node run.mjs up | down | reset | info | setup <NN> | ${PARTS.join(' | ')} [NN]`);
    process.exit(cmd ? 1 : 0);
}
