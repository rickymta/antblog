// Kiểm lab P01 trên MongoDB 8.0 thật (container `db` phải đang chạy:
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
const normalize = (s) => s.replace(/\r\n/g, '\n').split('\n').filter((l) => l !== 'Đã xoá database toinoi cũ').join('\n').trim();

test('thí nghiệm MG-004 đạt', () => {
  const { status, out } = run('p01/00-reset.js', 'p01/thu-nghiem/mg004-kieu-bson.js');
  assert.equal(status, 0, out);
  assert.match(out, /double\|0\.30000000000000004\|false/);
  assert.match(out, /\/\/ \[lab\] PASS: 13\n\/\/ \[lab\] FAIL: 0/);
});

test('p01 chạy đủ 8 file, 115 PASS, 0 FAIL', () => {
  const { status, out } = run('p01');
  assert.equal(status, 0, out);
  assert.match(out, /\/\/ \[lab\] File đã chạy: 8\n\/\/ \[lab\] PASS: 115\n\/\/ \[lab\] FAIL: 0\n\/\/ \[lab\] KẾT QUẢ: ĐẠT/);
  assert.equal(normalize(out), normalize(readFileSync(join(root, 'p01/OUTPUT.txt'), 'utf8')));
});

test('rollback của 06 chạy lặp được hai lần', () => {
  run('p01');
  for (let i = 0; i < 2; i++) {
    const { status, out } = run('p01/06-migration-rollback.js');
    assert.equal(status, 0, out);
    assert.match(out, /PASS R1 còn tài xế có vehicle_plate = 0/);
    assert.match(out, /PASS R2 số biển số đã sao lưu = 8/);
  }
});

// Chạy cuối để database toinoi giữ trạng thái đầy đủ sau 00 → 07 (DBStudio, bài viết).
test('trả database về trạng thái sau p01', () => {
  assert.equal(run('p01').status, 0);
});
