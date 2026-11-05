/**
 * TEST-031 · EP ở mức component: calculateFare (src/domain/fare.ts).
 * Hàm nhận `distanceKm: number` nên phân vùng «không phải số» của API biến thành «số không hữu hạn».
 * Thêm phân vùng ĐẦU RA: xe máy có vùng «bị nâng lên cước tối thiểu», ô tô thì vùng đó RỖNG.
 */
import { describe, expect, it } from 'vitest';
import { calculateFare } from '../../src/domain/fare';
import { DomainError } from '../../src/domain/errors';

const noExtras = { peak: false, raining: false } as const;

function codeOf(fn: () => unknown): string | null {
  try {
    fn();
    return null;
  } catch (error) {
    return error instanceof DomainError ? error.code : 'KHAC';
  }
}

describe('Phân vùng đầu vào của quãng đường (R-FARE-02)', () => {
  it.each([
    { id: 'EPC-01', partition: 'D1 < 0,5 km', km: 0.2 },
    { id: 'EPC-02', partition: 'D1 (số âm, cùng phân vùng)', km: -3 },
    { id: 'EPC-03', partition: 'D3 > 100 km', km: 150 },
    { id: 'EPC-04', partition: 'D4′ không hữu hạn', km: Number.NaN },
    { id: 'EPC-05', partition: 'D4′ không hữu hạn', km: Number.POSITIVE_INFINITY },
  ])('$id $partition: $km km ⇒ DISTANCE_OUT_OF_RANGE', ({ km }) => {
    expect(codeOf(() => calculateFare({ vehicle: 'bike', distanceKm: km, ...noExtras }))).toBe('DISTANCE_OUT_OF_RANGE');
  });

  it('EPC-06 D2 hợp lệ: xe máy 7,4 km ⇒ 46.000 đ', () => {
    expect(calculateFare({ vehicle: 'bike', distanceKm: 7.4, ...noExtras }).total).toBe(46_000);
  });
});

describe('Phân vùng đầu ra «có bị nâng lên cước tối thiểu không» (R-FARE-07)', () => {
  it.each([
    { id: 'EPO-01', vehicle: 'bike', km: 0.5, minApplied: true, total: 15_000 }, // 14.250 < 15.000
    { id: 'EPO-02', vehicle: 'bike', km: 7.4, minApplied: false, total: 46_000 },
    { id: 'EPO-03', vehicle: 'car', km: 0.5, minApplied: false, total: 31_000 }, // 30.500 > 30.000
  ] as const)('$id $vehicle $km km ⇒ tối thiểu áp dụng: $minApplied, tổng $total', ({ vehicle, km, minApplied, total }) => {
    const fare = calculateFare({ vehicle, distanceKm: km, ...noExtras });
    expect(fare.afterDiscount < fare.minimumFare).toBe(minApplied);
    expect(fare.total).toBe(total);
  });

  it('EPO-04 với ô tô, phân vùng «bị nâng lên tối thiểu» là RỖNG trên toàn miền hợp lệ', () => {
    // Quét mọi quãng đường hợp lệ ở độ chính xác 0,1 km: 0,5 … 100,0 (996 giá trị)
    const hits: number[] = [];
    for (let tenths = 5; tenths <= 1000; tenths++) {
      const fare = calculateFare({ vehicle: 'car', distanceKm: tenths / 10, ...noExtras });
      if (fare.afterDiscount < fare.minimumFare) hits.push(tenths / 10);
    }
    expect(hits).toEqual([]);
  });
});
