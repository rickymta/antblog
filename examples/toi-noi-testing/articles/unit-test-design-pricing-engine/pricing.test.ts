import { describe, expect, it } from 'vitest';
import { calculateFare, normalizeDistanceKm } from '../../src/domain/fare';
import { normalizePriceRequest, priceRequest } from '../../src/domain/pricing';
import { roundUpToThousandDong } from '../../src/domain/money';
import { quoteSchema } from '../../src/api/schemas';

describe('R-FARE-02 — biên sau chuẩn hoá', () => {
  it.each([
    [0.45, 0.5], [0.5, 0.5], [0.55, 0.6],
    [99.94, 99.9], [99.95, 100], [100, 100], [100.04, 100], [7.35, 7.4],
  ])('%s km được chuẩn hoá thành %s km', (raw, expected) => {
    expect(normalizeDistanceKm(raw)).toBe(expected);
  });

  it.each([0.44, 100.05, -1, NaN, Infinity, -Infinity])('từ chối %s km', raw => {
    expect(() => normalizeDistanceKm(raw)).toThrowError(expect.objectContaining({ code: 'DISTANCE_OUT_OF_RANGE' }));
  });
});

describe('R-FARE-03/04/05/06 — bảng quyết định xe máy 2 km', () => {
  it.each([
    [false, false, false, 21_000], [false, false, true, 17_000],
    [false, true, false, 29_000], [false, true, true, 24_000],
    [true, false, false, 27_000], [true, false, true, 21_000],
    [true, true, false, 35_000], [true, true, true, 28_000],
  ] as const)('peak=%s rain=%s voucher=%s → %s đ', (peak, raining, hasVoucher, total) => {
    const voucher = hasVoucher ? { code: 'TEST20', percent: 20, maxDiscount: 100_000, minAmount: 0 } : null;
    const request = normalizePriceRequest({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 2 });
    expect(priceRequest(request, { peak, raining, voucher }).total).toBe(total);
  });
});

describe('oracle và tiền tệ', () => {
  it('R-FARE-09: đối chiếu từng bước của ví dụ chuẩn', () => {
    const fare = calculateFare({ vehicle: 'bike', distanceKm: 7.4, peak: true, raining: true,
      voucher: { code: 'TEST20', percent: 20, maxDiscount: 100_000, minAmount: 0 } });
    expect(fare).toMatchObject({ base: 45_300, afterPeak: 56_625, beforeDiscount: 64_625,
      discount: 12_925, afterDiscount: 51_700, total: 52_000 });
  });

  it('R-FARE-10: giữ 17.812,5 đ ở bước trung gian', () => {
    const fare = calculateFare({ vehicle: 'bike', distanceKm: 0.5, peak: true, raining: false });
    expect(fare).toMatchObject({ afterPeak: 17_812.5, total: 18_000 });
  });

  it.each([['bike', 15_000], ['car', 30_000]] as const)('R-FARE-07: %s vẫn có cước sàn sau voucher', (vehicle, minimum) => {
    const fare = calculateFare({ vehicle, distanceKm: 0.5, peak: false, raining: false,
      voucher: { code: 'HALF', percent: 50, maxDiscount: 100_000, minAmount: 0 } });
    expect(fare.afterDiscount).toBeLessThan(minimum);
    expect(fare.total).toBe(minimum);
  });

  it('R-VOUCHER-06: tiền giảm không vượt trần', () => {
    const fare = calculateFare({ vehicle: 'bike', distanceKm: 2, peak: false, raining: false,
      voucher: { code: 'CAP', percent: 20, maxDiscount: 3_000, minAmount: 0 } });
    expect(fare).toMatchObject({ discount: 3_000, total: 18_000 });
  });

  it.each([[51_700_000, 52_000], [52_000_000, 52_000], [52_001_000, 53_000]])(
    'R-FARE-08: %s milli-đồng → %s đồng', (milli, dong) => {
      expect(roundUpToThousandDong(milli)).toBe(dong);
    });
});

describe('ranh giới schema và domain', () => {
  it.each([null, '2', undefined, true])('schema từ chối distanceKm=%s', distanceKm => {
    const result = quoteSchema.safeParse({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm });
    expect(result.success).toBe(false);
    if (!result.success) expect(result.error.issues.some(issue => issue.path.join('.') === 'distanceKm')).toBe(true);
  });

  it('R-VALID-01: thành phố sai được báo trước quãng đường sai', () => {
    expect(() => normalizePriceRequest({ kind: 'ride', cityCode: 'XX', vehicle: 'bike', distanceKm: 0 }))
      .toThrowError(expect.objectContaining({ code: 'CITY_NOT_SUPPORTED' }));
  });

  it('R-FARE-01: pricing điều phối giá ô tô', () => {
    const request = normalizePriceRequest({ kind: 'ride', cityCode: 'HN', vehicle: 'car', distanceKm: 2 });
    expect(priceRequest(request, { peak: false, raining: false })).toMatchObject({ total: 47_000, itemsSubtotal: null });
  });
});
