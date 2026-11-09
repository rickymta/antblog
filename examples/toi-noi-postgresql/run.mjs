#!/usr/bin/env node
// Runner của lab PostgreSQL Tới Nơi.
//
// Chạy một hoặc nhiều file .sql (hoặc cả thư mục một phần, theo thứ tự tên file)
// trên MỘT phiên PGlite mới — PostgreSQL 17.5 biên dịch sang WebAssembly, chạy
// ngay trong tiến trình Node, không cần cài server.
//
//   node run.mjs p01                       # mọi file NN-*.sql trong p01/, trừ *-rollback.sql
//   node run.mjs p01/00-reset.sql p01/01-schema.sql
//   node run.mjs p01 --persist .pgdata     # lưu dữ liệu vào thư mục để chạy tiếp lần sau
//   node run.mjs p01 --no-echo             # không in lại câu lệnh, chỉ in kết quả
//
// Kết quả in gần giống psql: bảng căn cột, "(n rows)", thẻ lệnh "INSERT 0 n",
// "UPDATE n", "ERROR:  …" kèm SQLSTATE và tên ràng buộc. Dòng bắt đầu bằng
// "-- [lab]" là chú thích của runner, psql thật không in.
//
// Quy ước kỳ vọng — dòng chú thích NGAY TRƯỚC câu lệnh:
//   -- expect-error: 23505 uq_customers_phone   câu lệnh PHẢI lỗi đúng SQLSTATE
//                                               (và đúng tên ràng buộc nếu ghi;
//                                               lỗi NOT NULL 23502 không có tên
//                                               ràng buộc ở PG 17 ⇒ ghi tên cột)
//   -- expect-rows: 0                           câu lệnh PHẢI trả đúng n dòng
//                                               (SELECT/RETURNING) hoặc đổi đúng
//                                               n dòng (UPDATE/DELETE/INSERT)
//   -- expect-value: 12345                      kết quả PHẢI là đúng 1 dòng × 1 cột
//                                               với giá trị (dạng chữ) như ghi
//   -- expect-line: 100123 | SG | 45000         (lặp nhiều dòng) TOÀN BỘ kết quả,
//                                               đúng thứ tự, mỗi chỉ thị một dòng;
//                                               các ô cách nhau bằng `|`
//   -- expect-md5: 0123…cdef                    md5 của toàn bộ kết quả (dùng cho
//                                               kết quả dài; xem canonicalResult)
// Trong expect-value / expect-line / expect-md5, NULL được viết là `NULL`
// (giống psql sau `\pset null NULL`), khoảng trắng hai đầu mỗi ô được bỏ qua.
// Lỗi khớp kỳ vọng được tính là PASS và runner chạy tiếp. Lỗi ngoài kỳ vọng
// hoặc kỳ vọng không đạt ⇒ FAIL, runner vẫn chạy hết rồi trả exit code 1.

import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { basename, join, relative } from 'node:path';
import { pathToFileURL } from 'node:url';
import { PGlite } from '@electric-sql/pglite';

/** Múi giờ phiên mặc định của lab — giống hồ sơ Tới Nơi (giờ Việt Nam). */
export const LAB_TIMEZONE = 'Asia/Ho_Chi_Minh';

// OID của các kiểu số: psql căn phải cột số, căn trái cột còn lại.
const NUMERIC_OIDS = new Set([20, 21, 23, 26, 700, 701, 790, 1700]);

// ---------------------------------------------------------------------------
// Tách file SQL thành từng câu lệnh.
//
// Hiểu: chú thích `--` và `/* */` (lồng nhau được), chuỗi '…' (kể cả E'…' với
// dấu \), định danh "…", chuỗi dollar $tag$…$tag$. Chú thích đứng trước câu
// lệnh được giữ lại cùng câu lệnh (để đọc chỉ thị expect-*).
export function splitSql(text) {
  const statements = [];
  let start = 0;
  let i = 0;
  let line = 1;
  let startLine = 1;
  const n = text.length;
  const push = (end) => {
    const chunk = text.slice(start, end);
    if (hasCode(chunk)) statements.push({ text: chunk.trim(), line: startLine + leadingNewlines(chunk) });
    else if (/--\s*expect-[a-z0-9]+\s*:/i.test(chunk)) {
      throw new Error(`Dòng ${startLine}: chỉ thị expect-* không đứng trước câu lệnh nào`);
    }
  };
  while (i < n) {
    const c = text[i];
    const next = text[i + 1];
    if (c === '\n') { line++; i++; continue; }
    if (c === '-' && next === '-') {
      while (i < n && text[i] !== '\n') i++;
      continue;
    }
    if (c === '/' && next === '*') {
      let depth = 1;
      i += 2;
      while (i < n && depth > 0) {
        if (text[i] === '\n') line++;
        if (text[i] === '/' && text[i + 1] === '*') { depth++; i += 2; continue; }
        if (text[i] === '*' && text[i + 1] === '/') { depth--; i += 2; continue; }
        i++;
      }
      continue;
    }
    if (c === "'") {
      const escapes = i > 0 && /[eE]/.test(text[i - 1]) && (i < 2 || !/[A-Za-z0-9_]/.test(text[i - 2]));
      i++;
      while (i < n) {
        if (text[i] === '\n') line++;
        if (escapes && text[i] === '\\') { i += 2; continue; }
        if (text[i] === "'") {
          if (text[i + 1] === "'") { i += 2; continue; }
          break;
        }
        i++;
      }
      i++;
      continue;
    }
    if (c === '"') {
      i++;
      while (i < n) {
        if (text[i] === '\n') line++;
        if (text[i] === '"') {
          if (text[i + 1] === '"') { i += 2; continue; }
          break;
        }
        i++;
      }
      i++;
      continue;
    }
    if (c === '$') {
      const m = /^\$([A-Za-z_][A-Za-z0-9_]*)?\$/.exec(text.slice(i));
      const prev = i > 0 ? text[i - 1] : '';
      if (m && !/[A-Za-z0-9_]/.test(prev)) {
        const tag = m[0];
        const close = text.indexOf(tag, i + tag.length);
        const end = close === -1 ? n : close + tag.length;
        for (let k = i; k < end; k++) if (text[k] === '\n') line++;
        i = end;
        continue;
      }
    }
    if (c === '\\' && atLineStart(text, i)) {
      throw new Error(`Dòng ${line}: lệnh meta của psql (\\…) không chạy được trong runner — chỉ dùng SQL thuần`);
    }
    if (c === ';') {
      push(i + 1);
      i++;
      start = i;
      startLine = line;
      continue;
    }
    i++;
  }
  push(n);
  return statements;
}

function atLineStart(text, i) {
  for (let k = i - 1; k >= 0; k--) {
    if (text[k] === '\n') return true;
    if (text[k] !== ' ' && text[k] !== '\t') return false;
  }
  return true;
}

function leadingNewlines(chunk) {
  const m = /^\s*/.exec(chunk);
  return (m[0].match(/\n/g) || []).length;
}

/** Bỏ chú thích và khoảng trắng; còn ký tự nào thì chunk là một câu lệnh thật. */
function hasCode(chunk) {
  return stripComments(chunk).replace(/;/g, '').trim().length > 0;
}

function stripComments(sql) {
  return sql.replace(/\/\*[\s\S]*?\*\//g, ' ').replace(/--[^\n]*/g, ' ');
}

/** Đọc chỉ thị expect-* trong các dòng chú thích đứng trước phần mã. */
export function parseExpectations(statementText) {
  const exp = {};
  for (const raw of statementText.split('\n')) {
    const lineText = raw.trim();
    if (lineText === '') continue;
    if (!lineText.startsWith('--')) break;
    let m = /^--\s*expect-error\s*:\s*([0-9A-Z]{5})(?:\s+([A-Za-z0-9_]+))?\s*$/.exec(lineText);
    if (m) { exp.error = { code: m[1], name: m[2] ?? null }; continue; }
    m = /^--\s*expect-rows\s*:\s*(\d+)\s*$/.exec(lineText);
    if (m) { exp.rows = Number(m[1]); continue; }
    m = /^--\s*expect-value\s*:\s?(.*)$/.exec(lineText);
    if (m) { exp.value = m[1].trim(); continue; }
    m = /^--\s*expect-line\s*:\s?(.*)$/.exec(lineText);
    if (m) { (exp.lines ??= []).push(normalizeLine(m[1])); continue; }
    m = /^--\s*expect-md5\s*:\s*([0-9a-f]{32})\s*$/.exec(lineText);
    if (m) { exp.md5 = m[1]; continue; }
    if (/^--\s*expect-/i.test(lineText)) throw new Error(`Chỉ thị không hiểu: ${lineText}`);
  }
  return exp;
}

/** Chuẩn hoá một dòng kỳ vọng `a |  b|c` → `a | b | c`. */
function normalizeLine(text) {
  return text.split('|').map((c) => c.trim()).join(' | ');
}

/** Giá trị một ô dạng chữ để so kỳ vọng: NULL → `NULL`, bỏ khoảng trắng hai đầu. */
export function cellText(v) {
  return v === null || v === undefined ? 'NULL' : String(v).trim();
}

/**
 * Dạng chuẩn của cả kết quả để tính md5: mỗi dòng là các ô (cellText) nối bằng
 * ` | `, các dòng nối bằng ký tự xuống dòng, không có tiêu đề cột, không có
 * dòng cuối trống.
 */
export function canonicalResult(rows) {
  return rows.map((r) => r.map(cellText).join(' | ')).join('\n');
}

export function resultMd5(rows) {
  return createHash('md5').update(canonicalResult(rows), 'utf8').digest('hex');
}

// ---------------------------------------------------------------------------
// Gửi một câu lệnh bằng "simple query protocol" (giống psql) để nhận đúng
// chuỗi hiển thị do PostgreSQL định dạng và đúng thẻ lệnh (command tag).
const encoder = new TextEncoder();
function simpleQuery(sql) {
  const body = encoder.encode(sql);
  const msg = new Uint8Array(1 + 4 + body.length + 1);
  msg[0] = 0x51; // 'Q'
  new DataView(msg.buffer).setInt32(1, 4 + body.length + 1);
  msg.set(body, 5);
  return msg;
}

/** Chạy một câu lệnh, trả { fields, rows, tag, error, notices }. */
export async function execStatement(db, sql) {
  const notices = [];
  const res = await db.execProtocol(simpleQuery(sql), {
    throwOnError: false,
    onNotice: (n) => notices.push(n),
  });
  const out = { fields: null, rows: [], tag: null, error: null, notices, suppressed: [] };
  // PGlite (không phải PostgreSQL) phát thêm "# 1743: aborting transaction N"
  // khi ROLLBACK hoặc khi câu lệnh lỗi. Server thật không có dòng này ⇒ ẩn đi.
  for (let k = notices.length - 1; k >= 0; k--) {
    if (/^# \d+: aborting transaction \d+$/.test(notices[k].message ?? '')) {
      out.suppressed.push(notices[k]);
      notices.splice(k, 1);
    }
  }
  for (const m of res.messages) {
    if (m.name === 'rowDescription') { out.fields = m.fields; out.rows = []; }
    else if (m.name === 'dataRow') out.rows.push(m.fields);
    else if (m.name === 'commandComplete') out.tag = m.text;
    else if (m.name === 'error') out.error = m;
  }
  return out;
}

// ---------------------------------------------------------------------------
// Định dạng giống psql (chế độ aligned).
const width = (s) => [...s].length;
const padRight = (s, w) => s + ' '.repeat(Math.max(0, w - width(s)));
const padLeft = (s, w) => ' '.repeat(Math.max(0, w - width(s))) + s;
function center(s, w) {
  const total = Math.max(0, w - width(s));
  const left = Math.floor(total / 2);
  return ' '.repeat(left) + s + ' '.repeat(total - left);
}

export function formatTable(fields, rows) {
  const headers = fields.map((f) => f.name);
  const cells = rows.map((r) => r.map((v) => (v === null ? '' : String(v))));
  const multiline = cells.some((r) => r.some((v) => v.includes('\n')));
  const widths = headers.map((h, j) =>
    Math.max(width(h), ...cells.map((r) => Math.max(...r[j].split('\n').map(width)))),
  );
  const numeric = fields.map((f) => NUMERIC_OIDS.has(f.dataTypeID));
  const lines = [];
  lines.push((' ' + headers.map((h, j) => center(h, widths[j])).join(' | ')).trimEnd());
  lines.push(widths.map((w) => '-'.repeat(w + 2)).join('+'));
  for (const r of cells) {
    const parts = r.map((v) => v.split('\n'));
    const height = multiline ? Math.max(...parts.map((p) => p.length)) : 1;
    for (let k = 0; k < height; k++) {
      const row = parts.map((p, j) => {
        const v = p[k] ?? '';
        return numeric[j] ? padLeft(v, widths[j]) : padRight(v, widths[j]);
      });
      lines.push((' ' + row.join(' | ')).trimEnd());
    }
  }
  lines.push(`(${rows.length} ${rows.length === 1 ? 'row' : 'rows'})`);
  return lines.join('\n');
}

function formatNotice(n) {
  const sev = n.severity || 'NOTICE';
  const lines = [`${sev}:  ${n.message}`];
  if (n.detail) lines.push(`DETAIL:  ${n.detail}`);
  if (n.hint) lines.push(`HINT:  ${n.hint}`);
  return lines.join('\n');
}

function formatError(e) {
  const lines = [`ERROR:  ${e.message}`];
  if (e.detail) lines.push(`DETAIL:  ${e.detail}`);
  if (e.hint) lines.push(`HINT:  ${e.hint}`);
  const meta = [`SQLSTATE ${e.code}`];
  if (e.constraint) meta.push(`constraint ${e.constraint}`);
  if (e.table) meta.push(`table ${e.table}`);
  if (e.column) meta.push(`column ${e.column}`);
  lines.push(`-- [lab] ${meta.join(' · ')}`);
  return lines.join('\n');
}

/** Số dòng mà thẻ lệnh báo (UPDATE 3 → 3, INSERT 0 2 → 2). */
function tagCount(tag) {
  const m = /(\d+)$/.exec(tag ?? '');
  return m ? Number(m[1]) : null;
}

// Thẻ lệnh psql KHÔNG in: SELECT (đã có "(n rows)"). Ngoại lệ: câu không trả
// bảng mà vẫn có thẻ SELECT n (CREATE TABLE … AS, SELECT … INTO) — psql in thẻ đó.
const SILENT_TAGS = /^(SELECT|FETCH|SHOW)\b/;
const isSilentTag = (r) => SILENT_TAGS.test(r.tag) && r.fields !== null;

// ---------------------------------------------------------------------------
/** Chuẩn bị phiên: múi giờ và mức thông báo giống server PostgreSQL thường gặp. */
export async function prepareSession(db) {
  // Server mặc định thường là client_min_messages = notice; PGlite để debug3
  // nên sẽ in cả thông báo DEBUG nội bộ. Đặt lại cho giống psql thật.
  await db.exec(`SET client_min_messages = notice; SET TIME ZONE '${LAB_TIMEZONE}';`);
}

export async function createDb({ persistDir } = {}) {
  const db = persistDir ? await PGlite.create({ dataDir: persistDir }) : await PGlite.create();
  await prepareSession(db);
  return db;
}

/**
 * Chạy danh sách file trên một db. `write` nhận từng đoạn văn bản in ra.
 * Trả về thống kê và kết quả từng câu (cho test).
 */
export async function runFiles(db, files, { write = (s) => process.stdout.write(s), echo = true, baseDir = process.cwd() } = {}) {
  const summary = {
    statements: 0,
    expectedErrors: 0,
    rowChecks: 0,
    valueChecks: 0,
    lineChecks: 0,
    md5Checks: 0,
    failures: [],
    results: [],
  };
  for (const file of files) {
    const rel = relative(baseDir, file).replace(/\\/g, '/');
    write(`\n-- [lab] ===== ${rel} =====\n`);
    let statements;
    try {
      statements = splitSql(readFileSync(file, 'utf8'));
    } catch (e) {
      summary.failures.push({ file: rel, line: 0, reason: e.message });
      write(`-- [lab] FAIL: ${e.message}\n`);
      continue;
    }
    for (const st of statements) {
      summary.statements++;
      const where = `${rel}:${st.line}`;
      let exp;
      try {
        exp = parseExpectations(st.text);
      } catch (e) {
        summary.failures.push({ file: rel, line: st.line, reason: e.message });
        write(`-- [lab] FAIL ${where}: ${e.message}\n`);
        continue;
      }
      if (echo) write(`\n${st.text}\n`);
      const r = await execStatement(db, st.text);
      const shown = [];
      for (const n of r.notices) shown.push(formatNotice(n));
      if (r.error) shown.push(formatError(r.error));
      else {
        if (r.fields) shown.push(formatTable(r.fields, r.rows) + (r.tag && !SILENT_TAGS.test(r.tag) ? '\n' : ''));
        if (r.tag && !isSilentTag(r)) shown.push(r.tag);
      }
      if (shown.length) write(shown.join('\n') + '\n');

      const record = { file: rel, line: st.line, sql: st.text, expect: exp, ...r, ok: true };
      // Đánh giá kỳ vọng.
      if (exp.error) {
        summary.expectedErrors++;
        const want = `${exp.error.code}${exp.error.name ? ' ' + exp.error.name : ''}`;
        if (!r.error) {
          record.ok = false;
          record.reason = `mong đợi lỗi ${want} nhưng câu lệnh chạy thành công`;
        } else {
          const name = r.error.constraint ?? r.error.column ?? null;
          if (r.error.code !== exp.error.code || (exp.error.name && exp.error.name !== name)) {
            record.ok = false;
            record.reason = `mong đợi lỗi ${want}, nhận ${r.error.code} ${name ?? ''}`.trim();
          } else {
            write(`-- [lab] PASS: đúng lỗi mong đợi ${want}\n`);
          }
        }
      } else if (r.error) {
        record.ok = false;
        record.reason = `lỗi ngoài kỳ vọng ${r.error.code}: ${r.error.message}`;
      }
      if (exp.rows !== undefined && !r.error) {
        summary.rowChecks++;
        const got = r.fields ? r.rows.length : tagCount(r.tag);
        if (got !== exp.rows) {
          record.ok = false;
          record.reason = `mong đợi ${exp.rows} dòng, nhận ${got}`;
        } else {
          write(`-- [lab] PASS: đúng ${exp.rows} dòng như mong đợi\n`);
        }
      }
      const fail = (reason) => {
        record.ok = false;
        record.reason = record.reason ? `${record.reason}; ${reason}` : reason;
      };
      if (exp.value !== undefined && !r.error) {
        summary.valueChecks++;
        if (!r.fields || r.rows.length !== 1 || r.fields.length !== 1) {
          fail(`expect-value cần đúng 1 dòng × 1 cột, nhận ${r.fields ? `${r.rows.length} dòng × ${r.fields.length} cột` : 'không có bảng kết quả'}`);
        } else if (cellText(r.rows[0][0]) !== exp.value) {
          fail(`mong đợi giá trị «${exp.value}», nhận «${cellText(r.rows[0][0])}»`);
        } else {
          write(`-- [lab] PASS: đúng giá trị mong đợi ${exp.value}\n`);
        }
      }
      if (exp.lines && !r.error) {
        summary.lineChecks++;
        const got = r.fields ? r.rows.map((row) => row.map(cellText).join(' | ')) : [];
        const k = exp.lines.findIndex((l, j) => got[j] !== l);
        if (got.length !== exp.lines.length || k !== -1) {
          const j = k === -1 ? Math.min(got.length, exp.lines.length) : k;
          fail(`expect-line: mong đợi ${exp.lines.length} dòng, nhận ${got.length}; khác ở dòng ${j + 1}: mong đợi «${exp.lines[j] ?? '(hết)'}», nhận «${got[j] ?? '(hết)'}»`);
        } else {
          write(`-- [lab] PASS: đúng nội dung ${exp.lines.length} dòng như mong đợi (expect-line)\n`);
        }
      }
      if (exp.md5 && !r.error) {
        summary.md5Checks++;
        const got = r.fields ? resultMd5(r.rows) : '(không có bảng kết quả)';
        if (got !== exp.md5) fail(`mong đợi md5 ${exp.md5}, nhận ${got}`);
        else write(`-- [lab] PASS: đúng md5 kết quả ${exp.md5}\n`);
      }
      if (!record.ok) {
        summary.failures.push({ file: rel, line: st.line, reason: record.reason });
        write(`-- [lab] FAIL ${where}: ${record.reason}\n`);
      }
      summary.results.push(record);
    }
  }
  return summary;
}

/** Mở rộng đối số: thư mục ⇒ các file NN-*.sql theo tên, bỏ *-rollback.sql. */
export function expandPaths(args) {
  const files = [];
  for (const a of args) {
    if (statSync(a).isDirectory()) {
      const names = readdirSync(a)
        .filter((f) => /^\d\d-.*\.sql$/.test(f) && !/-rollback\.sql$/.test(f))
        .sort();
      for (const f of names) files.push(join(a, f));
    } else files.push(a);
  }
  return files;
}

/** Phiên bản PGlite đang cài (đọc từ node_modules, không ghi cứng). */
function pgliteVersion() {
  try {
    const url = new URL('./node_modules/@electric-sql/pglite/package.json', import.meta.url);
    return JSON.parse(readFileSync(url, 'utf8')).version;
  } catch {
    return '?';
  }
}

function formatSummary(s) {
  const lines = [
    '',
    '-- [lab] ===== Tổng kết =====',
    `-- [lab] Câu lệnh đã chạy: ${s.statements}`,
    `-- [lab] Lỗi mong đợi (expect-error): ${s.expectedErrors}`,
    `-- [lab] Kiểm số dòng (expect-rows): ${s.rowChecks}`,
  ];
  // Chỉ in khi có dùng — giữ nguyên từng byte kết quả của các phần cũ (P01).
  if (s.valueChecks) lines.push(`-- [lab] Kiểm giá trị (expect-value): ${s.valueChecks}`);
  if (s.lineChecks) lines.push(`-- [lab] Kiểm nội dung (expect-line): ${s.lineChecks}`);
  if (s.md5Checks) lines.push(`-- [lab] Kiểm md5 kết quả (expect-md5): ${s.md5Checks}`);
  lines.push(`-- [lab] FAIL: ${s.failures.length}`);
  for (const f of s.failures) lines.push(`-- [lab]   ${f.file}:${f.line} — ${f.reason}`);
  lines.push(s.failures.length === 0 ? '-- [lab] KẾT QUẢ: ĐẠT' : '-- [lab] KẾT QUẢ: KHÔNG ĐẠT');
  return lines.join('\n') + '\n';
}

async function main(argv) {
  const args = [];
  let persistDir = null;
  let echo = true;
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === '--persist') persistDir = argv[++i];
    else if (argv[i] === '--no-echo') echo = false;
    else if (argv[i] === '-h' || argv[i] === '--help') {
      console.log('Cách dùng: node run.mjs <thư-mục-phần | file.sql ...> [--persist <thư-mục>] [--no-echo]');
      return 0;
    } else args.push(argv[i]);
  }
  if (args.length === 0) {
    console.error('Thiếu đối số. Ví dụ: node run.mjs p01');
    return 2;
  }
  const files = expandPaths(args);
  const db = await createDb({ persistDir });
  const ver = (await db.query("SELECT current_setting('server_version') AS v")).rows[0].v;
  process.stdout.write(`-- [lab] Engine: PostgreSQL ${ver} qua PGlite ${pgliteVersion()} (WebAssembly, một kết nối)\n`);
  process.stdout.write(`-- [lab] Phiên: TimeZone = ${LAB_TIMEZONE}, client_min_messages = notice${persistDir ? `, dữ liệu lưu ở ${persistDir}` : ', dữ liệu trong bộ nhớ'}\n`);
  process.stdout.write(`-- [lab] File: ${files.map((f) => basename(f)).join(', ')}\n`);
  const summary = await runFiles(db, files, { echo });
  process.stdout.write(formatSummary(summary));
  await db.close();
  return summary.failures.length === 0 ? 0 : 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  // `node run.mjs p01 | head` đóng ống sớm: thoát êm thay vì in stack EPIPE.
  process.stdout.on('error', (e) => { if (e.code === 'EPIPE') process.exit(0); else throw e; });
  main(process.argv.slice(2)).then(
    (code) => { process.exitCode = code; },
    (e) => { console.error(e); process.exitCode = 2; },
  );
}
