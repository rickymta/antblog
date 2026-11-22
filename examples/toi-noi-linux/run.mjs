#!/usr/bin/env node
// Runner của lab "Linux qua SSH Studio và Tới Nơi".
//
//   node run.mjs up            dựng và khởi động server lab
//   node run.mjs down          xoá server lab
//   node run.mjs reset         tạo lại server lab từ ảnh (xoá mọi thay đổi)
//   node run.mjs p01           tạo lại server rồi chạy kịch bản của cả Phần 1, in bản ghi
//   node run.mjs p01 05        như trên nhưng chỉ in bản ghi của bài 05 (các bài trước chạy im lặng)
//   node run.mjs setup 05      đưa server về đúng trạng thái ĐẦU bài 05 để bạn tự làm theo
//   node run.mjs info          in thông tin kết nối và fingerprint của server lab

import { CONTAINER, PASSWORD, USER, down, reset, root, runScript, scripts, up } from './lib/lab.mjs';

const [cmd, arg] = process.argv.slice(2);

function info() {
  const fp = root('ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub').split(' ')[1];
  console.log(`Server lab : ${CONTAINER} (Ubuntu ${root('. /etc/os-release && echo $VERSION_ID')})`);
  console.log('Địa chỉ    : 127.0.0.1   Cổng: 2222');
  console.log(`Người dùng : ${USER}   Mật khẩu: ${PASSWORD} (chỉ dùng cho lab)`);
  console.log(`Fingerprint: ${fp} (ED25519)`);
}

function part(name, only) {
  reset();
  const out = [];
  for (const s of scripts(name)) {
    const show = !only || s.id === only;
    if (only && s.id > only) break;
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
    if (!/^\d\d$/.test(arg || '')) throw new Error('Dùng: node run.mjs setup <số bài, ví dụ 05>');
    reset();
    for (const s of scripts('p01')) {
      if (s.id >= arg) break;
      runScript(s.path, { echo: false });
    }
    console.log(`Server lab đang ở trạng thái đầu bài LNX-0${arg}.`);
    info();
    break;
  }
  case 'p01':
    part('p01', arg);
    break;
  default:
    console.log('Dùng: node run.mjs up | down | reset | info | setup <NN> | p01 [NN]');
    process.exit(cmd ? 1 : 0);
}
