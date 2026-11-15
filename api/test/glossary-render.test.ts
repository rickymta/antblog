// Kiểm tra rehypeGlossary qua đúng đường render thật (ArticleContent của packages/ui → HTML tĩnh).
// Đặt ở đây vì api là gói có sẵn vitest; React và các plugin Markdown được phân giải từ node_modules gốc của repo.
import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';
import type { GlossaryTerm } from '../../packages/shared/src/glossary.js';
import { ArticleContent } from '../../packages/ui/src/ArticleContent.js';

const terms: GlossaryTerm[] = [
  { slug: 'smoke-test', term: 'smoke test', aliases: [], shortDefinition: 'Bộ kiểm thử nhỏ, chạy nhanh.', caseSensitive: false, autoLink: true },
  {
    slug: 'deduplication',
    term: 'khử trùng lặp',
    english: 'deduplication',
    aliases: ['deduplication'],
    shortDefinition: 'Loại bỏ bản ghi lặp theo "mã" <định danh>.',
    caseSensitive: false,
    autoLink: true,
  },
  { slug: 'manual-only', term: 'thủ công', aliases: [], shortDefinition: 'Chỉ khi gắn tay.', caseSensitive: false, autoLink: false },
];

const render = (markdown: string, glossary: GlossaryTerm[] | undefined = terms) =>
  renderToStaticMarkup(createElement(ArticleContent, { markdown, glossary }));

const dfns = (html: string) => [...html.matchAll(/<dfn[^>]*data-term="([^"]+)"[^>]*>([^<]*)<\/dfn>/g)].map((m) => `${m[1]}:${m[2]}`);

describe('rehypeGlossary qua ArticleContent', () => {
  it('đánh dấu lần xuất hiện đầu tiên, nhúng dữ liệu hộp thoại đã escape, giữ chữ gốc', () => {
    const html = render('Chạy **Smoke Test** trước. Sau smoke test là khử trùng lặp.\n\nLại smoke test.');
    expect(dfns(html)).toEqual(['smoke-test:Smoke Test', 'deduplication:khử trùng lặp']);
    expect(html).toContain('class="tkh-term"');
    expect(html).toContain('tabindex="0"');
    expect(html).toContain('data-term-title="khử trùng lặp"');
    expect(html).toContain('data-term-english="deduplication"');
    expect(html).toContain('data-term-definition="Loại bỏ bản ghi lặp theo &quot;mã&quot; &lt;định danh&gt;."');
  });

  it('bỏ qua code, heading, liên kết, SVG và vùng data-no-glossary', () => {
    const md = [
      '## smoke test trong tiêu đề',
      '',
      'Dùng `smoke test` và [smoke test](/articles/x).',
      '',
      '```',
      'smoke test',
      '```',
      '',
      '<svg><text>smoke test</text></svg>',
      '',
      '<div data-no-glossary><p>smoke test</p></div>',
      '',
      'Cuối cùng mới là smoke test.',
    ].join('\n');
    expect(dfns(render(md))).toEqual(['smoke-test:smoke test']);
    expect(render(md)).toMatch(/Cuối cùng mới là <dfn[^>]*>smoke test<\/dfn>/);
  });

  it('dfn gắn tay được bổ sung dữ liệu (kể cả autoLink false) và chặn đánh dấu tự động thuật ngữ đó', () => {
    const html = render('Trước: smoke test. Rồi <dfn data-term="smoke-test">kiểm thử khói</dfn>, làm <dfn data-term="manual-only">thủ công</dfn>.');
    expect(dfns(html)).toEqual(['smoke-test:kiểm thử khói', 'manual-only:thủ công']);
    expect(html).toContain('data-term-definition="Chỉ khi gắn tay."');
  });

  it('không có từ điển thì không đổi gì', () => {
    const plain = (glossary?: GlossaryTerm[]) => renderToStaticMarkup(createElement(ArticleContent, { markdown: 'smoke test', glossary }));
    expect(plain()).not.toContain('<dfn');
    expect(plain([])).not.toContain('<dfn');
  });
});
