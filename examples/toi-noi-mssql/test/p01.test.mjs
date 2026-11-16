// Kiểm lab P01 trên SQL Server 2022 thật (container `db` phải đang chạy:
// `npm run db:up`). Chạy lại toàn bộ p01 và so với OUTPUT.txt đã commit.
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
// Dòng này chỉ xuất hiện khi database đã có từ lần chạy trước.
const normalize = (s) => s.replace(/\r\n/g, '\n').split('\n').filter((l) => l !== 'Đã xoá database ToiNoi cũ').join('\n').trim();

test('thí nghiệm MS-004 đạt', () => {
  const { status, out } = run('p01/00-reset.sql', 'p01/thu-nghiem/ms004-kieu-du-lieu.sql');
  assert.equal(status, 0, out);
  assert.match(out, /Nguy\?n Văn An — Đà N\?ng\|Nguyễn Văn An — Đà Nẵng/);
  assert.match(out, /-- \[lab\] PASS: 3\n-- \[lab\] FAIL: 0/);
});

test('p01 chạy đủ 8 file, 80 PASS, 0 FAIL', () => {
  const { status, out } = run('p01');
  assert.equal(status, 0, out);
  assert.match(out, /-- \[lab\] File đã chạy: 8\n-- \[lab\] PASS: 80\n-- \[lab\] FAIL: 0\n-- \[lab\] KẾT QUẢ: ĐẠT/);
  assert.equal(normalize(out), normalize(readFileSync(join(root, 'p01/OUTPUT.txt'), 'utf8')));
});

test('rollback của 06 chạy lặp được hai lần', () => {
  run('p01');
  for (let i = 0; i < 2; i++) {
    const { status, out } = run('p01/06-migration-rollback.sql');
    assert.equal(status, 0, out);
    assert.match(out, /PASS R1 cột vehicle_plate = không/);
    assert.match(out, /PASS R2 số biển số đã sao lưu = 7/);
  }
});
