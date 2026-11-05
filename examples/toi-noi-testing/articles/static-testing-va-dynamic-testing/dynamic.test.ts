/**
 * DYNAMIC TESTING trên bản có lỗi: chạy calculateFareBuggy và so kết quả với SPEC.
 *
 * Ca mà bản lỗi trả sai được đánh `it.fails` (failure đã biết) để `npm test` của lab vẫn xanh.
 * Muốn thấy màu đỏ thật như lần chạy đầu:
 *   SHOW_RED=1 npx vitest run articles/static-testing-va-dynamic-testing
 */
import { describe, expect, it } from 'vitest';
import { calculateFare } from '../../src/domain/fare';
import { calculateFareBuggy } from './buggy-fare';
import { SINGLE_CONDITION_CASES, TC_CAR, TC_COMBO } from './cases';

const showRed = process.env.SHOW_RED === '1';
const knownFailure = showRed ? it : it.fails;

describe('Bước 1 · bộ test mỗi ca một điều kiện — XANH trên bản lỗi', () => {
  it.each(SINGLE_CONDITION_CASES)('$id ($basis) $title ⇒ $expectedTotal đ', (tc) => {
    expect(calculateFareBuggy(tc.input).total).toBe(tc.expectedTotal);
  });
});

describe('Bước 2 · thêm ca tổ hợp và ca ô tô — ĐỎ trên bản lỗi', () => {
  knownFailure(`${TC_COMBO.id} (${TC_COMBO.basis}) ${TC_COMBO.title} ⇒ ${TC_COMBO.expectedTotal} đ`, () => {
    expect(calculateFareBuggy(TC_COMBO.input).total).toBe(TC_COMBO.expectedTotal);
  });
  knownFailure(`${TC_CAR.id} (${TC_CAR.basis}) ${TC_CAR.title} ⇒ ${TC_CAR.expectedTotal} đ`, () => {
    expect(calculateFareBuggy(TC_CAR.input).total).toBe(TC_CAR.expectedTotal);
  });
});

describe('Bước 3 · phân tích failure: so bảng chi tiết với R-FARE-09', () => {
  it('TC-05: cước cơ bản khớp SPEC, lệch bắt đầu từ «sau cao điểm» ⇒ defect ở bước 2–3', () => {
    const b = calculateFareBuggy(TC_COMBO.input);
    expect(b.base).toBeCloseTo(45_300, 6); // SPEC 45.300 — khớp
    expect(b.afterPeak).toBeCloseTo(66_625, 6); // SPEC 56.625 — lệch 10.000 = 8.000 × 1,25
    expect(b.total).toBe(54_000); // SPEC 52.000
  });
  it('TC-06: trước làm tròn là 287.000,000…06 thay vì 287.000 ⇒ defect ở cách biểu diễn số', () => {
    const b = calculateFareBuggy(TC_CAR.input);
    expect(b.afterPeak).not.toBe(287_000);
    expect(b.afterPeak - 287_000).toBeGreaterThan(0);
    expect(b.afterPeak - 287_000).toBeLessThan(1e-6);
    expect(b.total).toBe(288_000);
  });
});

describe('Đối chứng · bản đúng src/domain/fare.ts qua cả sáu ca', () => {
  it.each([...SINGLE_CONDITION_CASES, TC_COMBO, TC_CAR])('$id ⇒ $expectedTotal đ', (tc) => {
    expect(calculateFare(tc.input).total).toBe(tc.expectedTotal);
  });
});
