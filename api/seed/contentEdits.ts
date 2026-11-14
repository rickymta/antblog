/**
 * Lõi của `npm run content:apply` — áp danh sách chỉnh sửa câu chữ (do người hoặc AI đề xuất) vào bài Markdown
 * với các chốt an toàn, để việc rà soát hàng loạt không thể làm hỏng code, liên kết hay frontmatter.
 * Quy trình: docs/content-dictionary/01-content-review.md.
 */
import matter from 'gray-matter';
import { proseLines, type Severity } from './contentLint.js';

export interface ContentEdit {
  /** Đoạn gốc, chép NGUYÊN VĂN từ bài, nằm trên một dòng, xuất hiện đúng một lần. */
  old: string;
  new: string;
  reason?: string;
  severity?: Severity;
}

export interface EditFile {
  file: string;
  edits: ContentEdit[];
}

export interface Rejection {
  edit: ContentEdit;
  why: string;
}

export interface EditResult {
  output: string;
  applied: ContentEdit[];
  rejected: Rejection[];
  /** Lỗi toàn tệp ⇒ không ghi gì (output = bản gốc). */
  fileError?: string;
}

/** Thuộc tính chứa chữ người đọc thấy/nghe — được sửa như văn xuôi (khớp VISIBLE_ATTRS của content:lint). */
const VISIBLE_ATTR_VALUE = /\b(aria-label|alt|title)=(["'])[^"']*\2/g;

/**
 * Các phần phải giữ nguyên tuyệt đối khi sửa câu chữ. Thẻ HTML được so sánh sau khi thay giá trị
 * aria-label/alt/title bằng chỗ trống, để sửa chữ hiển thị trong đó không bị coi là đổi thẻ.
 */
function protectedTokens(s: string): string[] {
  const out: string[] = [];
  for (const re of [/(`+)[^`]*?\1/g, /<\/?[A-Za-z][^>]*>/g, /\]\([^)]*\)/g, /https?:\/\/[^\s)>"]+/g]) {
    for (const m of s.matchAll(re)) out.push(m[0].startsWith('<') ? m[0].replace(VISIBLE_ATTR_VALUE, '$1=$2$2') : m[0]);
  }
  return out.sort();
}

function fencedBlocks(markdown: string): string[] {
  const blocks: string[] = [];
  let cur: string[] | null = null;
  let fence = '';
  for (const line of markdown.split(/\r?\n/)) {
    const t = line.trimStart();
    if (cur) {
      cur.push(line);
      if (t.startsWith(fence) && t.slice(fence.length).trim() === '') {
        blocks.push(cur.join('\n'));
        cur = null;
      }
      continue;
    }
    const f = /^(`{3,}|~{3,})/.exec(t);
    if (f) {
      fence = f[1]!;
      cur = [line];
    }
  }
  if (cur) blocks.push(cur.join('\n'));
  return blocks;
}

const same = (a: string[], b: string[]) => a.length === b.length && a.every((x, i) => x === b[i]);

function countOccurrences(hay: string, needle: string): number {
  let n = 0;
  for (let i = hay.indexOf(needle); i !== -1; i = hay.indexOf(needle, i + 1)) n++;
  return n;
}

export function applyEdits(markdown: string, edits: ContentEdit[]): EditResult {
  const eol = markdown.includes('\r\n') ? '\r\n' : '\n';
  const original = markdown.normalize('NFC').replace(/\r\n/g, '\n');
  let text = original;
  const applied: ContentEdit[] = [];
  const rejected: Rejection[] = [];

  for (const raw of edits) {
    const edit = { ...raw, old: raw.old.normalize('NFC'), new: raw.new.normalize('NFC') };
    const reject = (why: string) => rejected.push({ edit: raw, why });
    if (!edit.old.trim()) { reject('old rỗng'); continue; }
    if (edit.old === edit.new) { reject('old trùng new'); continue; }
    if (/\n/.test(edit.old) || /\n/.test(edit.new)) { reject('chỉ sửa trong một dòng'); continue; }
    const n = countOccurrences(text, edit.old);
    if (n !== 1) { reject(n === 0 ? 'không tìm thấy old (chép chưa nguyên văn?)' : `old xuất hiện ${n} lần — cần thêm ngữ cảnh`); continue; }
    if (!same(protectedTokens(edit.old), protectedTokens(edit.new))) {
      reject('đổi inline code / thẻ HTML / liên kết / URL');
      continue;
    }
    // Dòng chứa đoạn sửa phải là văn xuôi: không nằm trong khối code, <pre>, frontmatter ngoài title/summary.
    const start = text.indexOf(edit.old);
    const lineNo = text.slice(0, start).split('\n').length - 1;
    const lines = text.split('\n');
    const prose = proseLines(text)[lineNo] ?? '';
    const lineStart = start - text.slice(0, start).lastIndexOf('\n') - 1;
    const span = prose.slice(lineStart, lineStart + edit.old.length);
    const srcSpan = lines[lineNo]!.slice(lineStart, lineStart + edit.old.length);
    // Ký tự bị che (thành khoảng trắng) mà không nằm trong token được bảo vệ ⇒ đoạn sửa chạm vào vùng không phải văn xuôi.
    const maskedProtected = protectedTokens(edit.old)
      .reduce((s, tok) => s.split(tok).join(' '.repeat(tok.length)), srcSpan)
      // Cú pháp của thuộc tính hiển thị (aria-label=" … ") không phải văn xuôi nhưng được phép nằm trong đoạn sửa.
      .replace(/\b(aria-label|alt|title)=["']|["']/g, (m) => ' '.repeat(m.length));
    let touchesCode = false;
    for (let i = 0; i < srcSpan.length; i++) {
      if (span[i] === ' ' && srcSpan[i] !== ' ' && maskedProtected[i] !== ' ') { touchesCode = true; break; }
    }
    if (!prose.trim() || touchesCode) { reject('nằm trong khối code / frontmatter / vùng không phải văn xuôi'); continue; }
    text = text.slice(0, start) + edit.new + text.slice(start + edit.old.length);
    applied.push(raw);
  }

  // Chốt toàn tệp.
  const fail = (fileError: string): EditResult => ({ output: markdown, applied: [], rejected: edits.map((edit) => ({ edit, why: fileError })), fileError });
  if (!same(fencedBlocks(original), fencedBlocks(text))) return fail('khối code bị thay đổi');
  if (!same(protectedTokens(original), protectedTokens(text))) return fail('inline code / thẻ / liên kết bị thay đổi');
  if (text.split('\n').length !== original.split('\n').length) return fail('số dòng thay đổi');
  try {
    const before = matter(original).data as Record<string, unknown>;
    const after = matter(text).data as Record<string, unknown>;
    for (const k of new Set([...Object.keys(before), ...Object.keys(after)])) {
      if (k === 'title' || k === 'summary') {
        if (typeof after[k] !== 'string' || !(after[k] as string).trim()) return fail(`frontmatter ${k} không hợp lệ`);
      } else if (JSON.stringify(before[k]) !== JSON.stringify(after[k])) {
        return fail(`frontmatter ${k} bị thay đổi`);
      }
    }
  } catch (e) {
    return fail(`frontmatter không đọc được: ${(e as Error).message}`);
  }
  return { output: applied.length ? text.replace(/\n/g, eol) : markdown, applied, rejected };
}
