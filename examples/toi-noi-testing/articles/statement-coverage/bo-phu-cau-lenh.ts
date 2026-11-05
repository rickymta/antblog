/**
 * TEST-043 · Bốn ca kiểm thử thiết kế để phủ 100% câu lệnh của `src/domain/pricing.ts`.
 * Bộ ca được viết thành hàm nhận vào «một bản cài đặt» để chạy y nguyên trên mã thật của lab
 * và trên bản có lỗi `pricing-loi.ts` (cùng chữ ký).
 */
import { expect, it } from 'vitest';
import type { normalizePriceRequest, priceRequest } from '../../src/domain/pricing';

export interface PricingImpl {
  normalizePriceRequest: typeof normalizePriceRequest;
  priceRequest: typeof priceRequest;
}

const GIAM20 = { code: 'GIAM20', percent: 20, maxDiscount: 30_000, minAmount: 0 };
const FOOD15 = { code: 'FOOD15', percent: 15, maxDiscount: 25_000, minAmount: 50_000 };

export function boPhuCauLenh(p: PricingImpl): void {
  it('TC-01 · đặt xe máy 7,4 km, cao điểm, mưa, GIAM20 ⇒ 52.000 đ (R-FARE-09)', () => {
    const req = p.normalizePriceRequest({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 });
    const price = p.priceRequest(req, { peak: true, raining: true, voucher: GIAM20 });
    expect(price).toMatchObject({ discount: 12_925, total: 52_000, itemsSubtotal: null, deliveryFee: null });
  });

  it('TC-02 · thành phố chưa phục vụ ⇒ CITY_NOT_SUPPORTED (R-VALID-01)', () => {
    expect(() => p.normalizePriceRequest({ kind: 'ride', cityCode: 'DL', vehicle: 'bike', distanceKm: 5 })).toThrow(
      expect.objectContaining({ code: 'CITY_NOT_SUPPORTED' }),
    );
  });

  it('TC-03 · bưu kiện 6,1 kg ô tô 10 km cao điểm ⇒ phụ phí 4.000 đ bị nhân 1,25 ⇒ 174.000 đ (R-PARCEL-03/04)', () => {
    const req = p.normalizePriceRequest({ kind: 'parcel', cityCode: 'HN', vehicle: 'car', distanceKm: 10, parcelWeightKg: 6.1 });
    const price = p.priceRequest(req, { peak: true, raining: false });
    expect(price.fare).toMatchObject({ parcelSurcharge: 4_000, base: 139_000, afterPeak: 173_750 });
    expect(price.total).toBe(174_000);
  });

  it('TC-04 · đồ ăn 2 × 50.000 đ, 3 km, FOOD15 ⇒ 100.000 − 15.000 + 26.000 = 111.000 đ (R-FOOD-01/04/05)', () => {
    const items = [{ name: 'Cơm tấm', unitPrice: 50_000, quantity: 2 }];
    const req = p.normalizePriceRequest({ kind: 'food', cityCode: 'DN', distanceKm: 3, items });
    const price = p.priceRequest(req, { peak: false, raining: false, voucher: FOOD15 });
    expect(price).toMatchObject({ itemsSubtotal: 100_000, itemsDiscount: 15_000, deliveryFee: 26_000, total: 111_000 });
    expect(req.vehicle).toBe('bike');
  });
}

/** Ca biên dữ liệu: 20,04 kg làm tròn thành 20,0 kg ⇒ xe máy vẫn hợp lệ (R-PARCEL-01, R-PARCEL-02). */
export function tc05BienKhoiLuong(p: PricingImpl): void {
  const req = p.normalizePriceRequest({ kind: 'parcel', cityCode: 'SG', vehicle: 'bike', distanceKm: 5, parcelWeightKg: 20.04 });
  expect(req).toMatchObject({ vehicle: 'bike', parcelWeightKg: 20 });
}
