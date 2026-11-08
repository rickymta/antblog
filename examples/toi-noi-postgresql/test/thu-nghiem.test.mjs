// Các thí nghiệm SQL trích từ bài PG-001…PG-010 (p01/thu-nghiem): mỗi tệp chạy trên đúng nền dữ liệu ghi trong manifest
// và phải ĐẠT (không lỗi ngoài kỳ vọng, mọi expect-error/expect-rows khớp).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const manifest = JSON.parse(readFileSync(path.join(root, 'p01/thu-nghiem/manifest.json'), 'utf8'));

for (const entry of manifest.tep) {
  test(`${entry.bai} · ${entry.file} (nền ${entry.nen})`, () => {
    const base = manifest.nen[entry.nen];
    assert.ok(base, `nền không có trong manifest: ${entry.nen}`);
    const r = spawnSync(process.execPath, ['run.mjs', '--no-echo', ...base, `p01/thu-nghiem/${entry.file}`], { cwd: root, encoding: 'utf8' });
    assert.equal(r.status, 0, r.stdout.split('\n').filter((l) => l.includes('FAIL')).join('\n') || r.stderr);
  });
}
