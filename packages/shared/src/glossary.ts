/**
 * Từ điển keyword — kiểu dữ liệu và bộ dò keyword dùng chung (trang đọc, bản xem trước CMS, API sau này).
 * Đặc tả: docs/content-dictionary/02-glossary-feature.md.
 */

/** Thuật ngữ công khai (`GET /glossary`): đủ để dò trong bài và hiển thị hộp thoại. */
export interface GlossaryTerm {
  slug: string;
  term: string;
  english?: string;
  aliases: string[];
  shortDefinition: string;
  caseSensitive: boolean;
  autoLink: boolean;
}

export type TermStatus = 'draft' | 'published';

export interface AdminTerm extends GlossaryTerm {
  id: string;
  language: 'vi' | 'en';
  body?: string;
  status: TermStatus;
  editedInCms: boolean;
  updatedAt?: string;
}

export interface TermInput {
  slug: string;
  language?: 'vi' | 'en';
  term: string;
  english?: string;
  aliases?: string[];
  shortDefinition: string;
  body?: string;
  caseSensitive?: boolean;
  autoLink?: boolean;
  status?: TermStatus;
}

export interface GlossaryMatch {
  start: number;
  end: number;
  slug: string;
}

export interface GlossaryMatcher {
  /** Mọi chỗ khớp không chồng lấn trong `text`, theo thứ tự xuất hiện; cụm dài thắng cụm ngắn. */
  find(text: string): GlossaryMatch[];
  /** Tra thuật ngữ theo slug. */
  get(slug: string): GlossaryTerm | undefined;
}

// Ranh giới từ an toàn cho chữ có dấu (`\b` của JS không hiểu "ể", "ơ"...).
const BEFORE = '(?<![\\p{L}\\p{N}_])';
const AFTER = '(?![\\p{L}\\p{N}_])';

const escapeRegex = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const collapse = (s: string) => s.normalize('NFC').trim().replace(/\s+/g, ' ');

interface Bucket {
  regex: RegExp;
  /** Khoá tra cứu (đã thu gọn khoảng trắng, chữ thường nếu không phân biệt hoa thường) → slug. */
  bySpelling: Map<string, string>;
  key: (matched: string) => string;
}

function bucket(phrases: Array<{ text: string; slug: string }>, caseSensitive: boolean): Bucket | undefined {
  if (!phrases.length) return undefined;
  const key = (s: string) => (caseSensitive ? collapse(s) : collapse(s).toLocaleLowerCase('vi'));
  const bySpelling = new Map<string, string>();
  for (const p of phrases) if (!bySpelling.has(key(p.text))) bySpelling.set(key(p.text), p.slug);
  // Cụm dài đứng trước trong phép "hoặc" để "CAP theorem" thắng "CAP" ở cùng vị trí.
  const alternation = [...bySpelling.keys()]
    .sort((a, b) => b.length - a.length)
    .map((s) => escapeRegex(s).replace(/ /g, '\\s+'))
    .join('|');
  return { regex: new RegExp(`${BEFORE}(?:${alternation})${AFTER}`, caseSensitive ? 'gu' : 'giu'), bySpelling, key };
}

const cache = new WeakMap<readonly GlossaryTerm[], GlossaryMatcher>();

/**
 * Dựng bộ dò từ danh sách thuật ngữ (chỉ mục `autoLink`). Kết quả được cache theo chính mảng truyền vào,
 * nên gọi lại với cùng mảng (cùng một lần render) không dựng lại regex.
 */
export function buildGlossaryMatcher(terms: readonly GlossaryTerm[]): GlossaryMatcher {
  const cached = cache.get(terms);
  if (cached) return cached;
  const bySlug = new Map(terms.map((t) => [t.slug, t]));
  const sensitive: Array<{ text: string; slug: string }> = [];
  const insensitive: Array<{ text: string; slug: string }> = [];
  for (const t of terms) {
    if (!t.autoLink) continue;
    for (const text of [t.term, ...(t.aliases ?? [])]) {
      if (!collapse(text)) continue;
      (t.caseSensitive ? sensitive : insensitive).push({ text, slug: t.slug });
    }
  }
  const buckets = [bucket(sensitive, true), bucket(insensitive, false)].filter((b): b is Bucket => !!b);

  const matcher: GlossaryMatcher = {
    get: (slug) => bySlug.get(slug),
    find(text) {
      if (!buckets.length || !text.trim()) return [];
      const all: GlossaryMatch[] = [];
      for (const b of buckets) {
        b.regex.lastIndex = 0;
        for (const m of text.matchAll(b.regex)) {
          const slug = b.bySpelling.get(b.key(m[0]));
          if (slug) all.push({ start: m.index!, end: m.index! + m[0].length, slug });
        }
      }
      // Chọn tham lam: vị trí sớm trước, cùng vị trí thì cụm dài trước; bỏ chỗ chồng lấn chỗ đã chọn.
      all.sort((a, b) => a.start - b.start || b.end - a.end);
      const out: GlossaryMatch[] = [];
      let cursor = 0;
      for (const m of all) {
        if (m.start < cursor) continue;
        out.push(m);
        cursor = m.end;
      }
      return out;
    },
  };
  cache.set(terms, matcher);
  return matcher;
}
