/**
 * TEST-042 · Lõi định giá thật và bốn bản CÓ LỖI (chép từ src/domain/fare.ts rồi gài lỗi).
 * Không sửa src/ — bản lỗi chỉ dùng trong thư mục bài.
 *
 *  B1 rain-times-peak          cộng mưa trước khi nhân 1,25 (vi phạm R-FARE-05) — lỗi «lọt» ở TEST-041
 *  B2 minimum-before-voucher   nâng lên cước tối thiểu trước khi trừ voucher (vi phạm R-FARE-07)
 *  B3 round-nearest            làm tròn tới 1.000 đ GẦN NHẤT thay vì làm tròn LÊN (vi phạm R-FARE-08)
 *  B4 voucher-on-base          tính tiền giảm trên cước cơ bản thay vì số tiền sau bước 3 (vi phạm R-FARE-06)
 */
import { FARE_TABLE, normalizeDistanceKm, type FareBreakdown } from '../../src/domain/fare';
import { itemsSubtotal } from '../../src/domain/food';
import { percentOf, roundUpToThousandDong, toDong, toMilli } from '../../src/domain/money';
import { parcelSurcharge } from '../../src/domain/parcel';
import { normalizePriceRequest, priceRequest, type PriceRequest, type PriceResult } from '../../src/domain/pricing';
import type { Vehicle } from '../../src/domain/types';
import { voucherDiscountMilli, type VoucherTerms } from '../../src/domain/voucher';

export interface Conditions {
  peak: boolean;
  raining: boolean;
  voucher: VoucherTerms | null;
}
export type PriceFn = (req: PriceRequest, c: Conditions) => PriceResult;
export type Bug = 'B1 rain-times-peak' | 'B2 minimum-before-voucher' | 'B3 round-nearest' | 'B4 voucher-on-base';
export const BUGS: Bug[] = ['B1 rain-times-peak', 'B2 minimum-before-voucher', 'B3 round-nearest', 'B4 voucher-on-base'];

/** Lõi định giá thật của lab. */
export const srcPrice: PriceFn = (req, c) => priceRequest(normalizePriceRequest(req), c);

function buggyFare(
  bug: Bug,
  i: { vehicle: Vehicle; distanceKm: number; surcharge: number } & Conditions,
): FareBreakdown {
  const distanceKm = normalizeDistanceKm(i.distanceKm);
  const p = FARE_TABLE[i.vehicle];
  const openingFee = toMilli(p.openingFee);
  const distanceCharge = toMilli(p.perKm * distanceKm);
  const surcharge = toMilli(i.surcharge);
  const base = openingFee + distanceCharge + surcharge;
  const rain = i.raining ? toMilli(8_000) : 0;
  const afterPeak = i.peak ? percentOf(base, 125) : base;
  const beforeDiscount =
    bug === 'B1 rain-times-peak' ? (i.peak ? percentOf(base + rain, 125) : base + rain) : afterPeak + rain;
  const discountBase = bug === 'B4 voucher-on-base' ? base : beforeDiscount;
  const discount = i.voucher ? voucherDiscountMilli(i.voucher, discountBase) : 0;
  const minimum = toMilli(p.minimumFare);
  const afterDiscount =
    bug === 'B2 minimum-before-voucher' ? Math.max(beforeDiscount, minimum) - discount : beforeDiscount - discount;
  const raised = bug === 'B2 minimum-before-voucher' ? afterDiscount : Math.max(afterDiscount, minimum);
  const total = bug === 'B3 round-nearest' ? Math.round(raised / 1_000_000) * 1_000 : roundUpToThousandDong(raised);
  return {
    vehicle: i.vehicle, distanceKm,
    openingFee: toDong(openingFee), distanceCharge: toDong(distanceCharge), parcelSurcharge: toDong(surcharge),
    base: toDong(base), afterPeak: toDong(afterPeak), rainSurcharge: toDong(rain),
    beforeDiscount: toDong(beforeDiscount), discount: toDong(discount), afterDiscount: toDong(afterDiscount),
    minimumFare: p.minimumFare, total,
  };
}

/** Lõi định giá dùng công thức có lỗi; phần kiểm đầu vào giữ nguyên của src. */
export function buggyPrice(bug: Bug): PriceFn {
  return (req, c) => {
    const n = normalizePriceRequest(req);
    if (n.kind === 'food') {
      const subtotal = itemsSubtotal(n.items ?? []);
      const itemsDiscount = c.voucher ? toDong(voucherDiscountMilli(c.voucher, toMilli(subtotal))) : 0;
      const fee = buggyFare(bug, { vehicle: 'bike', distanceKm: n.distanceKm, surcharge: 0, peak: c.peak, raining: c.raining, voucher: null });
      return { fare: fee, itemsSubtotal: subtotal, itemsDiscount, deliveryFee: fee.total, discount: itemsDiscount, total: subtotal - itemsDiscount + fee.total };
    }
    const surcharge = n.parcelWeightKg === null ? 0 : parcelSurcharge(n.parcelWeightKg);
    const fare = buggyFare(bug, { vehicle: n.vehicle, distanceKm: n.distanceKm, surcharge, ...c });
    return { fare, itemsSubtotal: null, itemsDiscount: null, deliveryFee: null, discount: fare.discount, total: fare.total };
  };
}
