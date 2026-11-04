/**
 * BẢN CÓ LỖI — chỉ dùng cho bài TEST-001 «Software Testing là gì? Testing khác Debugging».
 *
 * Đây là bản chép của `calculateFare` (src/domain/fare.ts) mà một developer giả định viết
 * ở sprint đầu. Sáu bước của SPEC R-FARE-03 giống hệt bản đúng, chỉ khác bước 6:
 * developer hiểu «làm tròn tới bội số 1.000 đ» là bỏ phần lẻ (làm tròn XUỐNG),
 * trong khi SPEC R-FARE-08 yêu cầu làm tròn LÊN.
 *
 * KHÔNG sửa src/: bản đúng của lab chính là bản «sau khi sửa» trong bài.
 */
import {
  FARE_TABLE,
  normalizeDistanceKm,
  PEAK_PERCENT,
  RAIN_SURCHARGE,
  type FareBreakdown,
  type FareInput,
} from '../../src/domain/fare';
import { MILLI_PER_DONG, percentOf, toDong, toMilli } from '../../src/domain/money';
import { voucherDiscountMilli } from '../../src/domain/voucher';

/** Defect nằm ở đây: Math.floor thay vì Math.ceil. */
export function roundToThousandDongBuggy(milli: number): number {
  const thousandsOfDong = Math.floor(milli / (1000 * MILLI_PER_DONG));
  return thousandsOfDong * 1000;
}

export function calculateFareBuggy(input: FareInput): FareBreakdown {
  const distanceKm = normalizeDistanceKm(input.distanceKm);
  const prices = FARE_TABLE[input.vehicle];

  const openingFee = toMilli(prices.openingFee);
  const distanceCharge = toMilli(prices.perKm * distanceKm);
  const parcelSurcharge = toMilli(input.parcelSurcharge ?? 0);
  const base = openingFee + distanceCharge + parcelSurcharge; // bước 1
  const afterPeak = input.peak ? percentOf(base, PEAK_PERCENT) : base; // bước 2
  const rainSurcharge = input.raining ? toMilli(RAIN_SURCHARGE) : 0; // bước 3
  const beforeDiscount = afterPeak + rainSurcharge;
  const discount = input.voucher ? voucherDiscountMilli(input.voucher, beforeDiscount) : 0; // bước 4
  const afterDiscount = beforeDiscount - discount;
  const minimumFare = toMilli(prices.minimumFare); // bước 5
  const atLeastMinimum = Math.max(afterDiscount, minimumFare);
  const total = roundToThousandDongBuggy(atLeastMinimum); // bước 6 — CÓ LỖI

  return {
    vehicle: input.vehicle,
    distanceKm,
    openingFee: toDong(openingFee),
    distanceCharge: toDong(distanceCharge),
    parcelSurcharge: toDong(parcelSurcharge),
    base: toDong(base),
    afterPeak: toDong(afterPeak),
    rainSurcharge: toDong(rainSurcharge),
    beforeDiscount: toDong(beforeDiscount),
    discount: toDong(discount),
    afterDiscount: toDong(afterDiscount),
    minimumFare: prices.minimumFare,
    total,
  };
}
