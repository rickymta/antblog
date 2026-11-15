import { buildGlossaryMatcher, type GlossaryMatcher, type GlossaryTerm } from '@tkh/shared';
import type { Element, ElementContent, Properties, Root, RootContent, Text } from 'hast';

/**
 * Plugin rehype đánh dấu keyword của từ điển trong bài: `<dfn class="tkh-term" data-term="…">chữ gốc</dfn>`.
 * Dữ liệu hộp thoại (tên, tên tiếng Anh, định nghĩa ngắn) nằm luôn trên thẻ `dfn` qua thuộc tính `data-term-*`,
 * nên trang đọc (render ở máy chủ) và bản xem trước CMS không phải truyền thêm dữ liệu nào xuống trình duyệt.
 *
 * - Chỉ đánh dấu LẦN XUẤT HIỆN ĐẦU TIÊN của mỗi thuật ngữ trong bài.
 * - Bỏ qua chữ trong code, heading, liên kết, SVG… và mọi phần tử có `data-no-glossary`.
 * - Tác giả gắn tay `<dfn data-term="slug">…</dfn>` (kể cả thuật ngữ `autoLink: false`): plugin bổ sung dữ liệu và
 *   tính là đã đánh dấu.
 * Đặc tả: docs/content-dictionary/02-glossary-feature.md.
 */
export interface RehypeGlossaryOptions {
  terms?: readonly GlossaryTerm[];
}

const SKIP_TAGS = new Set(['pre', 'code', 'kbd', 'samp', 'a', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'abbr', 'svg', 'script', 'style', 'button', 'textarea', 'select', 'summary']);

export const TERM_CLASS = 'tkh-term';

function termProperties(t: GlossaryTerm): Properties {
  return {
    className: [TERM_CLASS],
    tabIndex: 0,
    dataTerm: t.slug,
    dataTermTitle: t.term,
    ...(t.english ? { dataTermEnglish: t.english } : {}),
    dataTermDefinition: t.shortDefinition,
  };
}

function isSkipped(el: Element): boolean {
  return SKIP_TAGS.has(el.tagName) || el.properties?.dataNoGlossary !== undefined;
}

/** Tách một text node thành các mảnh văn bản và thẻ `dfn` cho những thuật ngữ chưa được đánh dấu. */
function splitText(node: Text, matcher: GlossaryMatcher, used: Set<string>): ElementContent[] | undefined {
  const matches = matcher.find(node.value).filter((m) => {
    if (used.has(m.slug)) return false;
    used.add(m.slug);
    return true;
  });
  if (!matches.length) return undefined;
  const out: ElementContent[] = [];
  let cursor = 0;
  for (const m of matches) {
    if (m.start > cursor) out.push({ type: 'text', value: node.value.slice(cursor, m.start) });
    out.push({
      type: 'element',
      tagName: 'dfn',
      properties: termProperties(matcher.get(m.slug)!),
      children: [{ type: 'text', value: node.value.slice(m.start, m.end) }],
    });
    cursor = m.end;
  }
  if (cursor < node.value.length) out.push({ type: 'text', value: node.value.slice(cursor) });
  return out;
}

function walk(parent: Root | Element, matcher: GlossaryMatcher, used: Set<string>): void {
  const next: Array<RootContent | ElementContent> = [];
  let changed = false;
  for (const child of parent.children) {
    if (child.type === 'text') {
      const parts = splitText(child, matcher, used);
      if (parts) {
        next.push(...parts);
        changed = true;
        continue;
      }
    } else if (child.type === 'element') {
      const manual = child.tagName === 'dfn' ? child.properties?.dataTerm : undefined;
      if (typeof manual === 'string') {
        const t = matcher.get(manual);
        if (t) {
          child.properties = { ...child.properties, ...termProperties(t) };
          used.add(t.slug);
        }
      } else if (!isSkipped(child)) {
        walk(child, matcher, used);
      }
    }
    next.push(child);
  }
  if (changed) parent.children = next as typeof parent.children;
}

/** Lượt quét trước: thuật ngữ tác giả gắn tay ở bất kỳ đâu cũng chặn đánh dấu tự động trước đó trong bài. */
function collectManual(node: Root | Element, matcher: GlossaryMatcher, used: Set<string>): void {
  for (const child of node.children) {
    if (child.type !== 'element') continue;
    const manual = child.tagName === 'dfn' ? child.properties?.dataTerm : undefined;
    if (typeof manual === 'string' && matcher.get(manual)) used.add(manual);
    else collectManual(child, matcher, used);
  }
}

export function rehypeGlossary(options: RehypeGlossaryOptions = {}) {
  const terms = options.terms ?? [];
  return (tree: Root): void => {
    if (!terms.length) return;
    const matcher = buildGlossaryMatcher(terms);
    const manual = new Set<string>();
    collectManual(tree, matcher, manual);
    // Thuật ngữ đã gắn tay không được đánh dấu tự động thêm lần nữa ở chỗ khác.
    const used = new Set(manual);
    walk(tree, matcher, used);
  };
}
