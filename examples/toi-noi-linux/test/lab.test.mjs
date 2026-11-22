// Test của runner: không cần Docker. Chạy: npm test
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
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
