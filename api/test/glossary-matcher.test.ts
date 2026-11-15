// Bộ dò keyword nằm ở packages/shared (dùng chung cho web và admin); test đặt ở đây vì api là gói có sẵn vitest.
import { describe, expect, it } from 'vitest';
import { buildGlossaryMatcher, type GlossaryTerm } from '../../packages/shared/src/glossary.js';

const term = (slug: string, text: string, extra: Partial<GlossaryTerm> = {}): GlossaryTerm => ({
  slug,
  term: text,
  aliases: [],
  shortDefinition: `Định nghĩa ${slug}`,
  caseSensitive: false,
  autoLink: true,
  ...extra,
});

const found = (terms: GlossaryTerm[], text: string) =>
  buildGlossaryMatcher(terms).find(text).map((m) => `${m.slug}:${text.slice(m.start, m.end)}`);

describe('bộ dò keyword', () => {
  it('khớp theo ranh giới từ hiểu chữ có dấu, không khớp bên trong từ dài hơn', () => {
    const terms = [term('dedup', 'khử trùng lặp')];
    expect(found(terms, 'Cần khử trùng lặp theo mã.')).toEqual(['dedup:khử trùng lặp']);
    expect(found(terms, 'khử trùng lặpx')).toEqual([]);
    expect(found(terms, 'akhử trùng lặp')).toEqual([]);
  });

  it('không phân biệt hoa thường mặc định, khoảng trắng bất kỳ giữa các từ', () => {
    expect(found([term('smoke', 'smoke test')], 'Chạy Smoke   Test trước')).toEqual(['smoke:Smoke   Test']);
  });

  it('cụm dài thắng cụm ngắn ở cùng vị trí, các chỗ khớp không chồng lấn', () => {
    const terms = [term('cap', 'CAP', { caseSensitive: true }), term('cap-theorem', 'CAP theorem')];
    expect(found(terms, 'Định lý CAP theorem và CAP.')).toEqual(['cap-theorem:CAP theorem', 'cap:CAP']);
  });

  it('caseSensitive chỉ khớp đúng dạng chữ', () => {
    const terms = [term('go', 'Go', { caseSensitive: true })];
    expect(found(terms, 'Go là ngôn ngữ; go home.')).toEqual(['go:Go']);
  });

  it('alias được dò và trả về slug của thuật ngữ gốc; bỏ qua autoLink false và chuỗi rỗng', () => {
    const terms = [
      term('dedup', 'khử trùng lặp', { aliases: ['deduplication', '  '] }),
      term('manual', 'thủ công', { autoLink: false }),
    ];
    expect(found(terms, 'Deduplication, tức khử trùng lặp, làm thủ công.')).toEqual(['dedup:Deduplication', 'dedup:khử trùng lặp']);
    expect(buildGlossaryMatcher(terms).get('manual')?.term).toBe('thủ công');
  });

  it('danh sách rỗng hoặc chuỗi rỗng không khớp gì; cùng mảng thì dùng lại bộ dò đã dựng', () => {
    expect(found([], 'bất kỳ')).toEqual([]);
    const terms = [term('smoke', 'smoke test')];
    expect(found(terms, '   ')).toEqual([]);
    expect(buildGlossaryMatcher(terms)).toBe(buildGlossaryMatcher(terms));
  });
});
