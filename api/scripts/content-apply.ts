/**
 * `npm run content:apply -- <edits.json | thư-mục>... [--dry-run]`
 * Áp các đề xuất sửa câu chữ vào `seed/content/*.md`. Mỗi tệp JSON là một `EditFile` ({ file, edits[] })
 * hoặc một mảng `EditFile`. Đoạn nào không qua chốt an toàn (seed/contentEdits.ts) bị từ chối và in lý do.
 */
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { applyEdits, type EditFile } from '../seed/contentEdits.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const contentDir = path.join(root, 'seed/content');
const args = process.argv.slice(2);
const dryRun = args.includes('--dry-run');
// Mỗi đối số là một tệp JSON hoặc một thư mục (lấy mọi *.json bên trong — tránh giới hạn độ dài dòng lệnh trên Windows).
const inputs = args
  .filter((a) => !a.startsWith('--'))
  .flatMap((a) => {
    const p = path.resolve(a);
    return statSync(p).isDirectory()
      ? readdirSync(p).filter((f) => f.endsWith('.json')).sort().map((f) => path.join(p, f))
      : [p];
  });
if (inputs.length === 0) throw new Error('Cần ít nhất một tệp edits JSON hoặc thư mục chứa chúng.');

let applied = 0;
let rejected = 0;
for (const input of inputs) {
  const parsed = JSON.parse(readFileSync(path.resolve(input), 'utf8')) as EditFile | EditFile[];
  for (const ef of Array.isArray(parsed) ? parsed : [parsed]) {
    const name = path.basename(ef.file);
    if (!/^[a-z0-9-]+\.md$/.test(name)) throw new Error(`Tên tệp không hợp lệ: ${ef.file}`);
    const file = path.join(contentDir, name);
    const r = applyEdits(readFileSync(file, 'utf8'), ef.edits);
    applied += r.applied.length;
    rejected += r.rejected.length;
    console.log(`${name}: áp ${r.applied.length}, từ chối ${r.rejected.length}${r.fileError ? ` — LỖI TỆP: ${r.fileError}` : ''}`);
    for (const x of r.rejected) console.log(`  ✗ ${x.why}: "${x.edit.old.slice(0, 80)}"`);
    if (!dryRun && r.applied.length) writeFileSync(file, r.output, 'utf8');
  }
}
console.log(`\nTổng: áp ${applied}, từ chối ${rejected}${dryRun ? ' (dry-run, chưa ghi)' : ''}`);
