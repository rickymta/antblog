import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { applyFixes, compileRules, lintMarkdown, phraseRegex, proseLines, type Terminology } from '../seed/contentLint.js';

const seedDir = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'seed');

const t: Terminology = {
  version: 1,
  terms: [
    { id: 'in-depth-testing', use: 'kiểm thử chuyên sâu', avoid: [{ text: 'kiểm sâu', severity: 'high' }] },
    { id: 'go-lang', use: 'Go', avoid: [{ text: 'GO', severity: 'low', caseSensitive: true }] },
  ],
  patterns: [{ id: 'phu-phi-mua', pattern: 'phụ phí mưa', flags: 'i', suggest: 'phụ phí trời mưa', severity: 'medium' }],
};
const rules = compileRules(t);

describe('content:lint', () => {
  it('khớp theo ranh giới từ có dấu, không khớp bên trong từ dài hơn', () => {
    expect('đáng kiểm sâu không'.match(phraseRegex('kiểm sâu'))).toHaveLength(1);
    expect('kiểm sâuhơn'.match(phraseRegex('kiểm sâu'))).toBeNull();
    expect('Kiểm   sâu'.match(phraseRegex('kiểm sâu'))).toHaveLength(1);
  });

  it('chuẩn hoá NFC: bản NFD trong bài vẫn bị bắt', () => {
    const md = 'Đưa vào vòng kiểm sâu.'.normalize('NFD');
    expect(lintMarkdown('a.md', md, rules).map((f) => f.ruleId)).toEqual(['in-depth-testing']);
  });

  it('quét chữ hiển thị trong SVG và thuộc tính aria-label/alt/title, nhưng không quét thuộc tính khác', () => {
    const md = [
      '<svg viewBox="0 0 10 10" role="img" aria-label="Cổng trước vòng kiểm sâu">',
      '<style>.kiểm-sâu{fill:red}</style>',
      '<text x="1" y="2" data-x="kiểm sâu">Kiểm sâu</text>',
      '</svg>',
    ].join('\n');
    expect(lintMarkdown('a.md', md, rules).map((f) => `${f.line}:${f.match}`)).toEqual(['1:kiểm sâu', '3:Kiểm sâu']);
  });

  it('bỏ qua khối code, inline code, thẻ HTML, đích liên kết và frontmatter ngoài title/summary', () => {
    const md = [
      '---',
      'slug: kiem-sau',
      'title: "Smoke test: có đáng kiểm sâu?"',
      'tags: [kiểm sâu]',
      '---',
      '```ts',
      '// kiểm sâu',
      '```',
      'Dùng `kiểm sâu` và <span data-note="kiểm sâu">x</span> và [a](/articles/kiểm sâu).',
      '<pre>',
      'kiểm sâu',
      '</pre>',
      'Phụ phí mưa bị nhân lên, rồi mới kiểm sâu.',
    ].join('\n');
    const found = lintMarkdown('a.md', md, rules).map((f) => `${f.line}:${f.ruleId}`);
    expect(found).toEqual(['3:in-depth-testing', '13:phu-phi-mua', '13:in-depth-testing']);
  });

  it('giữ nguyên số dòng và độ dài dòng sau khi che', () => {
    const md = '---\ntitle: X\n---\n```\ncode\n```\nvăn `a` xuôi';
    const lines = proseLines(md);
    expect(lines).toHaveLength(7);
    expect(lines[6]).toHaveLength('văn `a` xuôi'.length);
  });

  it('caseSensitive chỉ bắt đúng dạng chữ', () => {
    expect(lintMarkdown('a.md', 'GO và go', rules).map((f) => f.match)).toEqual(['GO']);
  });

  it('notFollowedBy: không bắt cụm đã đúng', () => {
    const r = compileRules({
      version: 1,
      terms: [{ id: 'dedup', use: 'khử trùng lặp', avoid: [{ text: 'khử trùng', severity: 'high', notFollowedBy: 'lặp', autofix: true }] }],
    });
    expect(lintMarkdown('a.md', 'khử trùng theo mã; khử trùng lặp đã đúng.', r).map((f) => f.column)).toEqual([1]);
  });

  it('applyFixes chỉ sửa văn xuôi, giữ chữ hoa đầu cụm, CRLF và khối code', () => {
    const r = compileRules({
      version: 1,
      terms: [{ id: 'deep', use: 'kiểm thử chuyên sâu', avoid: [{ text: 'kiểm sâu', severity: 'high', autofix: true }] }],
      patterns: [{ id: 'mua', pattern: 'phụ phí mưa', flags: 'i', suggest: 'phụ phí trời mưa / …', replace: 'phụ phí trời mưa', severity: 'medium' }],
    });
    const md = ['Kiểm sâu ngay, rồi kiểm sâu tiếp với phụ phí mưa.', '```', 'kiểm sâu', '```', '`phụ phí mưa`'].join('\r\n');
    const { output, applied } = applyFixes(md, lintMarkdown('a.md', md, r));
    expect(applied).toHaveLength(3);
    expect(output).toBe(
      ['Kiểm thử chuyên sâu ngay, rồi kiểm thử chuyên sâu tiếp với phụ phí trời mưa.', '```', 'kiểm sâu', '```', '`phụ phí mưa`'].join('\r\n'),
    );
  });

  it('applyFixes bỏ qua finding không có fix', () => {
    const md = 'Phụ phí mưa và kiểm sâu.';
    expect(applyFixes(md, lintMarkdown('a.md', md, rules)).applied).toHaveLength(0);
  });

  it('chính tả kiểu cũ (huỷ, khoá): chỉ sửa âm tiết mở oa/oe/uy', () => {
    const real = JSON.parse(readFileSync(path.join(seedDir, 'terminology.json'), 'utf8')) as Terminology;
    const r = compileRules({ version: 1, terms: [], patterns: real.patterns?.filter((p) => p.id.startsWith('chinh-ta-')) });
    const md = 'Khóa chính, hủy chuyến, tùy chọn, Ủy quyền, sức khỏe; giữ nguyên hoàng, quý, khoảng, thuyền, hoá.';
    expect(applyFixes(md, lintMarkdown('a.md', md, r)).output).toBe(
      'Khoá chính, huỷ chuyến, tuỳ chọn, Uỷ quyền, sức khoẻ; giữ nguyên hoàng, quý, khoảng, thuyền, hoá.',
    );
  });

  it('terminology.json thật hợp lệ và biên dịch được', () => {
    const real = JSON.parse(readFileSync(path.join(seedDir, 'terminology.json'), 'utf8')) as Terminology;
    expect(() => compileRules(real)).not.toThrow();
    for (const term of real.terms) expect(term.use.trim()).not.toBe('');
  });
});
