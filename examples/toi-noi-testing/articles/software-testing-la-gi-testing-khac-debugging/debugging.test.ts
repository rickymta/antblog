/**
 * DEBUGGING (việc của developer): tái hiện → chẩn đoán (tìm defect) → sửa.
 * Sau đó tester chạy CONFIRMATION TEST và REGRESSION TEST trên bản đã sửa.
 * Bản đã sửa trong bài chính là src/domain/fare.ts của lab.
 */
import { describe, expect, it } from 'vitest';
import { calculateFare } from '../../src/domain/fare';
import { roundUpToThousandDong, toMilli } from '../../src/domain/money';
import { calculateFareBuggy, roundToThousandDongBuggy } from './buggy-fare';
import { FARE_TEST_CASES } from './test-cases';

const tc01 = FARE_TEST_CASES[0]!;

describe('Debugging · bước 1 — tái hiện failure', () => {
  it('cùng đầu vào TC-FARE-01, bản có lỗi luôn trả 51.000 đ (tái hiện được, tất định)', () => {
    const results = [1, 2, 3].map(() => calculateFareBuggy(tc01.input).total);
    expect(results).toEqual([51_000, 51_000, 51_000]);
  });
});

describe('Debugging · bước 2 — chẩn đoán: so từng bước với SPEC R-FARE-09', () => {
  it('bước 1–5 khớp SPEC, chỉ total lệch ⇒ defect nằm ở bước 6 (làm tròn)', () => {
    const b = calculateFareBuggy(tc01.input);
    expect(b).toMatchObject({ base: 45_300, afterPeak: 56_625, beforeDiscount: 64_625, discount: 12_925, afterDiscount: 51_700 });
    expect(b.total).toBe(51_000); // SPEC: 52.000
  });

  it('cô lập hàm làm tròn bằng ví dụ của R-FARE-08: 51.700 ⇒ 52.000, 52.000 ⇒ 52.000, 52.001 ⇒ 53.000', () => {
    const inputs = [51_700, 52_000, 52_001].map(toMilli);
    expect(inputs.map(roundToThousandDongBuggy)).toEqual([51_000, 52_000, 52_000]); // bản có lỗi: làm tròn xuống
    expect(inputs.map(roundUpToThousandDong)).toEqual([52_000, 52_000, 53_000]); // đúng SPEC
  });
});

describe('Sau khi sửa · confirmation test + regression test (tester)', () => {
  it.each(FARE_TEST_CASES)('$id chạy lại trên bản đã sửa ⇒ $expectedTotal đ', (tc) => {
    expect(calculateFare(tc.input).total).toBe(tc.expectedTotal);
  });
});
