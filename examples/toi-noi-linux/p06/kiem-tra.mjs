#!/usr/bin/env node
// Kiểm tra sản phẩm bàn giao của Phần 6 (LNX-060) trên lab ĐANG CHẠY:
//   node p06/kiem-tra.mjs
// Ngoài đọc cấu hình, bộ chấm thử thật: SSH tới app02 chỉ đi được từ bastion và app01, Ansible từ
// app01 tới cả hai máy, chạy script kiểm tra vận hành. Mã thoát khác 0 nếu có FAIL.

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
const asAn = (container, cmd) => on(container, `su - an -c ${JSON.stringify(cmd)}`);

// 1. Bastion và app02
const sshdB = on('toinoi-bastion', 'sshd -T').out;
check('bastion: không nhận mật khẩu', /^passwordauthentication no$/m.test(sshdB) && /^kbdinteractiveauthentication no$/m.test(sshdB));
check('bastion: không chuyển tiếp agent, không X11', /^allowagentforwarding no$/m.test(sshdB) && /^x11forwarding no$/m.test(sshdB));
const nc = (from, to) => on(from, `nc -z -w 3 ${to} 22`).status === 0;
check('app02: tới được cổng 22 từ bastion', nc('toinoi-bastion', 'toinoi-app-02'));
check('app02: tới được cổng 22 từ app01 (rsync)', nc('toinoi-app-01', 'toinoi-app-02'));
const ufw2 = on('toinoi-app-02', 'ufw status verbose').out;
check('app02: ufw bật, mặc định chặn kết nối vào', /Status: active/.test(ufw2) && /Default: deny \(incoming\)/.test(ufw2), ufw2.split('\n')[0]);
check('app02: SSH chỉ mở cho 10.20.0.10 và 10.20.0.11', /^22\/tcp\s+ALLOW IN\s+10\.20\.0\.10/m.test(ufw2) && /^22\/tcp\s+ALLOW IN\s+10\.20\.0\.11/m.test(ufw2) && !/^22\/tcp\s+ALLOW IN\s+Anywhere/m.test(ufw2));

// 2. Cập nhật và gia cố app01
check('unattended-upgrades bật hằng ngày', root("apt-config dump APT::Periodic::Unattended-Upgrade | cut -d'\"' -f2") === '1');
check('không tự khởi động lại, nginx ngoài danh sách tự vá', /Automatic-Reboot "false"/.test(root('cat /etc/apt/apt.conf.d/52unattended-toinoi')) && /"nginx"/.test(root('cat /etc/apt/apt.conf.d/52unattended-toinoi')));
check('fail2ban đang chạy, jail sshd bật', root('systemctl is-active fail2ban') === 'active' && /Status for the jail: sshd/.test(root('fail2ban-client status sshd')));
const jail = root('fail2ban-client get sshd ignoreip');
check('fail2ban không chặn 10.20.0.1 (đường vào của An) và bastion', /10\.20\.0\.1\b/.test(jail) && /10\.20\.0\.10/.test(jail), jail);
check('fail2ban chặn bằng ufw', /ufw/.test(root('fail2ban-client get sshd actions')));

// 3. Ansible từ app01
const ping = asAn('toinoi-app-01', 'cd ~/van-hanh && ansible all -i hosts.ini -m ping -o');
check('ansible ping tới app02 và bastion', /toinoi-app-02 \| SUCCESS/.test(ping.out) && /toinoi-bastion \| SUCCESS/.test(ping.out), ping.out);
const chk = asAn('toinoi-app-01', 'cd ~/van-hanh && ansible-playbook -i hosts.ini ssh-toinoi.yml --become-password-file ~/.config/ansible-become --check');
check('playbook đã áp dụng (chạy --check không còn gì thay đổi)', chk.status === 0 && /changed=0/.test(chk.out) && !/changed=[1-9]/.test(chk.out), chk.out.split('\n').slice(-4).join(' | '));
for (const m of ['toinoi-app-02', 'toinoi-bastion']) {
  check(`${m}: MaxAuthTries 3 do Ansible đặt`, /^maxauthtries 3$/m.test(on(m, 'sshd -T').out));
}

// 4. Triển khai
check('current trỏ tới releases/1.6.0', root('readlink /srv/toinoi/bao-gia/current') === 'releases/1.6.0');
const v = on('toinoi-app-01', 'curl -s --cacert /etc/ssl/certs/bao-gia.toinoi.lab.crt https://bao-gia.toinoi.lab/suc-khoe').out;
check('bao-gia 1.6.0 trả lời qua HTTPS', /"phien_ban": "1\.6\.0"/.test(v), v);
const sai = asAn('toinoi-app-01', 'trien-khai 9.9.9');
check('trien-khai từ chối release không tồn tại', sai.status !== 0 && /khong co release/.test(sai.out), sai.out);

// 5. Sau các sự cố
check('/du-lieu dưới 80% chỗ và inode', (() => {
  const [d, i] = root("df --output=pcent,ipcent /du-lieu | tail -n 1 | tr -d '%'").trim().split(/\s+/).map(Number);
  return d < 80 && i < 80;
})());
check('sao-luu-dem chạy thành công', root('systemctl is-failed sao-luu-dem || true') !== 'failed');
check('app02: authorized_keys của an quyền 600', on('toinoi-app-02', 'stat -c %a /home/an/.ssh/authorized_keys').out === '600');
const fpMoi = root('ssh-keygen -lf /usr/share/toinoi-lab/p06/ssh_host_ed25519_key_app-02-moi.pub').split(' ')[1];
const kh = asAn('toinoi-app-01', 'ssh-keygen -lF toinoi-app-02').out;
check('app01 đã tin host key mới của app02', kh.includes(fpMoi), kh);

// 6. Runbook và script kiểm tra
const rb = root('cat /srv/toinoi/runbook/van-hanh.md 2>/dev/null || true');
check('runbook có đủ các mục chính', ['Vào server', 'bao-gia không trả lời', 'Đĩa', 'Không SSH được', 'Cập nhật'].every((s) => rb.includes(s)));
const kt = asAn('toinoi-app-01', `echo toinoi-lab | sudo -S -v 2>/dev/null; kiem-tra-van-hanh`);
check('kiem-tra-van-hanh: 0 mục LOI', kt.status === 0 && /---- 0 muc LOI/.test(kt.out), kt.out.split('\n').filter((l) => /^LOI/.test(l)).join(' | '));

console.log(`\n${pass} PASS, ${fail} FAIL`);
process.exit(fail ? 1 : 0);
