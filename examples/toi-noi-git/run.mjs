#!/usr/bin/env node
// Runner của lab "Git qua GitStudio và Tới Nơi".
//
//   node run.mjs p01            # chạy mọi bài p01/NN-*.mjs theo thứ tự, in lệnh và kết quả
//   node run.mjs p01 05         # chỉ chạy bài GIT-005
//   node run.mjs setup 05       # dựng work/bao-gia ở trạng thái ĐẦU bài GIT-005 để mở bằng GitStudio
//   node run.mjs setup 05 --end # … hoặc ở trạng thái CUỐI bài
//
// Mỗi bài chạy trong thư mục work/ dựng lại từ đầu, nên chạy bao nhiêu lần cũng ra cùng kết quả.
// Kết quả mẫu: p01/OUTPUT.txt. Cuối cùng runner in bảng đếm PASS/FAIL.

import { readdirSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

import { build, expectedHashes } from './lib/history.mjs';
import { LAB, Session, WORK, freshDir } from './lib/lab.mjs';

async function loadPart(part) {
  const dir = join(LAB, part);
  const files = readdirSync(dir)
    .filter((f) => /^\d\d-.*\.mjs$/.test(f))
    .sort();
  const out = [];
  for (const f of files) {
    const mod = await import(pathToFileURL(join(dir, f)).href);
    out.push({ file: `${part}/${f}`, num: f.slice(0, 2), ...mod.default });
  }
  return out;
}

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('Cách dùng: node run.mjs p01 [NN] | node run.mjs setup NN [--end]');
  process.exit(2);
}

// Mã hash chuẩn tính trong một thư mục tạm riêng, trước khi bài nào chạy.
const hashes = expectedHashes(join(WORK, '.chuan'));

if (args[0] === 'setup') {
  const num = (args[1] ?? '').padStart(2, '0');
  const end = args.includes('--end');
  const all = await loadPart('p01');
  const art = all.find((a) => a.num === num);
  if (!art) {
    console.error(`Không có bài số ${num} trong p01`);
    process.exit(2);
  }
  const n = end ? art.end : art.start;
  if (n == null) {
    console.error(`${art.id} không dùng repository bao-gia dựng sẵn.`);
    process.exit(2);
  }
  const dir = join(WORK, 'bao-gia');
  build(dir, n);
  if (!end && art.prepare) art.prepare(dir);
  console.log(`Đã dựng ${dir} ở trạng thái ${end ? 'cuối' : 'đầu'} bài ${art.id} (${n} commit).`);
  console.log('Mở thư mục này bằng GitStudio: Mở repository… hoặc chuột phải thư mục → Open with GitStudio.');
  process.exit(0);
}

const part = args[0];
const only = args[1]?.padStart(2, '0');
const articles = (await loadPart(part)).filter((a) => !only || a.num === only);

let pass = 0;
let fail = 0;
for (const art of articles) {
  console.log(`\n# ==================== ${art.file} · ${art.id} ====================`);
  freshDir(WORK);
  const s = new Session();
  if (art.start != null) {
    build(join(WORK, 'bao-gia'), art.start);
    s.cd('bao-gia', { show: false });
  }
  try {
    await art.run(s, { hashes });
  } catch (e) {
    s.fail++;
    console.log(`FAIL ${art.id} dừng giữa chừng: ${e.message}`);
  }
  pass += s.pass;
  fail += s.fail;
}
// Dọn thư mục tạm của lịch sử chuẩn; để lại work/ của bài cuối để xem.
rmSync(join(WORK, '.chuan'), { recursive: true, force: true });

console.log(`\n# [lab] Bài đã chạy: ${articles.length}`);
console.log(`# [lab] PASS: ${pass}`);
console.log(`# [lab] FAIL: ${fail}`);
console.log(`# [lab] KẾT QUẢ: ${fail === 0 ? 'ĐẠT' : 'KHÔNG ĐẠT'}`);
process.exitCode = fail === 0 ? 0 : 1;
