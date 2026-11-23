#!/usr/bin/env node
// Kiểm tra sản phẩm bàn giao của LNX-030 trên server lab ĐANG CHẠY (không tạo lại server):
//   node p03/kiem-tra.mjs
// Ngoài việc đọc cấu hình, bộ chấm thử ĐĂNG NHẬP SSH THẬT từ trong container tới 127.0.0.1
// bằng các key của lab, kèm mã OTP khi cần. Mã thoát khác 0 nếu có FAIL.

import { docker, loginCommand, root } from '../lib/lab.mjs';

let pass = 0;
let fail = 0;
function check(name, ok, detail = '') {
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${ok || !detail ? '' : `\n      ${detail}`}`);
  ok ? pass++ : fail++;
}
const sh = (cmd) => {
  const r = docker(['exec', 'toinoi-app-01', 'bash', '-c', cmd]);
  return { status: r.status, out: (r.stdout + r.stderr).trim() };
};
const login = (user, key, otp = false) => sh(loginCommand(user, key, otp)).status === 0;
const sshdT = (ctx) => root(`sshd -T ${ctx ? `-C ${ctx}` : ''} 2>/dev/null`);
const has = (text, line) => text.split('\n').includes(line);

// Mã OTP chỉ dùng được một lần (DISALLOW_REUSE): chờ sang khung 30 giây mới trước khi thử.
function nextOtpWindow() {
  const s = 30 - (Math.floor(Date.now() / 1000) % 30) + 1;
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, s * 1000);
}

// 1. Tài khoản deploy
const id = sh('id deploy').out;
check('có tài khoản deploy thuộc nhóm toinoi và ssh-users', /toinoi/.test(id) && /ssh-users/.test(id), id);
check('deploy không có mật khẩu dùng được (passwd -S là L)', / L /.test(root('passwd -S deploy')), root('passwd -S deploy'));
const perms = root("stat -c '%a %U' /home/deploy/.ssh /home/deploy/.ssh/authorized_keys 2>&1 | tr '\\n' ' '").trim();
check('~deploy/.ssh là 700 và authorized_keys là 600, chủ là deploy', perms === '700 deploy 600 deploy', perms);
const fp = root('ssh-keygen -lf /home/deploy/.ssh/authorized_keys 2>&1');
check('authorized_keys của deploy chứa key deploy@ci', fp.includes('SHA256:yAG5n1d0W3YL5Q9tRyDrxn3P+LuTYFqC8CnyLarSPKs'), fp);

// 2. sudo
check('mọi file sudoers hợp lệ (visudo -c)', sh('visudo -c').status === 0, sh('visudo -c').out);
const sl = root('sudo -l -U deploy');
check(
  'deploy chỉ được restart/status bao-gia, không có ALL',
  /systemctl restart bao-gia/.test(sl) && /systemctl\s+status bao-gia/.test(sl) && !/\)\s+(NOPASSWD: )?ALL\b/.test(sl),
  sl.split('\n').slice(-2).join(' '),
);
check('sudoers của deploy có quyền 440', root("stat -c '%a' /etc/sudoers.d/deploy 2>&1") === '440');

// 3. sshd
const g = sshdT();
for (const line of [
  'passwordauthentication no',
  'permitrootlogin no',
  'allowgroups ssh-users',
  'authenticationmethods publickey,keyboard-interactive',
  'maxauthtries 3',
]) {
  check(`sshd -T: ${line}`, has(g, line));
}
const d = sshdT('user=deploy,host=ci,addr=10.0.0.5');
check('với deploy: authenticationmethods publickey', has(d, 'authenticationmethods publickey'));
check('với deploy: allowtcpforwarding no', has(d, 'allowtcpforwarding no'));
check('PAM của sshd dùng pam_google_authenticator', /^auth\s+required\s+pam_google_authenticator\.so/m.test(root('cat /etc/pam.d/sshd')));
const ga = root("stat -c '%a %U' /home/an/.google_authenticator 2>&1");
check('~an/.google_authenticator chỉ an đọc được (400 hoặc 600)', /^(400|600) an$/.test(ga), ga);

// 4. Đăng nhập thật
check('deploy đăng nhập được chỉ bằng key', login('deploy', 'deploy-ci'));
check('an KHÔNG đăng nhập được nếu chỉ có key (thiếu mã OTP)', !login('an', 'an-laptop'));
nextOtpWindow();
check('an đăng nhập được bằng key và mã OTP', login('an', 'an-laptop', true));
check('root KHÔNG đăng nhập được', !login('root', 'an-laptop'));
check('cuong (ngoài nhóm ssh-users) KHÔNG đăng nhập được', !login('cuong', 'an-laptop'));
const pw = sh('ssh -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o PreferredAuthentications=password an@127.0.0.1 true').out;
check('server không còn nhận đăng nhập bằng mật khẩu', /Permission denied \(publickey\)/.test(pw), pw);

console.log(`\n${pass} PASS, ${fail} FAIL`);
process.exit(fail ? 1 : 0);
