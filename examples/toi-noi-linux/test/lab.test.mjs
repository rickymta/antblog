// Test của runner: không cần Docker. Chạy: npm test
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

import { ROOT, compile, scripts } from '../lib/lab.mjs';

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
