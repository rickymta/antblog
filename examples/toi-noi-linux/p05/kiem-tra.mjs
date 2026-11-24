#!/usr/bin/env node
// Kiểm tra sản phẩm bàn giao của LNX-050 trên server lab ĐANG CHẠY:
//   node p05/kiem-tra.mjs
// Ngoài đọc cấu hình, bộ chấm thử kết nối thật từ toinoi-app-02 tới toinoi-app-01 (cổng mở và
// cổng bị chặn), gọi HTTPS qua nginx và chạy báo cáo tài nguyên. Mã thoát khác 0 nếu có FAIL.

import { docker, root } from '../lib/lab.mjs';

let pass = 0;
let fail = 0;
function check(name, ok, detail = '') {
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${ok || !detail ? '' : `\n      ${detail}`}`);
  ok ? pass++ : fail++;
}
const on = (container, cmd) => {
  const r = docker(['exec', container, 'bash', '-c', cmd]);
  return { status: r.status, out: (r.stdout + r.stderr).trim() };
};
const app02 = (cmd) => on('toinoi-app-02', cmd);
const port = (p) => app02(`nc -z -w 3 toinoi-app-01 ${p}`).status === 0;

// 1. Tường lửa
const ufw = root('ufw status verbose');
check('ufw đang bật', /Status: active/.test(ufw), ufw.split('\n')[0]);
check('mặc định chặn kết nối vào', /Default: deny \(incoming\)/.test(ufw));
check('SSH được giới hạn tần suất (LIMIT)', /^22\/tcp( \(OpenSSH\))?\s+LIMIT IN\s+Anywhere/m.test(ufw));
check('443 chỉ mở cho 10.20.0.0/24', /^443\/tcp\s+ALLOW IN\s+10\.20\.0\.0\/24/m.test(ufw) && !/^443\/tcp\s+ALLOW IN\s+Anywhere/m.test(ufw));
check('từ app02: cổng 22 mở', port(22));
check('từ app02: cổng 443 mở', port(443));
check('từ app02: cổng 80 bị chặn', !port(80));
check('từ app02: cổng 8080 (bao-gia) không tới được', !port(8080));

// 2. bao-gia chỉ nghe ở 127.0.0.1, nginx trước nó
const ss = root('ss -tlnH');
check('bao-gia nghe ở 127.0.0.1:8080, không ở 0.0.0.0', /127\.0\.0\.1:8080/.test(ss) && !/0\.0\.0\.0:8080/.test(ss));
check('nginx -t hợp lệ', on('toinoi-app-01', 'nginx -t').status === 0);
const https = app02('curl -sk --resolve bao-gia.toinoi.lab:443:10.20.0.11 https://bao-gia.toinoi.lab/suc-khoe');
check('HTTPS qua nginx trả /suc-khoe của bao-gia', /"trang_thai": "ok"/.test(https.out), https.out);
const cert = root("openssl x509 -in /etc/ssl/certs/bao-gia.toinoi.lab.crt -noout -ext subjectAltName 2>&1");
check('chứng chỉ có SAN DNS:bao-gia.toinoi.lab', /DNS:bao-gia\.toinoi\.lab/.test(cert), cert);
check('khoá riêng của chứng chỉ chỉ root đọc được', /^6[04]0 root$/.test(root("stat -c '%a %U' /etc/ssl/private/bao-gia.toinoi.lab.key")));
const redir = root("curl -s -o /dev/null -w '%{http_code} %{redirect_url}' http://bao-gia.toinoi.lab/suc-khoe");
check('HTTP chuyển sang HTTPS (301)', /^301 https:\/\/bao-gia\.toinoi\.lab\/suc-khoe/.test(redir), redir);

// 3. Ổ dữ liệu
check('/du-lieu đang được gắn, ext4', /ext4/.test(on('toinoi-app-01', 'findmnt -n -o FSTYPE /du-lieu').out));
const fstab = root('grep du-lieu /etc/fstab || true');
check('fstab gắn /du-lieu theo UUID, có nofail', /^UUID=\S+\s+\/du-lieu\s+ext4\s+\S*nofail/m.test(fstab), fstab);

// 4. Báo cáo tài nguyên
check('bao-cao-tai-nguyen.timer bật và đang chờ', root('systemctl is-enabled bao-cao-tai-nguyen.timer') === 'enabled' && root('systemctl is-active bao-cao-tai-nguyen.timer') === 'active');
check('timer 15 phút một lần', /\*:00?\/15/.test(root('systemctl show bao-cao-tai-nguyen.timer -p TimersCalendar --value')));
const truoc = Number(root('wc -l < /var/log/toinoi/tai-nguyen.log 2>/dev/null || echo 0'));
const chay = on('toinoi-app-01', 'systemctl start bao-cao-tai-nguyen.service');
const sau = Number(root('wc -l < /var/log/toinoi/tai-nguyen.log'));
const cuoi = root('tail -n 8 /var/log/toinoi/tai-nguyen.log');
check('chạy báo cáo ghi thêm vào /var/log/toinoi/tai-nguyen.log', chay.status === 0 && sau > truoc && /^=== /m.test(cuoi) && /bao-gia: active/.test(cuoi), cuoi);

// 5. Bản sao lưu ở app02
const sl = app02("su - an -c 'ls sao-luu/releases'");
check('app02 có bản sao lưu các release (rsync qua SSH)', /1\.5\.0/.test(sl.out) && /1\.6\.0/.test(sl.out), sl.out);

console.log(`\n${pass} PASS, ${fail} FAIL`);
process.exit(fail ? 1 : 0);
