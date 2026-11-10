// Đọc, chuyển dạng và chuẩn hoá đầu ra của psql.
import { createHash } from 'node:crypto';

// ---------------------------------------------------------------------------
// stderr của psql (VERBOSITY=verbose) → dạng ngắn như psql mặc định + một dòng
// "-- [lab] SQLSTATE … · constraint … · table … · column …" giống run.mjs.
const SEV = /^(?:psql:.*?:\d+: )?(ERROR|FATAL|PANIC|WARNING|NOTICE|INFO|LOG|DEBUG\d?):  (?:([0-9A-Z]{5}): )?(.*)$/;
const FIELD = /^(SCHEMA NAME|TABLE NAME|COLUMN NAME|DATATYPE NAME|CONSTRAINT NAME):  (.*)$/;

export function transformStderr(stderr) {
  const lines = stderr.replace(/\n+$/, '').split('\n').filter((l, i, a) => !(l === '' && a.length === 1));
  const out = [];
  const messages = [];
  let cur = null;
  const flush = () => {
    if (cur && cur.severity === 'ERROR' || cur && cur.severity === 'FATAL') {
      const meta = [`SQLSTATE ${cur.code}`];
      if (cur.constraint) meta.push(`constraint ${cur.constraint}`);
      if (cur.table) meta.push(`table ${cur.table}`);
      if (cur.column) meta.push(`column ${cur.column}`);
      out.push(`-- [lab] ${meta.join(' · ')}`);
    }
    cur = null;
  };
  for (const line of lines) {
    if (line === '' && out.length === 0) continue;
    let m = SEV.exec(line);
    if (m) {
      flush();
      cur = { severity: m[1], code: m[2] ?? null, message: m[3] };
      messages.push(cur);
      out.push(`${m[1]}:  ${m[3]}`);
      continue;
    }
    m = FIELD.exec(line);
    if (m && cur) {
      const key = { 'SCHEMA NAME': 'schema', 'TABLE NAME': 'table', 'COLUMN NAME': 'column', 'DATATYPE NAME': 'datatype', 'CONSTRAINT NAME': 'constraint' }[m[1]];
      cur[key] = m[2];
      continue;
    }
    if (/^LOCATION:  /.test(line)) continue;
    out.push(line);
  }
  flush();
  return { text: out.join('\n'), messages, error: messages.find((x) => x.severity === 'ERROR' || x.severity === 'FATAL') ?? null };
}

// ---------------------------------------------------------------------------
// Đọc bảng kiểu "aligned" của psql thành mảng ô (để chấm expect-value/line/md5).
// Vị trí cột lấy từ dòng gạch ngang ("---+-----"); ô nhiều dòng (dấu + ở mép
// phải) được nối lại bằng ký tự xuống dòng.

/** Độ rộng hiển thị kiểu psql: dấu kết hợp = 0, ký tự Đông Á rộng = 2. */
export function displayWidth(s) {
  let w = 0;
  for (const ch of s) {
    const cp = ch.codePointAt(0);
    if (/\p{Mn}|\p{Me}/u.test(ch) || cp === 0x200b) continue;
    if ((cp >= 0x1100 && cp <= 0x115f) || (cp >= 0x2e80 && cp <= 0xa4cf) || (cp >= 0xac00 && cp <= 0xd7a3)
      || (cp >= 0xf900 && cp <= 0xfaff) || (cp >= 0xfe30 && cp <= 0xfe4f) || (cp >= 0xff00 && cp <= 0xff60)
      || (cp >= 0xffe0 && cp <= 0xffe6) || (cp >= 0x1f300 && cp <= 0x1faff) || (cp >= 0x20000 && cp <= 0x3fffd)) w += 2;
    else w += 1;
  }
  return w;
}

/** Cắt chuỗi theo độ rộng hiển thị: trả [phần đầu rộng `width`, phần còn lại]. */
function cutWidth(s, width) {
  let w = 0;
  let k = 0;
  const chars = [...s];
  while (k < chars.length && w < width) {
    w += displayWidth(chars[k]);
    k++;
  }
  // gộp các dấu kết hợp đi liền sau
  while (k < chars.length && displayWidth(chars[k]) === 0) k++;
  return [chars.slice(0, k).join(''), chars.slice(k).join('')];
}

export function parseAligned(stdout) {
  const lines = stdout.split('\n');
  const sepIdx = lines.findIndex((l) => /^-+(\+-+)*$/.test(l));
  if (sepIdx < 1) return null;
  const segs = lines[sepIdx].split('+').map((s) => s.length);
  const headers = splitRow(lines[sepIdx - 1], segs).map((c) => c.text.trim());
  const rows = [];
  let cur = null;
  for (let i = sepIdx + 1; i < lines.length; i++) {
    const l = lines[i];
    if (/^\(\d+ rows?\)$/.test(l)) break;
    if (l === '' && !cur) break;
    const cells = splitRow(l, segs);
    if (!cur) cur = cells.map((c) => c.text);
    else cells.forEach((c, j) => { cur[j] += '\n' + c.text; });
    if (!cells.some((c) => c.more)) {
      rows.push(cur.map((v) => v.replace(/\s+$/gm, '').replace(/^ /gm, '')));
      cur = null;
    }
  }
  return { headers, rows };
}

function splitRow(line, segs) {
  const cells = [];
  let rest = line;
  for (let j = 0; j < segs.length; j++) {
    const w = segs[j];
    let seg;
    [seg, rest] = cutWidth(rest, w);
    if (j < segs.length - 1) rest = rest.slice(1); // bỏ dấu | giữa các cột
    const padded = seg + ' '.repeat(Math.max(0, w - displayWidth(seg)));
    // seg = ' ' + nội dung + ký hiệu mép phải (' ' hoặc '+' nếu ô còn dòng tiếp)
    const marker = [...padded].at(-1);
    const inner = [...padded].slice(1, -1).join('');
    cells.push({ text: inner.trim() === '' ? '' : inner.trimEnd().replace(/^ ?/, (s) => s), more: marker === '+' });
  }
  return cells.map((c) => ({ ...c, text: c.text.trim() }));
}

export function cellText(v) {
  return v === null || v === undefined ? 'NULL' : String(v).trim();
}

export function canonicalResult(rows) {
  return rows.map((r) => r.map(cellText).join(' | ')).join('\n');
}

export function resultMd5(rows) {
  return createHash('md5').update(canonicalResult(rows), 'utf8').digest('hex');
}

// ---------------------------------------------------------------------------
// Chuẩn hoá phần KHÔNG tất định trước khi in ra (OUTPUT.txt): thời gian đo,
// đường dẫn/tên tệp tạm, cổng. Giữ nguyên số dòng, số buffer, kế hoạch.
export function normalizeOutput(text, { root, port } = {}) {
  let t = text.replace(/\r\n/g, '\n');
  t = t.replace(/actual time=\d+\.\d+\.\.\d+\.\d+/g, 'actual time=T..T');
  t = t.replace(/\b(Planning|Execution|Serialization) Time: \d+\.\d+ ms/g, '$1 Time: T ms');
  t = t.replace(/^Time: \d+[.,]\d+ ms(?: \([^)]*\))?$/gm, 'Time: T ms');
  t = t.replace(/\b(read|write)=\d+\.\d+/g, (m, k) => (/I\/O Timings/.test(t) ? `${k}=T` : m));
  t = t.replace(/pgsql_tmp\d+\.\d+/g, 'pgsql_tmpPID.N');
  t = t.replace(/\bTiming: Generation [^\n]*/g, 'Timing: …');
  t = t.replace(/\bQuery Identifier: -?\d+/g, 'Query Identifier: Q');
  t = t.replace(/\bduration: \d+\.\d+ ms/g, 'duration: T ms');
  if (root) t = t.split(root).join('<TMP>').split(root.replace(/\\/g, '/')).join('<TMP>');
  if (port) t = t.replace(new RegExp(`\\b${port}\\b`, 'g'), '<PORT>');
  return t.split('\n').map((l) => l.trimEnd()).join('\n');
}

export const VOLATILE_BEGIN = '-- [lab] >>> kết quả biến động giữa các lần chạy (test không so khớp phần này)';
export const VOLATILE_END = '-- [lab] <<< hết phần biến động';

/**
 * Chuẩn hoá MẠNH hơn để test so khớp OUTPUT.txt với lần chạy mới: che thêm
 * cost/ước lượng rows/width (phụ thuộc ANALYZE lấy mẫu ngẫu nhiên trên bảng lớn)
 * và bỏ các khối đánh dấu biến động.
 */
export function normalizeForCompare(text) {
  const out = [];
  let skip = false;
  for (const raw of text.replace(/\r\n/g, '\n').split('\n')) {
    const l = raw.trimEnd();
    if (l === VOLATILE_BEGIN) { skip = true; out.push(l); continue; }
    if (l === VOLATILE_END) { skip = false; out.push(l); continue; }
    if (skip) continue;
    let x = l
      .replace(/\(cost=[\d.]+\.\.[\d.]+ rows=\d+ width=\d+\)/g, '(cost=… rows=… width=…)');
    // Số buffer hit/read/written chia khác nhau chút ít giữa các lần chạy (vòng đệm của
    // Seq Scan, bgwriter, file tạm); "buffers full" của WAL phụ thuộc thời điểm. Các con
    // số quan trọng đã được chỉ thị expect-match trong tệp lab kiểm riêng.
    if (/^\s*Buffers: /.test(x)) x = x.replace(/Buffers: .*$/, 'Buffers: …');
    x = x.replace(/ buffers full=\d+/g, ' buffers full=N');
    // Bộ nhớ/đĩa của nút Hash/Sort/HashAggregate có thể lệch vài kB giữa các lần (cấp phát).
    x = x.replace(/\b(Memory Usage|Disk Usage|Memory|Disk): \d+kB/g, '$1: NkB');
    out.push(x);
  }
  return out.join('\n').replace(/\n+$/, '\n');
}
