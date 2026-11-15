import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import rehypeSlug from 'rehype-slug';
import rehypeHighlight from 'rehype-highlight';
import GithubSlugger from 'github-slugger';
import type { GlossaryTerm } from '@tkh/shared';
import { rehypeGlossary } from './rehypeGlossary';
import 'highlight.js/styles/github-dark.css';

/**
 * Bộ hiển thị nội dung bài viết DÙNG CHUNG cho trang đọc và bản xem trước ở trang quản trị —
 * đổi cách render ở đây là đổi cho cả hai nơi.
 *
 * Nội dung do tác giả (vai trò author/admin) viết nên cho phép HTML/SVG thô (rehype-raw) để nhúng hình.
 * KHÔNG dùng component này cho chữ người đọc gửi lên (bình luận) — bình luận hiển thị dạng văn bản thuần.
 */
export function ArticleContent({
  markdown,
  className,
  glossary,
}: {
  markdown: string;
  className?: string;
  /** Thuật ngữ của từ điển keyword (`GET /glossary`). Vắng hoặc rỗng = không đánh dấu keyword. */
  glossary?: readonly GlossaryTerm[];
}) {
  return (
    <div className={className ? `tkh-prose ${className}` : 'tkh-prose'}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        // rehypeGlossary chạy SAU rehypeRaw (để thấy `<dfn data-term>` tác giả viết tay) và trước highlight.
        rehypePlugins={[rehypeRaw, [rehypeGlossary, { terms: glossary }], rehypeSlug, [rehypeHighlight, { detect: false, ignoreMissing: true }]]}
      >
        {markdown}
      </ReactMarkdown>
    </div>
  );
}

export interface Heading {
  id: string;
  text: string;
  depth: 2 | 3;
}

/**
 * Mục lục H2/H3 lấy từ Markdown, id sinh bằng github-slugger — CÙNG thuật toán với rehype-slug,
 * nên `#id` khớp với tiêu đề đã render. Bỏ qua tiêu đề nằm trong khối mã.
 */
export function extractHeadings(markdown: string): Heading[] {
  const slugger = new GithubSlugger();
  const out: Heading[] = [];
  let inFence = false;
  for (const raw of markdown.split(/\r?\n/)) {
    const line = raw.trimEnd();
    if (/^(```|~~~)/.test(line.trimStart())) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;
    const m = /^(#{1,6})\s+(.+?)\s*#*\s*$/.exec(line);
    if (!m) continue;
    const text = m[2]!
      .replace(/`([^`]*)`/g, '$1')
      .replace(/\*\*|__|\*|_/g, '')
      .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
      .trim();
    // Gọi slug cho MỌI cấp tiêu đề để số đếm trùng tên khớp rehype-slug, nhưng chỉ giữ H2/H3.
    const id = slugger.slug(text);
    const depth = m[1]!.length;
    if (depth === 2 || depth === 3) out.push({ id, text, depth });
  }
  return out;
}
