// Chạy toàn bộ lab Phần 1 và so với kết quả mẫu p01/OUTPUT.txt.
// Kết quả phải giống từng ký tự: git chạy với cấu hình cô lập, danh tính và thời điểm cố định.
// Bản git khác có thể đổi câu chữ của vài thông báo; khi đó xem diff rồi cập nhật OUTPUT.txt.

import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

test('lab p01 chạy đạt và khớp OUTPUT.txt', () => {
  const r = spawnSync(process.execPath, ['run.mjs', 'p01'], { cwd: root, encoding: 'utf8' });
  assert.equal(r.status, 0, r.stdout + r.stderr);
  assert.match(r.stdout, /# \[lab\] FAIL: 0/);
  const expected = readFileSync(join(root, 'p01', 'OUTPUT.txt'), 'utf8').replace(/\r\n/g, '\n');
  assert.equal(r.stdout.replace(/\r\n/g, '\n'), expected);
});

test('setup dựng đúng trạng thái đầu bài GIT-005', () => {
  const r = spawnSync(process.execPath, ['run.mjs', 'setup', '05'], { cwd: root, encoding: 'utf8' });
  assert.equal(r.status, 0, r.stderr);
  const st = spawnSync('git', ['status', '--porcelain'], { cwd: join(root, 'work', 'bao-gia'), encoding: 'utf8' });
  assert.equal(st.stdout.trim(), 'M src/BaoGia/FareCalculator.cs');
});
