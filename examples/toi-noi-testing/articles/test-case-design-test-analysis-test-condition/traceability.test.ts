/**
 * TCD-01 — kiểm chính kết quả test analysis (trước khi có test case nào):
 * truy vết xuôi R-VOUCHER → TCOND, truy vết ngược TCOND → SPEC, quy ước mã, ưu tiên, câu hỏi.
 * Chạy: npx vitest run articles/test-case-design-test-analysis-test-condition
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { SPEC_QUESTIONS, TEST_CONDITIONS } from './test-conditions';

const spec = readFileSync(new URL('../../SPEC.md', import.meta.url), 'utf8');
const specCodes = new Set(spec.match(/R-[A-Z]+-\d{2}/g) ?? []);
const voucherCodes = [...specCodes].filter((c) => c.startsWith('R-VOUCHER-')).sort();
const checks = readFileSync(new URL('./voucher-conditions.test.ts', import.meta.url), 'utf8');

describe('TCD-01 · truy vết test basis ↔ test condition', () => {
  it('SPEC có đúng 8 yêu cầu R-VOUCHER và mỗi yêu cầu có ≥ 1 test condition (xuôi)', () => {
    expect(voucherCodes).toEqual(['R-VOUCHER-01', 'R-VOUCHER-02', 'R-VOUCHER-03', 'R-VOUCHER-04', 'R-VOUCHER-05', 'R-VOUCHER-06', 'R-VOUCHER-07', 'R-VOUCHER-08']);
    const uncovered = voucherCodes.filter((code) => !TEST_CONDITIONS.some((c) => c.basis.includes(code as never)));
    expect(uncovered).toEqual([]);
  });

  it('ma trận xuôi: số test condition của từng yêu cầu', () => {
    const count = Object.fromEntries(voucherCodes.map((code) => [code, TEST_CONDITIONS.filter((c) => c.basis.includes(code as never)).length]));
    expect(count).toEqual({
      'R-VOUCHER-01': 4, 'R-VOUCHER-02': 2, 'R-VOUCHER-03': 1, 'R-VOUCHER-04': 3,
      'R-VOUCHER-05': 3, 'R-VOUCHER-06': 5, 'R-VOUCHER-07': 6, 'R-VOUCHER-08': 1,
    });
  });

  it('ngược: mọi mã mà test condition và câu hỏi trích dẫn đều có trong SPEC', () => {
    const cited = new Set([...TEST_CONDITIONS.flatMap((c) => c.basis), ...SPEC_QUESTIONS.flatMap((q) => q.about)]);
    expect([...cited].filter((code) => !specCodes.has(code))).toEqual([]);
  });

  it('quy ước mã: TCOND-VOUCHER-nn, duy nhất, liên tục 01…23', () => {
    const ids = TEST_CONDITIONS.map((c) => c.id);
    expect(ids.every((id) => /^TCOND-VOUCHER-\d{2}$/.test(id))).toBe(true);
    expect(new Set(ids).size).toBe(ids.length);
    expect(ids).toEqual(Array.from({ length: 23 }, (_, i) => `TCOND-VOUCHER-${String(i + 1).padStart(2, '0')}`));
  });

  it('ưu tiên sơ bộ: 9 cao · 11 trung bình · 3 thấp', () => {
    const by = (p: string) => TEST_CONDITIONS.filter((c) => c.priority === p).length;
    expect([by('cao'), by('trung-binh'), by('thap')]).toEqual([9, 11, 3]);
  });

  it('mọi condition ưu tiên cao đã có bằng chứng chạy được trong voucher-conditions.test.ts', () => {
    const missing = TEST_CONDITIONS.filter((c) => c.priority === 'cao' && !checks.includes(c.id)).map((c) => c.id);
    expect(missing).toEqual([]);
  });

  it('condition bị chặn trỏ tới một câu hỏi có thật; mọi câu hỏi có mã Q-xx duy nhất', () => {
    const qIds = SPEC_QUESTIONS.map((q) => q.id);
    expect(new Set(qIds).size).toBe(qIds.length);
    for (const c of TEST_CONDITIONS.filter((x) => x.blockedBy)) expect(qIds, c.id).toContain(c.blockedBy);
  });

  it('mức chi tiết: statement không chứa giá trị cụ thể (số tiền, mã voucher mẫu) — đó là việc của test design', () => {
    const tooFine = TEST_CONDITIONS.filter((c) => /\d{1,3}\.\d{3}|GIAM20|FOOD15|HN10|XMAS50/.test(c.statement)).map((c) => c.id);
    expect(tooFine).toEqual([]);
  });
});
