import { describe, expect, it } from 'vitest';
import { applyEdits } from '../seed/contentEdits.js';

const md = [
  '---',
  'slug: bai-mau',
  'title: "Năm test đầu tiên cho cước"',
  'category: testing',
  '---',
  'Một lần sửa có thể khiến phụ phí mưa bị nhân, xem `quote()` và [bài](/articles/x).',
  '```ts',
  'const phụPhíMưa = 1; // phụ phí mưa',
  '```',
  'Câu lặp. Câu lặp.',
].join('\n');

describe('content:apply', () => {
  it('áp sửa văn xuôi và title, giữ inline code/liên kết', () => {
    const r = applyEdits(md, [
      { old: 'phụ phí mưa bị nhân, xem `quote()`', new: 'phụ phí trời mưa cũng bị nhân, xem `quote()`' },
      { old: 'Năm test đầu tiên cho cước', new: 'Năm test case đầu tiên cho chức năng tính cước' },
    ]);
    expect(r.rejected).toEqual([]);
    expect(r.output).toContain('phụ phí trời mưa cũng bị nhân, xem `quote()` và [bài](/articles/x).');
    expect(r.output).toContain('title: "Năm test case đầu tiên cho chức năng tính cước"');
    expect(r.output).toContain('const phụPhíMưa = 1; // phụ phí mưa');
  });

  it('từ chối: không tìm thấy, lặp nhiều lần, nhiều dòng, đổi inline code hay liên kết', () => {
    const r = applyEdits(md, [
      { old: 'không có trong bài', new: 'x' },
      { old: 'Câu lặp.', new: 'Câu khác.' },
      { old: 'nhân,\nxem', new: 'x' },
      { old: 'xem `quote()`', new: 'xem `quoteFare()`' },
      { old: '[bài](/articles/x)', new: '[bài](/articles/y)' },
    ]);
    expect(r.applied).toEqual([]);
    expect(r.rejected.map((x) => x.why)).toEqual([
      'không tìm thấy old (chép chưa nguyên văn?)',
      'old xuất hiện 2 lần — cần thêm ngữ cảnh',
      'chỉ sửa trong một dòng',
      'đổi inline code / thẻ HTML / liên kết / URL',
      'đổi inline code / thẻ HTML / liên kết / URL',
    ]);
    expect(r.output).toBe(md);
  });

  it('từ chối sửa trong khối code và frontmatter ngoài title/summary', () => {
    const r = applyEdits(md, [
      { old: '// phụ phí mưa', new: '// phụ phí trời mưa' },
      { old: 'category: testing', new: 'category: khac' },
    ]);
    expect(r.applied).toEqual([]);
    expect(r.rejected.every((x) => x.why.startsWith('nằm trong'))).toBe(true);
  });

  it('cho sửa chữ trong aria-label/alt/title nhưng không cho đổi thuộc tính khác của thẻ', () => {
    const svg = '<svg viewBox="0 0 10 10" role="img" aria-label="Rẻ, bán kính lỗi lớn">\n<text x="1">x</text>\n</svg>';
    const ok = applyEdits(svg, [{ old: 'aria-label="Rẻ, bán kính lỗi lớn"', new: 'aria-label="Rẻ, phạm vi ảnh hưởng lớn"' }]);
    expect(ok.rejected).toEqual([]);
    expect(ok.output).toContain('aria-label="Rẻ, phạm vi ảnh hưởng lớn"');
    const bad = applyEdits(svg, [{ old: 'viewBox="0 0 10 10" role="img"', new: 'viewBox="0 0 20 20" role="img"' }]);
    expect(bad.applied).toEqual([]);
  });

  it('huỷ cả tệp nếu title bị làm rỗng; giữ CRLF khi áp', () => {
    expect(applyEdits(md, [{ old: '"Năm test đầu tiên cho cước"', new: '""' }]).fileError).toBe('frontmatter title không hợp lệ');
    const crlf = md.replace(/\n/g, '\r\n');
    const r = applyEdits(crlf, [{ old: 'Câu lặp. Câu lặp.', new: 'Câu này lặp hai lần.' }]);
    expect(r.output).toBe(crlf.replace('Câu lặp. Câu lặp.', 'Câu này lặp hai lần.'));
  });
});
