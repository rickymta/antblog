// Tách tệp SQL (kiểu psql) thành từng bước chạy + đọc chỉ thị trong chú thích.
//
// Khác splitSql của run.mjs (PGlite) ở chỗ CHO PHÉP lệnh meta của psql:
//   * dòng bắt đầu bằng `\` (vd. `\timing on`, `\d+ trips`, `\x`) là một bước riêng;
//   * `\` giữa dòng sau một câu SQL (vd. `SELECT … \gset`, `… \gx`) kết thúc câu
//     SQL đó — cả câu lẫn lệnh meta là MỘT bước;
//   * `COPY … FROM STDIN;` kèm các dòng dữ liệu tới dòng `\.` là một bước.
// Hiểu chú thích `--` và `/* */` (lồng nhau), chuỗi '…' (cả E'…'), định danh "…",
// chuỗi dollar $tag$…$tag$.

export function splitPsql(text) {
  text = text.replace(/\r\n/g, '\n');
  const steps = [];
  let start = 0;
  let i = 0;
  let line = 1;
  let startLine = 1;
  const n = text.length;
  const push = (end, kind) => {
    let chunk = text.slice(start, end);
    if (hasCode(chunk)) {
      steps.push({ text: chunk.replace(/^\s*\n/, '').replace(/\s+$/, ''), line: startLine + leadingNewlines(chunk), kind });
    } else if (/^\s*--\s*(expect-[a-z-]+|lab\s*:|@[A-Za-z][A-Za-z0-9_]*\s*$)/im.test(chunk)) {
      throw new Error(`Dòng ${startLine}: chỉ thị (expect-*/lab:/@phiên) không đứng trước bước nào`);
    }
  };
  const isCopyFromStdin = (chunk) => /^\s*COPY\b[\s\S]*\bFROM\s+STDIN\b/i.test(stripComments(chunk));
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
      const m = /^\$([A-Za-z_][A-Za-z0-9_]*)?\$/.exec(text.slice(i, i + 80));
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
    if (c === '\\') {
      // Lệnh meta tới hết dòng.
      let e = text.indexOf('\n', i);
      if (e === -1) e = n;
      push(e, hasCode(text.slice(start, i)) ? 'sql' : 'meta');
      i = e;
      start = i;
      startLine = line;
      continue;
    }
    if (c === ';') {
      const chunk = text.slice(start, i + 1);
      if (isCopyFromStdin(chunk)) {
        // Dữ liệu COPY: các dòng sau cho tới dòng `\.`.
        const endMark = text.indexOf('\n\\.', i);
        let e = endMark === -1 ? n : endMark + 3;
        for (let k = i; k < e; k++) if (text[k] === '\n') line++;
        push(e, 'copy');
        i = e;
        start = i;
        startLine = line;
        continue;
      }
      push(i + 1, 'sql');
      i++;
      start = i;
      startLine = line;
      continue;
    }
    i++;
  }
  push(n, 'sql');
  return steps;
}

function leadingNewlines(chunk) {
  const m = /^\s*/.exec(chunk);
  return (m[0].match(/\n/g) || []).length;
}

export function stripComments(sql) {
  return sql.replace(/\/\*[\s\S]*?\*\//g, ' ').replace(/--[^\n]*/g, ' ');
}

function hasCode(chunk) {
  return stripComments(chunk).replace(/;/g, '').trim().length > 0;
}

/** Phần mã của bước (bỏ khối chú thích đứng đầu). */
export function codeOf(stepText) {
  const lines = stepText.split('\n');
  let k = 0;
  while (k < lines.length && (lines[k].trim() === '' || lines[k].trim().startsWith('--'))) k++;
  return lines.slice(k).join('\n');
}

/**
 * Đọc chỉ thị trong các dòng chú thích đứng trước phần mã của một bước.
 *
 * Kỳ vọng (giống run.mjs, cộng thêm vài chỉ thị cho kế hoạch thực thi):
 *   expect-error: SQLSTATE [tên-ràng-buộc|tên-cột]
 *   expect-rows: n          — biến ROW_COUNT của psql sau câu lệnh
 *   expect-value: v         — đúng 1 dòng × 1 cột, giá trị v
 *   expect-line: a | b      — (lặp) toàn bộ kết quả, đúng thứ tự
 *   expect-md5: hex         — md5 dạng chuẩn của toàn bộ kết quả
 *   expect-match: regex     — (lặp) MỖI regex khớp ít nhất một dòng đầu ra
 *   expect-no-match: regex  — (lặp) không dòng đầu ra nào khớp
 *   expect-range: lo hi     — đúng 1 ô, là số trong [lo, hi]
 *   expect-blocked          — (nhiều phiên) bước phải bị CHẶN chờ khoá
 * Chỉ thị runner:
 *   lab: restart            — khởi động lại server trước bước này
 *   lab: bien-dong          — kết quả đổi giữa các lần chạy (vd. thống kê lấy mẫu):
 *                             test so OUTPUT.txt bỏ qua phần kết quả của bước
 *   lab: nhat-ky            — sau bước, in các dòng mới của nhật ký server
 *   lab: cho-ms n           — (nhiều phiên) chờ tối đa n ms trước khi coi bước "đang chạy"
 *   @TEN                    — (nhiều phiên) chạy bước trên phiên TEN
 */
export function parseDirectives(stepText) {
  const exp = {};
  const lab = {};
  for (const raw of stepText.split('\n')) {
    const t = raw.trim();
    if (t === '') continue;
    if (!t.startsWith('--')) break;
    let m;
    if ((m = /^--\s*expect-error\s*:\s*([0-9A-Z]{5})(?:\s+([A-Za-z0-9_]+))?\s*$/.exec(t))) { exp.error = { code: m[1], name: m[2] ?? null }; continue; }
    if ((m = /^--\s*expect-rows\s*:\s*(\d+)\s*$/.exec(t))) { exp.rows = Number(m[1]); continue; }
    if ((m = /^--\s*expect-value\s*:\s?(.*)$/.exec(t))) { exp.value = m[1].trim(); continue; }
    if ((m = /^--\s*expect-line\s*:\s?(.*)$/.exec(t))) { (exp.lines ??= []).push(normLine(m[1])); continue; }
    if ((m = /^--\s*expect-md5\s*:\s*([0-9a-f]{32})\s*$/.exec(t))) { exp.md5 = m[1]; continue; }
    if ((m = /^--\s*expect-match\s*:\s?(.*)$/.exec(t))) { (exp.match ??= []).push(compile(m[1].trim(), t)); continue; }
    if ((m = /^--\s*expect-no-match\s*:\s?(.*)$/.exec(t))) { (exp.noMatch ??= []).push(compile(m[1].trim(), t)); continue; }
    if ((m = /^--\s*expect-range\s*:\s*(-?[0-9.]+)\s+(-?[0-9.]+)\s*$/.exec(t))) { exp.range = [Number(m[1]), Number(m[2])]; continue; }
    if (/^--\s*expect-blocked\s*$/.test(t)) { exp.blocked = true; continue; }
    if (/^--\s*expect-/i.test(t)) throw new Error(`Chỉ thị không hiểu: ${t}`);
    if ((m = /^--\s*lab\s*:\s*(.+)$/.exec(t))) {
      const v = m[1].trim();
      let k;
      if (v === 'restart') lab.restart = true;
      else if (v === 'bien-dong') lab.volatile = true;
      else if (v === 'nhat-ky') lab.serverLog = true;
      else if ((k = /^cho-ms\s+(\d+)$/.exec(v))) lab.waitMs = Number(k[1]);
      else throw new Error(`Chỉ thị lab: không hiểu: ${t}`);
      continue;
    }
    if ((m = /^--\s*@([A-Za-z][A-Za-z0-9_]*)\s*$/.exec(t))) { lab.session = m[1]; continue; }
  }
  return { exp, lab };
}

function compile(src, line) {
  try {
    return new RegExp(src);
  } catch (e) {
    throw new Error(`Regex sai trong «${line}»: ${e.message}`);
  }
}

export function normLine(text) {
  return text.split('|').map((c) => c.trim()).join(' | ');
}
