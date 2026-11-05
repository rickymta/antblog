/**
 * Quét toàn miền quãng đường hợp lệ (0,5–100 km, bước 0,1 km = 996 giá trị) để xem D2 (số thực) lộ ở đâu.
 * So bản lỗi với bản đúng src/domain/fare.ts — không cần biết trước kết quả mong đợi từng ca.
 */
import { describe, expect, it } from 'vitest';
import { calculateFare, type FareInput } from '../../src/domain/fare';
import { calculateFareBuggy } from './buggy-fare';

function mismatches(vehicle: FareInput['vehicle'], peak: boolean): number[] {
  const found: number[] = [];
  for (let tenths = 5; tenths <= 1000; tenths++) {
    const input: FareInput = { vehicle, distanceKm: tenths / 10, peak, raining: false };
    if (calculateFareBuggy(input).total !== calculateFare(input).total) found.push(tenths / 10);
  }
  return found;
}

describe('D2 chỉ lộ ở một số ít quãng đường', () => {
  it('ô tô, chỉ cao điểm: 12/996 quãng lộ sai 1.000 đ', () => {
    expect(mismatches('car', true)).toEqual([18.6, 32.2, 36.2, 40.2, 44.2, 67.4, 71.4, 75.4, 79.4, 83.4, 87.4, 91.4]);
  });
  it('ô tô không cao điểm, xe máy có hoặc không cao điểm: 0 quãng lộ', () => {
    expect(mismatches('car', false)).toEqual([]);
    expect(mismatches('bike', true)).toEqual([]);
    expect(mismatches('bike', false)).toEqual([]);
  });
});
