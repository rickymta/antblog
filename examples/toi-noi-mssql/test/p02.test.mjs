// Kiểm lab P02 trên SQL Server 2022 thật (container `db` phải đang chạy).
// Chạy lại toàn bộ p02 (trên lược đồ P01) và so với p02/OUTPUT.txt đã commit.
// Chạy SAU test/p01.test.mjs (node --test chạy file theo thứ tự tên) nên
// database ToiNoi kết thúc ở trạng thái của phần mới nhất.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const run = (...args) => {
  const r = spawnSync(process.execPath, ['run.mjs', ...args], { cwd: root, encoding: 'utf8' });
  return { status: r.status, out: r.stdout.replace(/\r\n/g, '\n') };
};
const normalize = (s) => s.replace(/\r\n/g, '\n').split('\n').filter((l) => l !== 'Đã xoá database ToiNoi cũ').join('\n').trim();

test('p02 chạy đủ, 0 FAIL, khớp OUTPUT.txt', () => {
  const { status, out } = run('p01/00-reset.sql', 'p01/01-schema.sql', 'p02');
  assert.equal(status, 0, out);
  assert.match(out, /-- \[lab\] FAIL: 0\n-- \[lab\] KẾT QUẢ: ĐẠT/);
  assert.equal(normalize(out), normalize(readFileSync(join(root, 'p02/OUTPUT.txt'), 'utf8')));
});

test('p02 không chạy được khi thiếu lược đồ P01', () => {
  run('p01/00-reset.sql');
  const { status, out } = run('p02/00-mo-rong-schema.sql');
  assert.notEqual(status, 0);
  assert.match(out, /Chưa có lược đồ P01/);
  assert.equal(run('p01/00-reset.sql', 'p01/01-schema.sql', 'p02').status, 0);
});
