// Chạy lại toàn bộ P01 trên keyspace mới và so từng dòng với p01/OUTPUT.txt.
// Chỉ bỏ qua những gì phụ thuộc máy và thời điểm chạy: thời gian của COPY FROM, độ trễ và các ước lượng
// kích thước trong nodetool (histogram theo bucket, phụ thuộc lần flush/compaction).
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');

function normalize(text) {
  let inSh = false;
  return text
    .split(/\r?\n/)
    .map((line) => {
      if (line.startsWith('==== ')) inSh = line.endsWith('.sh ====');
      if (/rows imported from/.test(line)) return line.replace(/in .* seconds/, 'in … seconds');
      if (inSh && /^(\d+%|Max|Number of partitions|Compacted partition)/.test(line)) return line.replace(/\d+(\.\d+)?/g, '#').replace(/\s+/g, ' ');
      return line.trimEnd();
    })
    .join('\n')
    .trim();
}

test('P01 chạy lại cho đúng kết quả mẫu', { timeout: 600_000 }, () => {
  const r = spawnSync(process.execPath, ['run.mjs', 'p01'], { cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
  const expected = readFileSync(path.join(root, 'p01/OUTPUT.txt'), 'utf8');
  assert.match(r.stdout, /-- \[lab\] FAIL: 0/);
  assert.equal(r.status, 0);
  const a = normalize(r.stdout).split('\n');
  const b = normalize(expected).split('\n');
  for (let i = 0; i < Math.max(a.length, b.length); i++) assert.equal(a[i], b[i], `khác ở dòng ${i + 1}`);
});
