#!/usr/bin/env node
// Kiểm tra sản phẩm bàn giao của LNX-040 trên server lab ĐANG CHẠY:
//   node p04/kiem-tra.mjs            đọc cấu hình, thử dịch vụ, giết tiến trình xem có tự chạy lại
//   node p04/kiem-tra.mjs --boot     thêm bước khởi động lại cả server (docker restart) rồi kiểm tra
//                                    bao-gia và timer tự chạy lại sau boot
// Mã thoát khác 0 nếu có FAIL.

import { CONTAINER, docker, root } from '../lib/lab.mjs';

let pass = 0;
let fail = 0;
function check(name, ok, detail = '') {
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${ok || !detail ? '' : `\n      ${detail}`}`);
  ok ? pass++ : fail++;
}
const sh = (cmd) => {
  const r = docker(['exec', CONTAINER, 'bash', '-c', cmd]);
  return { status: r.status, out: (r.stdout + r.stderr).trim() };
};
const wait = (s) => Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, s * 1000);
const prop = (unit, p) => root(`systemctl show ${unit} -p ${p} --value`);
const health = () => sh('curl -s --max-time 3 http://127.0.0.1:8080/suc-khoe');

// 1. Unit bao-gia
check('bao-gia.service đang chạy (active)', root('systemctl is-active bao-gia') === 'active');
check('bao-gia.service bật lúc boot (enabled)', root('systemctl is-enabled bao-gia') === 'enabled');
check('chạy bằng người dùng baogia, nhóm toinoi', prop('bao-gia', 'User') === 'baogia' && prop('bao-gia', 'Group') === 'toinoi',
  `User=${prop('bao-gia', 'User')} Group=${prop('bao-gia', 'Group')}`);
check('Restart=on-failure', prop('bao-gia', 'Restart') === 'on-failure', prop('bao-gia', 'Restart'));
const envFiles = prop('bao-gia', 'EnvironmentFiles');
check('đọc cấu hình từ shared/config/bao-gia.env và bi-mat.env', /bao-gia\.env/.test(envFiles) && /bi-mat\.env/.test(envFiles), envFiles);
check('ExecReload gửi SIGHUP', /kill -HUP/.test(prop('bao-gia', 'ExecReload')));
check('current trỏ tới releases/1.5.0', root('readlink /srv/toinoi/bao-gia/current') === 'releases/1.5.0');

// 2. Siết bằng drop-in
check('drop-in: ProtectSystem=strict', prop('bao-gia', 'ProtectSystem') === 'strict');
check('drop-in: ProtectHome=yes, PrivateTmp=yes', prop('bao-gia', 'ProtectHome') === 'yes' && prop('bao-gia', 'PrivateTmp') === 'yes');
check('drop-in: ghi được /var/log/toinoi', /\/var\/log\/toinoi/.test(prop('bao-gia', 'ReadWritePaths')));
check('drop-in: MemoryMax đặt (không phải infinity)', /^\d+$/.test(prop('bao-gia', 'MemoryMax')), prop('bao-gia', 'MemoryMax'));

// 3. Dịch vụ trả lời đúng
const h = health();
check('GET /suc-khoe trả trang_thai ok, phiên bản 1.5.0', /"trang_thai": "ok"/.test(h.out) && /"1\.5\.0"/.test(h.out), h.out);
const q = sh("curl -s 'http://127.0.0.1:8080/bao-gia?dich_vu=car4&km=12.5'");
check('GET /bao-gia?dich_vu=car4&km=12.5 trả gia 69000', /"gia": 69000/.test(q.out), q.out);
check('yêu cầu được ghi vào /var/log/toinoi/bao-gia.log', /dich_vu=car4 km=12\.5 gia=69000/.test(root('tail -n 1 /var/log/toinoi/bao-gia.log')));

// 4. Tự chạy lại khi tiến trình chết
const truoc = prop('bao-gia', 'MainPID');
root(`kill -KILL ${truoc}`);
wait(4);
const sau = prop('bao-gia', 'MainPID');
check('bị SIGKILL thì systemd tự khởi động lại', sau !== '0' && sau !== truoc && root('systemctl is-active bao-gia') === 'active', `MainPID ${truoc} -> ${sau}`);

// 5. reload và logrotate
root('systemctl reload bao-gia');
wait(1);
check('reload: ứng dụng nhận SIGHUP', /nhan SIGHUP/.test(root('journalctl -u bao-gia -n 3 -o cat --no-pager')));
const lr = sh('logrotate -d /etc/logrotate.d/bao-gia 2>&1');
check('/etc/logrotate.d/bao-gia hợp lệ (logrotate -d không báo lỗi)', lr.status === 0 && !/error/i.test(lr.out), lr.out.split('\n').find((l) => /error/i.test(l)) || '');
const lrc = root('cat /etc/logrotate.d/bao-gia');
check('logrotate: daily, rotate 14, compress, reload sau khi xoay', /daily/.test(lrc) && /rotate 14/.test(lrc) && /compress/.test(lrc) && /reload bao-gia/.test(lrc));

// 6. Timer dọn báo cáo và journal
check('bao-gia-don-bao-cao.timer bật và đang chờ', root('systemctl is-enabled bao-gia-don-bao-cao.timer') === 'enabled' && root('systemctl is-active bao-gia-don-bao-cao.timer') === 'active');
check('timer chạy 02:30 mỗi ngày, Persistent=yes', /02:30:00/.test(prop('bao-gia-don-bao-cao.timer', 'TimersCalendar')) && prop('bao-gia-don-bao-cao.timer', 'Persistent') === 'yes');
check('service dọn dẹp chạy bằng baogia, Type=oneshot', prop('bao-gia-don-bao-cao.service', 'User') === 'baogia' && prop('bao-gia-don-bao-cao.service', 'Type') === 'oneshot');
check('journald giới hạn SystemMaxUse', /SystemMaxUse=/.test(root('systemd-analyze cat-config systemd/journald.conf')));
check('an đọc được journal (nhóm systemd-journal)', /systemd-journal/.test(root('id -nG an')));

// 7. Quyền của deploy (Phần 3) vẫn đủ để CI khởi động lại dịch vụ
const d = sh('sudo -u deploy sudo -n /usr/bin/systemctl restart bao-gia');
wait(2);
check('deploy khởi động lại được bao-gia bằng sudo, không cần mật khẩu', d.status === 0 && root('systemctl is-active bao-gia') === 'active', d.out);

// 8. Khởi động lại cả server
if (process.argv.includes('--boot')) {
  docker(['restart', CONTAINER]);
  let ok = false;
  for (let i = 0; i < 30 && !ok; i++) {
    wait(2);
    ok = /"trang_thai": "ok"/.test(health().out);
  }
  check('sau khi khởi động lại server, bao-gia tự chạy và trả lời', ok);
  check('sau khi khởi động lại server, timer dọn báo cáo vẫn chờ', root('systemctl is-active bao-gia-don-bao-cao.timer') === 'active');
}

console.log(`\n${pass} PASS, ${fail} FAIL`);
process.exit(fail ? 1 : 0);
