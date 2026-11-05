/**
 * BẢN THAY ĐỔI ĐỀ XUẤT «fare-2026.2» — chỉ dùng cho bài TEST-020 «Maintenance Testing».
 *
 * Mô phỏng commit sẽ sửa src/domain/fare.ts, src/domain/types.ts và src/domain/pricing.ts khi
 * Tới Nơi (1) tăng giá mỗi km xe máy 4.500 đ ⇒ 5.000 đ (con số GIẢ ĐỊNH của bài) và
 * (2) ngừng phục vụ Nha Trang (NT). Không sửa src/: bản hiện hành của lab là «trước thay đổi».
 *
 * Sáu bước của R-FARE-03 giữ nguyên; chỉ bảng giá và danh sách thành phố đổi.
 */
import { DomainError } from '../../src/domain/errors';
import { FARE_TABLE, normalizeDistanceKm, PEAK_PERCENT, RAIN_SURCHARGE, type FareBreakdown, type FareInput } from '../../src/domain/fare';
import { percentOf, roundUpToThousandDong, toDong, toMilli } from '../../src/domain/money';
import { CITY_CODES } from '../../src/domain/types';
import { voucherDiscountMilli } from '../../src/domain/voucher';

export const PRICING_VERSION_2026_2 = 'fare-2026.2';

/** R-FARE-01 sau thay đổi: chỉ giá mỗi km xe máy đổi. */
export const FARE_TABLE_2026_2 = {
  bike: { ...FARE_TABLE.bike, perKm: 5_000 },
  car: FARE_TABLE.car,
} as const;

/** R-GEN-03 sau thay đổi: bỏ Nha Trang. */
export const CITY_CODES_2026_2 = CITY_CODES.filter((code) => code !== 'NT');

export function assertCitySupported20262(cityCode: string): void {
  if (!(CITY_CODES_2026_2 as readonly string[]).includes(cityCode)) {
    throw new DomainError('CITY_NOT_SUPPORTED', `Tới Nơi chưa phục vụ thành phố "${cityCode}".`);
  }
}

/** Cùng công thức R-FARE-03, bảng giá fare-2026.2. */
export function calculateFare20262(input: FareInput): FareBreakdown {
  const distanceKm = normalizeDistanceKm(input.distanceKm);
  const prices = FARE_TABLE_2026_2[input.vehicle];

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
  const total = roundUpToThousandDong(Math.max(afterDiscount, minimumFare)); // bước 6

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
