#!/usr/bin/env node
// Runner của lab Cassandra · Tới Nơi. Không có dependency npm.
//
//   node run.mjs p01                 chạy mọi file p01/NN-*.cql|.sh theo thứ tự tên
//   node run.mjs p01/00-reset.cql …  chạy đúng các file được chỉ định
//
// File .cql chạy bằng cqlsh BÊN TRONG container node1 (một phiên cqlsh cho mỗi file).
// File .sh chạy bằng sh trong container node1 (nodetool, sstablemetadata…).
//
// Quy ước trong file .cql:
//   -- ## Tiêu đề         in tiêu đề mục vào kết quả (runner chèn một dòng mốc rồi thay bằng tiêu đề)
//   -- EXPECT: <chuỗi>    câu lệnh ngay sau PHẢI lỗi và thông điệp lỗi chứa <chuỗi>  → PASS/FAIL
// Mọi lỗi không được khai báo EXPECT đều tính là FAIL.
import { spawnSync } from 'node:child_process';
import { mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.dirname(fileURLToPath(import.meta.url));
const NODE = process.env.LAB_NODE ?? 'node1';
const PORT = process.env.LAB_PORT ?? '19042';
const env = { ...process.env, MSYS_NO_PATHCONV: '1' };

function listFiles(args) {
  const out = [];
  for (const a of args) {
    const abs = path.join(root, a);
    if (statSync(abs).isDirectory()) {
      out.push(
        ...readdirSync(abs)
          .filter((f) => /^\d\d-.*\.(cql|sh)$/.test(f) && !f.includes('rollback'))
          .sort()
          .map((f) => path.posix.join(a.replaceAll('\\', '/'), f)),
      );
    } else out.push(a.replaceAll('\\', '/'));
  }
  return out;
}

/** Tách file thành câu lệnh; ghi lại dòng kết thúc, tiêu đề và EXPECT đứng trước mỗi câu. */
function parse(text) {
  const lines = text.split(/\r?\n/);
  const out = []; // dòng của file sẽ chạy
  const marks = new Map(); // dòng kết thúc (trong file chạy) -> { kind, value }
  let pendingExpect = null;
  let inStmt = false;
  let inString = false;
  let inBatch = false;
  for (const line of lines) {
    const trimmed = line.trim();
    if (!inStmt && trimmed.startsWith('-- ## ')) {
      out.push(`@@MARK;`);
      marks.set(out.length, { kind: 'title', value: trimmed.slice(6) });
      continue;
    }
    if (!inStmt && trimmed.startsWith('-- EXPECT:')) {
      pendingExpect = trimmed.slice(10).trim();
      out.push('');
      continue;
    }
    out.push(line);
    if (!inStmt && (trimmed === '' || trimmed.startsWith('--'))) continue;
    if (!inStmt) inBatch = /^BEGIN\s+(UNLOGGED\s+|COUNTER\s+)?BATCH/i.test(trimmed);
    inStmt = true;
    // đếm dấu nháy đơn để không coi ';' trong chuỗi là kết thúc câu
    for (const ch of line) if (ch === "'") inString = !inString;
    const code = line.replace(/--.*$/, '').trimEnd();
    if (!inString && code.endsWith(';') && (!inBatch || /APPLY\s+BATCH\s*;$/i.test(code))) {
      inStmt = false;
      if (pendingExpect) marks.set(out.length, { kind: 'expect', value: pendingExpect });
      else marks.set(out.length, { kind: 'stmt' });
      pendingExpect = null;
    }
  }
  return { body: out.join('\n') + '\n', marks };
}

const counts = { files: 0, pass: 0, fail: 0 };

function runCql(file) {
  const { body, marks } = parse(readFileSync(path.join(root, file), 'utf8'));
  const runDir = path.join(root, '.run');
  mkdirSync(runDir, { recursive: true });
  const tmp = `${file.replaceAll('/', '__')}`;
  writeFileSync(path.join(runDir, tmp), body);
  const inContainer = `/lab/.run/${tmp}`;
  const r = spawnSync(
    'docker',
    // stdout và stderr gộp TRONG container để thông báo lỗi nằm đúng chỗ giữa các kết quả
    [
      'compose', 'exec', '-T', '-e', 'PYTHONUNBUFFERED=1', NODE, 'sh', '-c',
      `cqlsh 127.0.0.1 ${PORT} --no-color --request-timeout=60 -f ${inContainer} 2>&1`,
    ],
    { cwd: root, env, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
  );
  const raw = `${r.stdout}${r.stderr}`; // stderr chỉ còn lỗi của docker
  const errRe = new RegExp(`^${inContainer.replaceAll('.', '\\.')}:(\\d+):(.*)$`);
  const seen = new Set();
  const lines = [];
  for (const line of raw.split(/\r?\n/)) {
    const m = errRe.exec(line);
    if (!m) {
      lines.push(line);
      continue;
    }
    const end = Number(m[1]) - 1; // cqlsh báo dòng ngay sau câu lệnh
    const msg = m[2];
    const mark = marks.get(end);
    if (seen.has(end)) continue; // lỗi cú pháp in nhiều dòng cùng số dòng
    seen.add(end);
    if (mark?.kind === 'title') {
      lines.push(`-- ## ${mark.value}`);
    } else if (mark?.kind === 'expect') {
      const ok = msg.includes(mark.value);
      counts[ok ? 'pass' : 'fail']++;
      lines.push(`${ok ? 'PASS' : 'FAIL'} lỗi mong đợi (dòng ${end}): ${msg.replace(/^.*?message="?/, '').replace(/"$/, '')}`);
    } else {
      counts.fail++;
      lines.push(`FAIL lỗi không mong đợi (dòng ${end}): ${msg}`);
    }
  }
  for (const [end, mark] of marks) {
    if (mark.kind === 'expect' && !seen.has(end)) {
      counts.fail++;
      lines.push(`FAIL câu lệnh dòng ${end} lẽ ra phải lỗi "${mark.value}" nhưng đã chạy được`);
    }
  }
  return lines
    .join('\n')
    .replace(/\n{3,}/g, '\n\n')
    .trim();
}

function runSh(file) {
  const r = spawnSync('docker', ['compose', 'exec', '-T', NODE, 'sh', `/lab/${file}`], {
    cwd: root,
    env,
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  });
  if (r.status !== 0) counts.fail++;
  return `${r.stdout}${r.stderr}`.trim() + (r.status !== 0 ? `\nFAIL script thoát với mã ${r.status}` : '');
}

const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('Cách dùng: node run.mjs p01 | node run.mjs p01/00-reset.cql …');
  process.exit(2);
}
for (const file of listFiles(args)) {
  counts.files++;
  console.log(`\n==== ${file} ====`);
  console.log(file.endsWith('.sh') ? runSh(file) : runCql(file));
}
console.log(`\n-- [lab] File đã chạy: ${counts.files}`);
console.log(`-- [lab] PASS: ${counts.pass}`);
console.log(`-- [lab] FAIL: ${counts.fail}`);
console.log(`-- [lab] KẾT QUẢ: ${counts.fail === 0 ? 'ĐẠT' : 'CHƯA ĐẠT'}`);
process.exit(counts.fail === 0 ? 0 : 1);
