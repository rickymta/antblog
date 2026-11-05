/**
 * TEST-037 — Pairwise Testing: kiểm chính bộ test được sinh (độ phủ cặp, số hàng) và
 * đo xem bộ nào chạm tới các lỗi tương tác GIẢ LẬP F1…F5 (app-config-model.ts).
 */
import { describe, expect, it } from 'vitest';
import { APP_MODEL, faultsFound, L9_SUITE } from './app-config-model';
import { allRows, coverage, generate, label } from './pairwise';

describe('TEST-037 · mô hình cấu hình ứng dụng khách', () => {
  it('tổ hợp đầy đủ = 3 × 3 × 2 × 2 × 3 = 108', () => {
    expect(allRows(APP_MODEL)).toHaveLength(108);
  });

  it('có 67 cặp giá trị cần phủ (tổng tích số giá trị của mọi cặp tham số)', () => {
    expect(coverage(APP_MODEL, [], 2).feasible).toBe(67);
  });
});

describe('TEST-037 · each-choice và pairwise', () => {
  it('each-choice: 3 hàng phủ mọi giá trị nhưng chỉ 29/67 cặp', () => {
    const suite = generate(APP_MODEL, 1);
    expect(suite).toHaveLength(3);
    expect(coverage(APP_MODEL, suite, 1).percent).toBe(100);
    expect(coverage(APP_MODEL, suite, 2).covered).toBe(29);
  });

  it('bộ sinh tham lam: 100% cặp với 12 hàng (≈ 11% tổ hợp đầy đủ)', () => {
    const suite = generate(APP_MODEL, 2);
    expect(coverage(APP_MODEL, suite, 2).percent).toBe(100);
    expect(suite).toHaveLength(12);
  });

  it('bộ L9 dựng tay: 100% cặp với 9 hàng — chạm cận dưới 3 × 3', () => {
    expect(L9_SUITE).toHaveLength(9);
    expect(coverage(APP_MODEL, L9_SUITE, 2).percent).toBe(100);
  });

  it('L9 không chứa hàng trùng và mọi chỉ số giá trị đều hợp lệ', () => {
    const keys = L9_SUITE.map((row) => JSON.stringify(label(APP_MODEL, row)));
    expect(new Set(keys).size).toBe(9);
    L9_SUITE.forEach((row) => row.forEach((v, p) => expect(APP_MODEL[p]?.values[v]).toBeDefined()));
  });
});

describe('TEST-037 · lỗi tương tác giả lập', () => {
  it('mọi bộ phủ đủ cặp đều chạm F1 (1 tham số) và F2, F3 (2 tham số)', () => {
    for (const suite of [generate(APP_MODEL, 2), L9_SUITE]) {
      expect(faultsFound(suite)).toEqual(expect.arrayContaining(['F1', 'F2', 'F3']));
    }
  });

  it('lỗi 3–4 tham số: chạm hay không là do may — hai bộ pairwise cho kết quả khác nhau', () => {
    expect(faultsFound(generate(APP_MODEL, 2))).toEqual(['F1', 'F2', 'F3', 'F5']);
    expect(faultsFound(L9_SUITE)).toEqual(['F1', 'F2', 'F3', 'F4']);
  });

  it('3-wise chắc chắn chạm F4 (3 tham số); bộ này cũng chạm F5', () => {
    const suite = generate(APP_MODEL, 3);
    expect(suite).toHaveLength(28);
    expect(faultsFound(suite)).toEqual(['F1', 'F2', 'F3', 'F4', 'F5']);
  });
});
