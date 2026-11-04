import { describe, expect, it } from 'vitest';
import { customerCancellationFee } from '../../src/domain/cancel';
import { calculateFare } from '../../src/domain/fare';
import { priceFood } from '../../src/domain/food';
import { parcelSurcharge } from '../../src/domain/parcel';
import { isPeakTime, vnTime } from '../../src/domain/time';
import { nextStatus } from '../../src/domain/trip-state';

const voucher20 = { code: 'GIAM20', percent: 20, maxDiscount: 30_000, minAmount: 0 };

describe('smoke · domain', () => {
  it('R-FARE-09: ví dụ kiểm 7,4 km xe máy, cao điểm, mưa, giảm 20% ⇒ 52.000 đ', () => {
    const fare = calculateFare({ vehicle: 'bike', distanceKm: 7.4, peak: true, raining: true, voucher: voucher20 });
    expect(fare).toMatchObject({ base: 45_300, afterPeak: 56_625, beforeDiscount: 64_625, discount: 12_925, afterDiscount: 51_700, total: 52_000 });
  });

  it('R-FARE-02: 0,45 km làm tròn thành 0,5 (hợp lệ), 0,44 km bị từ chối', () => {
    expect(calculateFare({ vehicle: 'bike', distanceKm: 0.45, peak: false, raining: false }).distanceKm).toBe(0.5);
    expect(() => calculateFare({ vehicle: 'bike', distanceKm: 0.44, peak: false, raining: false })).toThrow(
      expect.objectContaining({ code: 'DISTANCE_OUT_OF_RANGE' }),
    );
  });

  it('R-PARCEL-03: phụ phí theo kg bắt đầu vượt 5 kg', () => {
    expect([5, 5.1, 6, 6.1, 30].map(parcelSurcharge)).toEqual([0, 2_000, 2_000, 4_000, 50_000]);
  });

  it('R-FOOD-01: tổng = tiền món − giảm + phí giao (giảm không trừ vào phí giao)', () => {
    const price = priceFood({
      distanceKm: 3,
      items: [{ name: 'Cơm tấm', unitPrice: 50_000, quantity: 2 }],
      peak: false,
      raining: false,
      voucher: { code: 'FOOD15', percent: 15, maxDiscount: 25_000, minAmount: 50_000 },
    });
    expect(price).toMatchObject({ itemsSubtotal: 100_000, itemsDiscount: 15_000, total: 100_000 - 15_000 + 26_000 });
  });

  it('R-FARE-04: giờ cao điểm tính theo giờ Việt Nam, không theo múi giờ máy', () => {
    expect(isPeakTime(vnTime(2026, 11, 4, 11, 0))).toBe(true);
    expect(isPeakTime(vnTime(2026, 11, 4, 13, 0))).toBe(false);
    expect(isPeakTime(new Date('2026-11-04T10:30:00Z'))).toBe(true); // 17:30 giờ VN
  });

  it('R-STATE-03: chuyển không hợp lệ ném INVALID_TRANSITION', () => {
    expect(nextStatus('searching', 'accept')).toBe('accepted');
    expect(() => nextStatus('completed', 'customer_cancel')).toThrow(expect.objectContaining({ code: 'INVALID_TRANSITION' }));
  });

  it('R-CANCEL-02: đúng 2 phút sau khi nhận đã tính phí', () => {
    const acceptedAt = vnTime(2026, 11, 4, 9, 0);
    const at = (seconds: number) => new Date(acceptedAt.getTime() + seconds * 1000);
    const base = { status: 'accepted' as const, vehicle: 'bike' as const, acceptedAt, etaMinutes: 5 };
    expect(customerCancellationFee({ ...base, now: at(119) })).toEqual({ fee: 0, reason: 'grace_period' });
    expect(customerCancellationFee({ ...base, now: at(120) })).toEqual({ fee: 10_000, reason: 'charged' });
  });
});
