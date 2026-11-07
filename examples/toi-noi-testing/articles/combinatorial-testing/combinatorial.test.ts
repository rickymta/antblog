/**
 * TEST-038 — Combinatorial Testing trên điều kiện voucher (SPEC R-VOUCHER-01…08).
 * 1) Bộ test chạy trên mã THẬT (priceQuote của src/app/quote.ts) và phải xanh.
 * 2) Bản sao luồng (voucher-flow.ts, biến thể "dung") trùng kết quả với priceQuote trên mọi tổ hợp.
 * 3) Đo bộ t = 1, 2, 3 bắt được bản lỗi nào.
 */
import { describe, expect, it } from 'vitest';
import type { AppDeps } from '../../src/app/deps';
import { priceQuote } from '../../src/app/quote';
import { DomainError } from '../../src/domain/errors';
import { allRows, coverage, generate, type Row } from './t-wise';
import { evaluate, fails, VARIANTS, type Variant } from './voucher-flow';
import {
  MIXED_MODEL,
  negativeCases,
  toScenario,
  VOUCHER_CONSTRAINTS,
  VOUCHER_MODEL,
  type Outcome,
  type Scenario,
} from './voucher-model';

/** Gọi priceQuote thật với kho voucher/lượt dùng/thời tiết dựng từ kịch bản. */
async function runReal(s: Scenario): Promise<Outcome> {
  const deps = {
    vouchers: { findByCode: async (code: string) => (code === s.voucher.code ? structuredClone(s.voucher) : null) },
    weather: { isRaining: async () => false },
    trips: {
      countVoucherUses: async ({ from, to }: { from: Date; to: Date }) => s.uses.filter((u) => u >= from && u < to).length,
    },
  } as unknown as AppDeps;
  try {
    const q = await priceQuote(deps, 'C001', s.input, s.at);
    return { ok: true, discount: q.discount };
  } catch (error) {
    if (error instanceof DomainError) return { ok: false, code: error.code };
    throw error;
  }
}

const full = allRows(VOUCHER_MODEL, VOUCHER_CONSTRAINTS);
const scenarios = (rows: readonly Row[]) => rows.map((r, i) => toScenario(VOUCHER_MODEL, r, `#${i + 1}`));
const redCount = (v: Variant, rows: readonly Row[]) => scenarios(rows).filter((s) => fails(v, s)).length;

describe('TEST-038 · mô hình và ràng buộc', () => {
  it('tích Descartes 10.368 tổ hợp; còn 5.616 sau ba ràng buộc C1–C3', () => {
    expect(allRows(VOUCHER_MODEL)).toHaveLength(10_368);
    expect(full).toHaveLength(5_616);
  });

  it('mã thật (priceQuote) cho đúng kết quả mong đợi trên cả 5.616 tổ hợp hợp lệ và 7 test âm', async () => {
    for (const s of [...scenarios(full), ...negativeCases()]) {
      expect(await runReal(s), s.id).toEqual(s.expected);
    }
  });

  it('bản sao luồng "dung" trùng kết quả với priceQuote — kể cả trên mô hình trộn giá trị lỗi', async () => {
    const mixed = allRows(MIXED_MODEL, VOUCHER_CONSTRAINTS).map((r, i) => toScenario(MIXED_MODEL, r, `m${i}`));
    for (const s of [...scenarios(full), ...negativeCases(), ...mixed]) {
      expect(evaluate('dung', s), s.id).toEqual(await runReal(s));
    }
  });
});

describe('TEST-038 · each-choice, pairwise, 3-wise có ràng buộc', () => {
  const sizes: Record<number, number> = { 1: 3, 2: 14, 3: 40 };
  for (const t of [1, 2, 3]) {
    it(`t = ${t}: ${sizes[t]} test, phủ 100% bộ-${t} khả thi, không hàng nào vi phạm ràng buộc`, () => {
      const suite = generate(VOUCHER_MODEL, t, VOUCHER_CONSTRAINTS);
      expect(suite).toHaveLength(sizes[t] as number);
      expect(coverage(VOUCHER_MODEL, suite, t, VOUCHER_CONSTRAINTS).percent).toBe(100);
      expect(suite.every(VOUCHER_CONSTRAINTS)).toBe(true);
    });
  }
});

describe('TEST-038 · sức bắt lỗi trên bốn bản build có lỗi gài', () => {
  const count = (rows: readonly Row[]) => Object.fromEntries(VARIANTS.map((v) => [v, redCount(v, rows)]));

  it('t = 1 chỉ chắc chắn bắt lỗi bậc 1 (V1)', () => {
    expect(count(generate(VOUCHER_MODEL, 1, VOUCHER_CONSTRAINTS))).toEqual({
      dung: 0, 'V1-trim': 1, 'V2-ngay-utc': 1, 'V3-pham-vi': 0, 'V4-luot-utc': 0,
    });
  });

  it('t = 2 chắc chắn bắt V1–V3; V4 (bậc 3) bị bắt ở bộ này là do tình cờ', () => {
    expect(count(generate(VOUCHER_MODEL, 2, VOUCHER_CONSTRAINTS))).toEqual({
      dung: 0, 'V1-trim': 4, 'V2-ngay-utc': 2, 'V3-pham-vi': 1, 'V4-luot-utc': 1,
    });
  });

  it('t = 3 chắc chắn bắt cả V4', () => {
    expect(count(generate(VOUCHER_MODEL, 3, VOUCHER_CONSTRAINTS))).toEqual({
      dung: 0, 'V1-trim': 15, 'V2-ngay-utc': 4, 'V3-pham-vi': 10, 'V4-luot-utc': 4,
    });
  });

  it('7 test âm (base choice) chỉ bắt V3 — chúng kiểm luồng lỗi, không kiểm tổ hợp hợp lệ', () => {
    const negatives = negativeCases();
    expect(VARIANTS.map((v) => negatives.filter((s) => fails(v, s)).length)).toEqual([0, 0, 0, 1, 0]);
  });
});

describe('TEST-038 · trộn giá trị lỗi vào mô hình làm che tổ hợp', () => {
  it('3-wise trên mô hình trộn: 70 test, 50 test dừng ở lỗi voucher, chỉ 1.741/2.089 bộ-3 hợp lệ chạy tới cuối', () => {
    const suite = generate(MIXED_MODEL, 3, VOUCHER_CONSTRAINTS);
    const reached = suite.filter((r) => toScenario(MIXED_MODEL, r, '').expected.ok);
    expect(suite).toHaveLength(70);
    expect(suite.length - reached.length).toBe(50);
    expect(coverage(VOUCHER_MODEL, reached, 3, VOUCHER_CONSTRAINTS).covered).toBe(1_741);
  });
});
