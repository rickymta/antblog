// Test của runner: không cần Docker. Chạy: npm test
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT, compile, containerOf, scripts, segments } from '../lib/lab.mjs';

test('Phần 1 có đủ 10 kịch bản, đánh số liên tục', () => {
  const ids = scripts('p01').map((s) => s.id);
  assert.deepEqual(ids, ['01', '02', '03', '04', '05', '06', '07', '08', '09', '10']);
});

test('compile in lại lệnh kèm dấu nhắc và giữ mã thoát', () => {
  const sh = compile('## Tiêu đề\n# ghi chú\nls /khong-co\necho $?\n');
  assert.match(sh, /printf '\\n# %s\\n' 'Tiêu đề'/);
  assert.match(sh, /__p 'ls \/khong-co'\nls \/khong-co\n__p 'echo \$\?'\necho \$\?/);
  assert.match(sh, /local s=\$\?/);
  assert.doesNotMatch(sh, /ghi chú/);
});

test('dòng #@ chạy nhưng không in', () => {
  const sh = compile("#@ touch a.txt\nls\n");
  assert.match(sh, /\ntouch a\.txt\n/);
  assert.doesNotMatch(sh, /__p 'touch a\.txt'/);
});

test('dấu nháy đơn trong lệnh được thoát đúng', () => {
  const sh = compile("grep -o 'dich_vu=[a-z0-9]*' f\n");
  assert.ok(sh.includes(`__p 'grep -o '\\''dich_vu=[a-z0-9]*'\\'' f'`));
});

test('heredoc và dòng nối bằng \\ được giữ thành một lệnh', () => {
  const sh = compile("cat > a.txt <<'EOF'\nmot\nEOF\nls \\\n  -l\n");
  assert.ok(sh.includes("cat > a.txt <<'EOF'\nmot\nEOF\n"));
  assert.ok(sh.includes('ls \\\n  -l\n'));
});

test('chỉ kịch bản có sudo mới xác thực sudo trước', () => {
  assert.doesNotMatch(compile('ls\n'), /sudo -S -v/);
  assert.match(compile('ls\n'), /expand_aliases/);
  assert.match(compile('sudo ls\n'), /sudo -S -v/);
});

test('chế độ im lặng (setup) không in gì', () => {
  assert.doesNotMatch(compile('## A\nls\n', { echo: false }), /__p '/);
});

test('OUTPUT.txt có bản ghi của cả 10 bài', () => {
  const out = readFileSync(join(ROOT, 'p01', 'OUTPUT.txt'), 'utf8');
  for (const s of scripts('p01')) assert.ok(out.includes(`==== p01/${s.name} ====`), s.name);
  assert.ok(!out.includes('\t'), 'không còn ký tự tab');
  assert.ok(!out.includes('/run/lab.sh'), 'không lộ tên file kịch bản tạm');
});

test('Phần 2 có đủ 10 kịch bản, đánh số tiếp từ 11', () => {
  assert.deepEqual(
    scripts('p02').map((s) => s.id),
    ['11', '12', '13', '14', '15', '16', '17', '18', '19', '20'],
  );
});

test('#root chạy bằng sudo không in, và bật xác thực sudo trước', () => {
  const sh = compile('#root /usr/share/x.sh\nls\n');
  assert.match(sh, /sudo -S -v/);
  assert.match(sh, /sudo -n bash -c '\/usr\/share\/x\.sh' >\/dev\/null/);
  assert.doesNotMatch(sh, /__p '\/usr\/share/);
});

test('#file in "vim ĐƯỜNG" rồi ghi nội dung; #check ghi mà không in', () => {
  const dir = join(ROOT, 'p02');
  const sh = compile('#file bin/xin-chao xin-chao\n#check ~/x xin-chao\n', { dir });
  const body = readFileSync(join(dir, 'files', 'xin-chao')).toString('base64');
  assert.match(sh, /__p 'vim bin\/xin-chao'\nbase64 -d > bin\/xin-chao <<'__B64__'/);
  assert.ok(sh.includes(body.slice(0, 40)));
  assert.equal((sh.match(/__p 'vim/g) || []).length, 1);
  assert.match(sh, /base64 -d > ~\/x <<'__B64__'/);
});

test('#wait chờ, source ~/.bashrc nạp .bashrc như phiên tương tác', () => {
  const sh = compile('#wait 65\nsource ~/.bashrc\n');
  assert.match(sh, /\nsleep 65\n/);
  assert.match(sh, /__p 'source ~\/.bashrc'\n__rc\n/);
});

test('mọi file được #file/#check nhắc tới đều có trong p02/files', () => {
  for (const s of scripts('p02')) {
    for (const m of readFileSync(s.path, 'utf8').matchAll(/^#(?:file|check) \S+ (\S+)(?: \d\d)?$/gm)) {
      assert.ok(existsSync(join(ROOT, 'p02', 'files', m[1])), `${s.name}: thiếu files/${m[1]}`);
    }
  }
});

test('OUTPUT.txt của Phần 2 có bản ghi của cả 10 bài', () => {
  const out = readFileSync(join(ROOT, 'p02', 'OUTPUT.txt'), 'utf8');
  for (const s of scripts('p02')) assert.ok(out.includes(`==== p02/${s.name} ====`), s.name);
  assert.ok(!out.includes('/run/lab.sh'), 'không lộ tên file kịch bản tạm');
});

test('Phần 3 có đủ 10 kịch bản, đánh số tiếp từ 21', () => {
  assert.deepEqual(
    scripts('p03').map((s) => s.id),
    ['21', '22', '23', '24', '25', '26', '27', '28', '29', '30'],
  );
});

test('#sudofile in lệnh visudo/sudoedit rồi ghi file hệ thống bằng sudo', () => {
  const dir = join(ROOT, 'p03');
  const v = compile('#sudofile visudo /etc/sudoers.d/cuong sudoers-cuong 02\n', { dir });
  assert.match(v, /sudo -S -v/);
  assert.match(v, /__p 'sudo visudo -f \/etc\/sudoers\.d\/cuong'/);
  assert.match(v, /sudo -n tee \/etc\/sudoers\.d\/cuong/);
  // File mới do visudo -f tạo có quyền 0640 như thật; bài tự chmod 440.
  assert.match(v, /chmod 0640 \/etc\/sudoers\.d\/cuong/);
  const e = compile('#sudofile sudoedit /etc/pam.d/sshd pam-sshd\n', { dir });
  assert.match(e, /__p 'sudoedit \/etc\/pam\.d\/sshd'/);
  assert.match(e, /chmod 0644 \/etc\/pam\.d\/sshd/);
});

test('#login đăng nhập SSH thật bằng key của lab, không in; otp dùng askpass', () => {
  const sh = compile('#login an an-laptop otp\n#login root an-laptop\n');
  assert.match(sh, /sudo -S -v/);
  assert.doesNotMatch(sh, /__p '.*ssh -i/);
  assert.match(sh, /\/usr\/share\/toinoi-lab\/p03\/an-laptop/);
  assert.match(sh, /SSH_ASKPASS=\/usr\/share\/toinoi-lab\/p03\/askpass-otp/);
  assert.match(sh, /BatchMode=yes/);
  assert.match(sh, /root@127\.0\.0\.1/);
});

test('mọi file được #file/#check/#sudofile nhắc tới đều có trong p03/files', () => {
  for (const s of scripts('p03')) {
    const src = readFileSync(s.path, 'utf8');
    for (const m of src.matchAll(/^#(?:file|check) \S+ (\S+)(?: \d\d)?$|^#sudofile \S+ \S+ (\S+)(?: \d\d)?$/gm)) {
      const f = m[1] || m[2];
      assert.ok(existsSync(join(ROOT, 'p03', 'files', f)), `${s.name}: thiếu files/${f}`);
    }
  }
});

test('OUTPUT.txt của Phần 3 có bản ghi của cả 10 bài', () => {
  const out = readFileSync(join(ROOT, 'p03', 'OUTPUT.txt'), 'utf8');
  for (const s of scripts('p03')) assert.ok(out.includes(`==== p03/${s.name} ====`), s.name);
  assert.doesNotMatch(out, /(^|[\s'"=])toinoi-lab($|[\s'"])/m, 'không in mật khẩu của lab');
});

test('Phần 4 có đủ 10 kịch bản, đánh số tiếp từ 31', () => {
  assert.deepEqual(
    scripts('p04').map((s) => s.id),
    ['31', '32', '33', '34', '35', '36', '37', '38', '39', '40'],
  );
});

test('#ngat gửi SIGHUP cho mọi job, xoá bảng job và về thư mục home, không in gì', () => {
  const sh = compile('sleep 600 &\n#ngat\njobs\n');
  assert.match(sh, /for __j in \$\(jobs -p\); do kill -HUP "\$__j" 2>\/dev\/null; done; disown -a; cd ~/);
  assert.doesNotMatch(sh, /__p '#ngat'/);
  assert.equal((sh.match(/__p '/g) || []).length, 2);
});

test('mọi file được #file/#sudofile nhắc tới đều có trong p04/files', () => {
  for (const s of scripts('p04')) {
    const src = readFileSync(s.path, 'utf8');
    for (const m of src.matchAll(/^#(?:file|check) \S+ (\S+)(?: \d\d)?$|^#sudofile \S+ \S+ (\S+)(?: \d\d)?$/gm)) {
      const f = m[1] || m[2];
      assert.ok(existsSync(join(ROOT, 'p04', 'files', f)), `${s.name}: thiếu files/${f}`);
    }
  }
});

test('OUTPUT.txt của Phần 4 có bản ghi của cả 10 bài, không còn mã điều khiển', () => {
  const out = readFileSync(join(ROOT, 'p04', 'OUTPUT.txt'), 'utf8');
  for (const s of scripts('p04')) assert.ok(out.includes(`==== p04/${s.name} ====`), s.name);
  assert.ok(!/\x1b/.test(out), 'không còn ký tự ESC (OSC 8 của systemctl, màu)');
  assert.ok(!/^Hint: You are currently not seeing messages/m.test(out.split('==== p04/36')[1] || ''), 'từ LNX-036 an đọc được journal');
});

test('Phần 5 có đủ 10 kịch bản, đánh số tiếp từ 41', () => {
  assert.deepEqual(
    scripts('p05').map((s) => s.id),
    ['41', '42', '43', '44', '45', '46', '47', '48', '49', '50'],
  );
});

test('#may tách kịch bản thành đoạn theo server, bỏ đoạn chỉ có ghi chú', () => {
  const segs = segments('# LNX\nhostname\n#may app02\nnc -zv toinoi-app-01 22\n#may app01\n# chỉ ghi chú\n');
  assert.deepEqual(
    segs.map((s) => s.may),
    ['app01', 'app02'],
  );
  assert.ok(segs[1].lines.includes('nc -zv toinoi-app-01 22'));
  assert.equal(segments('ls\n').length, 1);
  assert.equal(containerOf('app01'), 'toinoi-app-01');
  assert.equal(containerOf('app02'), 'toinoi-app-02');
});

test('mọi file được #file/#sudofile nhắc tới đều có trong p05/files', () => {
  for (const s of scripts('p05')) {
    const src = readFileSync(s.path, 'utf8');
    for (const m of src.matchAll(/^#(?:file|check) \S+ (\S+)(?: \d\d)?$|^#sudofile \S+ \S+ (\S+)(?: \d\d)?$/gm)) {
      const f = m[1] || m[2];
      assert.ok(existsSync(join(ROOT, 'p05', 'files', f)), `${s.name}: thiếu files/${f}`);
    }
  }
});

test('OUTPUT.txt của Phần 5 có bản ghi của cả 10 bài, có dấu nhắc của toinoi-app-02', () => {
  const out = readFileSync(join(ROOT, 'p05', 'OUTPUT.txt'), 'utf8');
  for (const s of scripts('p05')) assert.ok(out.includes(`==== p05/${s.name} ====`), s.name);
  assert.ok(!/\x1b/.test(out), 'không còn ký tự ESC');
  assert.match(out, /^an@toinoi-app-02:~\$ /m);
});
