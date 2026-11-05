/**
 * TEST-047 · MC/DC cho quyết định «voucher được áp» (R-VOUCHER-03…07) — chạy thật trên src/app/quote.ts.
 * Chạy: npx vitest run articles/mc-dc-testing --reporter=verbose
 *
 * Phần A kiểm THIẾT KẾ bộ test (vectơ điều kiện tính từ dữ liệu, cặp độc lập, N + 1 ca).
 * Phần B CHẠY 10 ca trên code thật. Phần C kiểm MC/DC trên từng quyết định của code (assertVoucherEligible…).
 */
import { describe, expect, it } from 'vitest';
import { mcdcReport, type CondValue, type TestVector } from './mcdc';
import { conditionsOf, MCDC_CASES, runCase, specDecision } from './voucher-cases';

const NAMES = ['C1 tìm thấy', 'C2 ≥ validFrom', 'C3 ≤ validTo', 'C4 kinds rỗng', 'C5 kinds chứa', 'C6 cities rỗng', 'C7 cities chứa', 'C8 A ≥ min', 'C9 còn lượt'];

const specVectors: TestVector[] = MCDC_CASES.map((c) => {
  const conditions = conditionsOf(c);
  return { id: c.id, conditions, outcome: specDecision(conditions) };
});

describe('A · thiết kế bộ MC/DC', () => {
  it('kết quả mong đợi của từng ca khớp quyết định V tính từ vectơ điều kiện', () => {
    for (const c of MCDC_CASES) {
      expect(specDecision(conditionsOf(c)), c.id).toBe(!('error' in c.expected));
    }
  });

  it('10 ca = N + 1 và mọi điều kiện có cặp độc lập ⇒ MC/DC 100%', () => {
    const report = mcdcReport(specVectors);
    expect(MCDC_CASES).toHaveLength(NAMES.length + 1);
    const table = report.pairs.map((p, i) => `${NAMES[i]}: ${p ? p.pair.join(' ↔ ') : 'THIẾU'}`);
    expect(table).toEqual([
      'C1 tìm thấy: M1 ↔ M2',
      'C2 ≥ validFrom: M1 ↔ M3',
      'C3 ≤ validTo: M1 ↔ M4',
      'C4 kinds rỗng: M5 ↔ M6',
      'C5 kinds chứa: M1 ↔ M5',
      'C6 cities rỗng: M7 ↔ M8',
      'C7 cities chứa: M1 ↔ M7',
      'C8 A ≥ min: M1 ↔ M9',
      'C9 còn lượt: M1 ↔ M10',
    ]);
    expect(report.percent).toBe(100);
  });

  it('bỏ M6 thì mất cặp của C4 ⇒ MC/DC còn 8/9 = 88,9%', () => {
    const report = mcdcReport(specVectors.filter((v) => v.id !== 'M6'));
    expect({ shown: report.shown, percent: report.percent, missing: report.pairs.findIndex((p) => p === null) }).toEqual({
      shown: 8,
      percent: 88.9,
      missing: 3, // C4
    });
  });

  it('chỉ M1 + M2 vẫn đạt decision coverage nhưng MC/DC chỉ 1/9', () => {
    const report = mcdcReport(specVectors.filter((v) => v.id === 'M1' || v.id === 'M2'));
    expect({ bothOutcomes: report.bothOutcomes, shown: report.shown }).toEqual({ bothOutcomes: true, shown: 1 });
  });
});

describe('B · chạy 10 ca MC/DC trên src/app/quote.ts', () => {
  for (const c of MCDC_CASES) {
    it(`${c.id} · ${c.note}`, async () => {
      expect(await runCase(c)).toEqual(c.expected);
    });
  }
});

/**
 * Phần C — code KHÔNG có một biểu thức V; nó là chuỗi quyết định nhỏ (mỗi `if`/biểu thức logic).
 * Từ vectơ đặc tả suy ra vectơ của từng quyết định trong code, có tính ngắt mạch của `||`.
 */
interface CodeDecision {
  name: string;
  /** Ca có đi tới quyết định này không (các guard trước đó đều qua). */
  reached: (c: CondValue[]) => boolean;
  vector: (c: CondValue[]) => CondValue[];
}
const or2 = (a: boolean, b: boolean): CondValue[] => (a ? [true, null] : [false, b]);
const kindOk = (c: CondValue[]) => c[3] === true || c[4] === true;
const cityOk = (c: CondValue[]) => c[5] === true || c[6] === true;
const notExpired = (c: CondValue[]) => c[0] === true && c[1] === true && c[2] === true;

const CODE_DECISIONS: CodeDecision[] = [
  { name: 'voucher.ts:34 day < validFrom || day > validTo', reached: (c) => c[0] === true, vector: (c) => or2(c[1] === false, c[2] === false) },
  { name: 'voucher.ts:40 kinds.length === 0 || kinds.includes', reached: notExpired, vector: (c) => or2(c[3] === true, c[4] === true) },
  { name: 'voucher.ts:41 cityCodes.length === 0 || cityCodes.includes', reached: notExpired, vector: (c) => or2(c[5] === true, c[6] === true) },
  { name: 'voucher.ts:42 !kindOk || !cityOk', reached: notExpired, vector: (c) => or2(!kindOk(c), !cityOk(c)) },
  { name: 'voucher.ts:52 amountMilli < minAmount', reached: (c) => notExpired(c) && kindOk(c) && cityOk(c), vector: (c) => [c[7] === false] },
  { name: 'voucher.ts:63 usedToday >= perCustomerPerDay', reached: (c) => notExpired(c) && kindOk(c) && cityOk(c) && c[7] === true, vector: (c) => [c[8] === false] },
];

describe('C · cùng 10 ca đạt MC/DC trên TỪNG quyết định của code', () => {
  for (const d of CODE_DECISIONS) {
    it(d.name, () => {
      const vectors: TestVector[] = MCDC_CASES.map((c) => ({ id: c.id, cs: conditionsOf(c) }))
        .filter((x) => d.reached(x.cs))
        .map((x) => {
          const v = d.vector(x.cs);
          return { id: x.id, conditions: v, outcome: v.some((b) => b === true) }; // các quyết định trên đều là OR
        });
      const report = mcdcReport(vectors);
      expect({ percent: report.percent, bothOutcomes: report.bothOutcomes }).toEqual({ percent: 100, bothOutcomes: true });
    });
  }
});
