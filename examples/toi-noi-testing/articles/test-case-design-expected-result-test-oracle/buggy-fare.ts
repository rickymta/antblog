/**
 * TCD-04 — bản chép `calculateFare` (src/domain/fare.ts) có MỘT defect cố ý, đóng vai «build rc»:
 * phụ phí mưa bị cộng TRƯỚC khi nhân hệ số cao điểm (đúng ra R-FARE-05: cộng sau, không bị nhân 1,25).
 * Chỉ sai khi cao điểm VÀ mưa cùng lúc.
 */
import { FARE_TABLE, normalizeDistanceKm, PEAK_PERCENT, RAIN_SURCHARGE, type FareBreakdown, type FareInput } from '../../src/domain/fare';
import { percentOf, roundUpToThousandDong, toDong, toMilli } from '../../src/domain/money';
import { voucherDiscountMilli } from '../../src/domain/voucher';

export function buggyCalculateFare(input: FareInput): FareBreakdown {
  const distanceKm = normalizeDistanceKm(input.distanceKm);
  const prices = FARE_TABLE[input.vehicle];
  const openingFee = toMilli(prices.openingFee);
  const distanceCharge = toMilli(prices.perKm * distanceKm);
  const parcelSurcharge = toMilli(input.parcelSurcharge ?? 0);
  const base = openingFee + distanceCharge + parcelSurcharge;
  const rainSurcharge = input.raining ? toMilli(RAIN_SURCHARGE) : 0;
  // DEFECT: khi cao điểm, mưa bị gộp vào trước khi nhân 1,25
  const afterPeak = input.peak ? percentOf(base + rainSurcharge, PEAK_PERCENT) : base;
  const beforeDiscount = input.peak ? afterPeak : afterPeak + rainSurcharge;
  const discount = input.voucher ? voucherDiscountMilli(input.voucher, beforeDiscount) : 0;
  const afterDiscount = beforeDiscount - discount;
  const minimumFare = toMilli(prices.minimumFare);
  const total = roundUpToThousandDong(Math.max(afterDiscount, minimumFare));
  return {
    vehicle: input.vehicle, distanceKm,
    openingFee: toDong(openingFee), distanceCharge: toDong(distanceCharge), parcelSurcharge: toDong(parcelSurcharge),
    base: toDong(base), afterPeak: toDong(afterPeak), rainSurcharge: toDong(rainSurcharge),
    beforeDiscount: toDong(beforeDiscount), discount: toDong(discount), afterDiscount: toDong(afterDiscount),
    minimumFare: prices.minimumFare, total,
  };
}
