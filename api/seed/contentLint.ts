/**
 * Lõi của `npm run content:lint` — dò thuật ngữ/cụm từ không chuẩn trong bài Markdown theo `seed/terminology.json`.
 * Chỉ quét VĂN XUÔI: frontmatter (trừ title/summary), khối code, inline code, thẻ HTML, đích liên kết đều bị che
 * bằng khoảng trắng cùng độ dài để số dòng/cột báo ra vẫn khớp tệp gốc.
 * Quy trình rà soát: docs/content-dictionary/01-content-review.md.
 */

export type Severity = 'high' | 'medium' | 'low';
export const SEVERITIES: readonly Severity[] = ['high', 'medium', 'low'];

/** Một cách viết cần tránh của thuật ngữ. */
export interface AvoidRule {
  text: string;
  severity: Severity;
  caseSensitive?: boolean;
  /** Không khớp nếu ngay sau cụm (bỏ qua khoảng trắng) là chuỗi này — ví dụ "khử trùng" nhưng không phải "khử trùng lặp". */
  notFollowedBy?: string;
  /** true: `--fix` được phép thay bằng `use` của thuật ngữ. Chỉ bật khi phép thay đúng ở MỌI ngữ cảnh. */
  autofix?: boolean;
  note?: string;
}

/** Một mục của bảng thuật ngữ chuẩn. `use` là cách viết chuẩn trong bài tiếng Việt. */
export interface TermEntry {
  id: string;
  use: string;
  english?: string;
  /** true: giữ nguyên thuật ngữ tiếng Anh, không dịch. */
  keepEnglish?: boolean;
  /** true: đưa vào từ điển keyword (giai đoạn 1). */
  glossary?: boolean;
  definition?: string;
  note?: string;
  avoid?: AvoidRule[];
}

/** Quy tắc văn phong dạng regex (cụm từ cụt, lược danh từ…), không gắn với một thuật ngữ. */
export interface PatternRule {
  id: string;
  pattern: string;
  /** Cờ regex bổ sung ngoài `gu`, ví dụ `i`. */
  flags?: string;
  suggest: string;
  severity: Severity;
  /** Chuỗi thay thế cho `--fix` (khác `suggest`, vốn có thể liệt kê nhiều phương án). Vắng = không tự sửa. */
  replace?: string;
  note?: string;
}

export interface Terminology {
  version: number;
  terms: TermEntry[];
  patterns?: PatternRule[];
}

export interface CompiledRule {
  id: string;
  regex: RegExp;
  suggest: string;
  severity: Severity;
  fix?: string;
  note?: string;
}

export interface Finding {
  file: string;
  line: number;
  column: number;
  ruleId: string;
  match: string;
  suggest: string;
  severity: Severity;
  fix?: string;
  note?: string;
  excerpt: string;
}

const LETTER_BEFORE = '(?<![\\p{L}\\p{N}_])';
const LETTER_AFTER = '(?![\\p{L}\\p{N}_])';

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/** Ranh giới từ an toàn cho chữ có dấu (`\b` của JS không hiểu "ể", "ơ"...). */
export function phraseRegex(text: string, caseSensitive = false, notFollowedBy?: string): RegExp {
  const words = (s: string) => escapeRegex(s.normalize('NFC').trim()).replace(/\s+/g, '\\s+');
  const guard = notFollowedBy ? `(?!\\s+${words(notFollowedBy)}${LETTER_AFTER})` : '';
  return new RegExp(`${LETTER_BEFORE}${words(text)}${LETTER_AFTER}${guard}`, caseSensitive ? 'gu' : 'giu');
}

export function compileRules(t: Terminology): CompiledRule[] {
  const rules: CompiledRule[] = [];
  const ids = new Set<string>();
  for (const term of t.terms) {
    if (ids.has(term.id)) throw new Error(`terminology.json: trùng id "${term.id}".`);
    ids.add(term.id);
    for (const a of term.avoid ?? []) {
      rules.push({
        id: term.id,
        regex: phraseRegex(a.text, a.caseSensitive, a.notFollowedBy),
        suggest: term.use,
        severity: a.severity,
        fix: a.autofix ? term.use : undefined,
        note: a.note ?? term.note,
      });
    }
  }
  for (const p of t.patterns ?? []) {
    if (ids.has(p.id)) throw new Error(`terminology.json: trùng id "${p.id}".`);
    ids.add(p.id);
    const flags = [...new Set(`gu${p.flags ?? ''}`)].join('');
    rules.push({
      id: p.id,
      regex: new RegExp(p.pattern.normalize('NFC'), flags),
      suggest: p.suggest,
      severity: p.severity,
      fix: p.replace?.normalize('NFC'),
      note: p.note,
    });
  }
  return rules;
}

const blank = (s: string) => s.replace(/[^\n]/g, ' ');

/** Thuộc tính HTML/SVG mà người đọc nhìn thấy hoặc trình đọc màn hình đọc to — vẫn là văn xuôi. */
const VISIBLE_ATTRS = /\b(aria-label|alt|title)=(["'])([^"']*)\2/g;

/** Che một thẻ HTML nhưng giữ lại giá trị của các thuộc tính hiển thị. */
function maskTag(tag: string): string {
  let out = blank(tag);
  for (const m of tag.matchAll(VISIBLE_ATTRS)) {
    const start = m.index! + m[0].length - 1 - m[3]!.length;
    out = out.slice(0, start) + m[3] + out.slice(start + m[3]!.length);
  }
  return out;
}

/** Che phần không phải văn xuôi trong MỘT dòng (inline code, thẻ HTML, đích liên kết, URL trần). */
function maskInline(line: string): string {
  return line
    .replace(/(`+)[^`]*?\1/g, blank)
    .replace(/<!--.*?-->/g, blank)
    .replace(/<\/?[A-Za-z][^>]*>/g, maskTag)
    .replace(/\]\([^)]*\)/g, (m) => ']' + blank(m.slice(1)))
    .replace(/https?:\/\/\S+/g, blank);
}

/** Khối bị che toàn bộ. `<svg>` KHÔNG nằm đây: chữ trong `<text>` và aria-label hiển thị cho người đọc. */
const MASKED_BLOCK_TAGS = ['pre', 'code', 'script', 'style'];

/**
 * Trả về từng dòng của bài với phần không phải văn xuôi đã bị che. Độ dài từng dòng giữ nguyên (sau NFC).
 */
export function proseLines(markdown: string): string[] {
  const lines = markdown.normalize('NFC').split(/\r?\n/);
  const out: string[] = [];
  let i = 0;

  // Frontmatter: chỉ giữ giá trị của title/summary.
  if (lines[0]?.trim() === '---') {
    out.push(blank(lines[0]));
    i = 1;
    for (; i < lines.length; i++) {
      const l = lines[i]!;
      if (l.trim() === '---') {
        out.push(blank(l));
        i++;
        break;
      }
      const m = /^(title|summary):\s*/.exec(l);
      out.push(m ? blank(m[0]) + l.slice(m[0].length) : blank(l));
    }
  }

  let fence: string | null = null;
  let htmlBlock: string | null = null;
  for (; i < lines.length; i++) {
    const l = lines[i]!;
    const trimmed = l.trimStart();
    if (fence) {
      out.push(blank(l));
      if (trimmed.startsWith(fence) && trimmed.slice(fence.length).trim() === '') fence = null;
      continue;
    }
    const f = /^(`{3,}|~{3,})/.exec(trimmed);
    if (f) {
      fence = f[1]!;
      out.push(blank(l));
      continue;
    }
    if (htmlBlock) {
      out.push(blank(l));
      if (new RegExp(`</${htmlBlock}>`, 'i').test(l)) htmlBlock = null;
      continue;
    }
    const open = new RegExp(`<(${MASKED_BLOCK_TAGS.join('|')})(\\s|>|$)`, 'i').exec(l);
    if (open) {
      const tag = open[1]!.toLowerCase();
      const closesSameLine = new RegExp(`</${tag}>`, 'i').test(l.slice(open.index));
      if (!closesSameLine) {
        htmlBlock = tag;
        out.push(blank(l));
        continue;
      }
      // Mở và đóng cùng dòng: che riêng đoạn đó, phần còn lại vẫn là văn xuôi.
      const re = new RegExp(`<${tag}[\\s>][\\s\\S]*?</${tag}>`, 'gi');
      out.push(maskInline(l.replace(re, blank)));
      continue;
    }
    out.push(maskInline(l));
  }
  return out;
}

export function lintMarkdown(file: string, markdown: string, rules: CompiledRule[]): Finding[] {
  const prose = proseLines(markdown);
  const original = markdown.normalize('NFC').split(/\r?\n/);
  const findings: Finding[] = [];
  prose.forEach((line, idx) => {
    if (!line.trim()) return;
    for (const rule of rules) {
      rule.regex.lastIndex = 0;
      for (const m of line.matchAll(rule.regex)) {
        const col = m.index ?? 0;
        const src = original[idx] ?? '';
        const start = Math.max(0, col - 40);
        const end = Math.min(src.length, col + m[0].length + 40);
        findings.push({
          file,
          line: idx + 1,
          column: col + 1,
          ruleId: rule.id,
          match: m[0],
          suggest: rule.suggest,
          severity: rule.severity,
          fix: rule.fix,
          note: rule.note,
          excerpt: (start > 0 ? '…' : '') + src.slice(start, end).trim() + (end < src.length ? '…' : ''),
        });
      }
    }
  });
  return findings.sort((a, b) => a.line - b.line || a.column - b.column);
}

/** Giữ chữ hoa đầu cụm: "Kiểm sâu" → "Kiểm thử chuyên sâu". */
function matchCase(found: string, replacement: string): string {
  const first = found.charAt(0);
  if (first && first === first.toLocaleUpperCase('vi') && first !== first.toLocaleLowerCase('vi')) {
    return replacement.charAt(0).toLocaleUpperCase('vi') + replacement.slice(1);
  }
  return replacement;
}

/**
 * Áp các finding có `fix` vào nội dung. Bỏ qua finding chồng lấn finding khác trên cùng dòng.
 * Giữ nguyên kiểu xuống dòng của tệp; nội dung trả về ở dạng NFC.
 */
export function applyFixes(markdown: string, findings: Finding[]): { output: string; applied: Finding[] } {
  const eol = markdown.includes('\r\n') ? '\r\n' : '\n';
  const lines = markdown.normalize('NFC').split(/\r?\n/);
  const applied: Finding[] = [];
  const byLine = new Map<number, Finding[]>();
  for (const f of findings) if (f.fix !== undefined) byLine.set(f.line, [...(byLine.get(f.line) ?? []), f]);
  for (const [lineNo, list] of byLine) {
    let line = lines[lineNo - 1]!;
    let limit = Infinity;
    for (const f of [...list].sort((a, b) => b.column - a.column)) {
      const start = f.column - 1;
      const end = start + f.match.length;
      if (end > limit || line.slice(start, end) !== f.match) continue;
      line = line.slice(0, start) + matchCase(f.match, f.fix!) + line.slice(end);
      limit = start;
      applied.push(f);
    }
    lines[lineNo - 1] = line;
  }
  return { output: lines.join(eol), applied };
}
