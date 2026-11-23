#!/usr/bin/env node
// Runner của lab "Git qua GitStudio và Tới Nơi".
//
//   node run.mjs p01            # chạy mọi bài p01/NN-*.mjs theo thứ tự, in lệnh và kết quả
//   node run.mjs p01 05         # chỉ chạy bài GIT-005
//   node run.mjs p02 14         # chỉ chạy bài GIT-014
//   node run.mjs setup 05       # dựng work/bao-gia ở trạng thái ĐẦU bài GIT-005 để mở bằng GitStudio
//   node run.mjs setup 14 --end # … hoặc ở trạng thái CUỐI bài
//
// Mỗi bài chạy trong thư mục work/ dựng lại từ đầu, nên chạy bao nhiêu lần cũng ra cùng kết quả.
// Kết quả mẫu: p01/OUTPUT.txt, p02/OUTPUT.txt. Cuối cùng runner in bảng đếm PASS/FAIL.

import { readdirSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

import { build, expectedHashes } from './lib/history.mjs';
import { build2, expectedRefs2 } from './lib/history2.mjs';
import { build3, expectedRefs3 } from './lib/history3.mjs';
import { LAB, Session, WORK, freshDir } from './lib/lab.mjs';

/** Cách dựng repository dựng sẵn cho từng phần: n = số commit (P01) hoặc số bước (P02). */
const BUILDERS = { p01: build, p02: build2, p03: build3 };
/** Ref chuẩn sau n bước của từng phần (dựng trong thư mục tạm riêng). */
const EXPECTED = {
  p02: (n) => expectedRefs2(join(WORK, '.chuan2'), n),
  p03: (n) => expectedRefs3(join(WORK, '.chuan3', 'bao-gia'), n),
};
const PARTS = Object.keys(BUILDERS);

async function loadPart(part) {
  const dir = join(LAB, part);
  const files = readdirSync(dir)
    .filter((f) => /^\d\d-.*\.mjs$/.test(f))
    .sort();
  const out = [];
  for (const f of files) {
    const mod = await import(pathToFileURL(join(dir, f)).href);
    out.push({ part, file: `${part}/${f}`, num: f.slice(0, 2), ...mod.default });
  }
  return out;
}

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('Cách dùng: node run.mjs p01|p02 [NN] | node run.mjs setup NN [--end]');
  process.exit(2);
}

// Mã hash chuẩn tính trong thư mục tạm riêng, trước khi bài nào chạy.
const hashes = expectedHashes(join(WORK, '.chuan'));

if (args[0] === 'setup') {
  const num = (args[1] ?? '').padStart(2, '0');
  const end = args.includes('--end');
  const all = (await Promise.all(PARTS.map(loadPart))).flat();
  const art = all.find((a) => a.num === num);
  if (!art) {
    console.error(`Không có bài số ${num}`);
    process.exit(2);
  }
  const n = end ? art.end : art.start;
  if (n == null) {
    console.error(`${art.id} không dùng repository bao-gia dựng sẵn.`);
    process.exit(2);
  }
  const dir = join(WORK, 'bao-gia');
  BUILDERS[art.part](dir, n);
  if (!end && art.prepare) art.prepare(dir);
  rmSync(join(WORK, '.chuan'), { recursive: true, force: true });
  console.log(`Đã dựng ${dir} ở trạng thái ${end ? 'cuối' : 'đầu'} bài ${art.id}.`);
  console.log('Mở thư mục này bằng GitStudio: Mở repository… hoặc chuột phải thư mục → Open with GitStudio.');
  process.exit(0);
}

const part = args[0];
if (!BUILDERS[part]) {
  console.error(`Không có phần ${part}`);
  process.exit(2);
}
const only = args[1]?.padStart(2, '0');
const articles = (await loadPart(part)).filter((a) => !only || a.num === only);

// Ref chuẩn của Phần 2 tính trước cho mọi mốc bài cần (tránh dựng giữa lúc bài đang chạy).
const refsAt = new Map();
if (EXPECTED[part]) {
  for (const a of articles) {
    for (const n of [a.start, a.end]) {
      if (n != null && !refsAt.has(n)) refsAt.set(n, EXPECTED[part](n));
    }
  }
  for (const d of ['.chuan2', '.chuan3']) rmSync(join(WORK, d), { recursive: true, force: true });
}

let pass = 0;
let fail = 0;
for (const art of articles) {
  console.log(`\n# ==================== ${art.file} · ${art.id} ====================`);
  freshDir(WORK);
  const s = new Session();
  if (art.start != null) {
    BUILDERS[part](join(WORK, 'bao-gia'), art.start);
    s.cd('bao-gia', { show: false });
  }
  try {
    await art.run(s, { hashes, refs2: (n) => refsAt.get(n), refs3: (n) => refsAt.get(n) });
  } catch (e) {
    s.fail++;
    console.log(`FAIL ${art.id} dừng giữa chừng: ${e.message}`);
  }
  pass += s.pass;
  fail += s.fail;
}
rmSync(join(WORK, '.chuan'), { recursive: true, force: true });

console.log(`\n# [lab] Bài đã chạy: ${articles.length}`);
console.log(`# [lab] PASS: ${pass}`);
console.log(`# [lab] FAIL: ${fail}`);
console.log(`# [lab] KẾT QUẢ: ${fail === 0 ? 'ĐẠT' : 'KHÔNG ĐẠT'}`);
process.exitCode = fail === 0 ? 0 : 1;
