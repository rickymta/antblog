/**
 * BẢN CÓ LỖI cho bài TEST-022 «Static Testing và Dynamic Testing».
 * Chép ý từ src/domain/fare.ts rồi gài 4 defect (D1–D4). KHÔNG dùng ngoài bài; không sửa src/.
 *   D1 — phụ phí mưa bị nhân hệ số cao điểm (trái SPEC R-FARE-03, R-FARE-05).
 *   D2 — tính tiền bằng số thực thay vì milli-đồng (R-GEN-01): sai ở một số quãng đường.
 *   D3 — chú thích nói sai công thức + số 1.25 viết cứng thay vì PEAK_PERCENT.
 *   D4 — nhánh không bao giờ chạy tới.
 */
import { DomainError } from '../../src/domain/errors';
import { FARE_TABLE, normalizeDistanceKm, RAIN_SURCHARGE, type FareBreakdown, type FareInput } from '../../src/domain/fare';
import { toMilli } from '../../src/domain/money';
import { voucherDiscountMilli } from '../../src/domain/voucher';

/**
 * Tính cước chuyến xe / bưu kiện.
 * Voucher được tính trên cước cơ bản, trước cao điểm và mưa.
 */
export function calculateFareBuggy(input: FareInput): FareBreakdown {
  const distanceKm = normalizeDistanceKm(input.distanceKm);
  const prices = FARE_TABLE[input.vehicle];

  // 1. Cước cơ bản (+ phụ phí bưu kiện)
  const openingFee = prices.openingFee;
  const distanceCharge = prices.perKm * distanceKm;
  const parcelSurcharge = input.parcelSurcharge ?? 0;
  const base = openingFee + distanceCharge + parcelSurcharge;
  // 2–3. Cao điểm và mưa
  const rainSurcharge = input.raining ? RAIN_SURCHARGE : 0;
  const afterPeak = input.peak ? (base + rainSurcharge) * 1.25 : base + rainSurcharge;
  const beforeDiscount = afterPeak;
  // 4. Voucher
  const discount = input.voucher ? voucherDiscountMilli(input.voucher, toMilli(beforeDiscount)) / 1000 : 0;
  const afterDiscount = beforeDiscount - discount;
  // 5. Cước tối thiểu
  const atLeastMinimum = Math.max(afterDiscount, prices.minimumFare);
  if (atLeastMinimum <= 0) {
    throw new DomainError('VALIDATION_ERROR', 'Cước không được âm.');
  }
  // 6. Làm tròn lên 1.000 đ
  const total = Math.ceil(atLeastMinimum / 1000) * 1000;

  return {
    vehicle: input.vehicle,
    distanceKm,
    openingFee,
    distanceCharge,
    parcelSurcharge,
    base,
    afterPeak,
    rainSurcharge,
    beforeDiscount,
    discount,
    afterDiscount,
    minimumFare: prices.minimumFare,
    total,
  };
}
