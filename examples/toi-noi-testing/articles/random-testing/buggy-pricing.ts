/**
 * TEST-041 · Hai bản cài đặt CÓ LỖI của công thức cước (chép từ src/domain/fare.ts rồi gài lỗi).
 * Không sửa src/ — chỉ dùng trong thư mục bài.
 *
 *  - 'minimum-before-voucher': nâng lên cước tối thiểu TRƯỚC khi trừ voucher (đảo bước 4 và 5 của R-FARE-03)
 *    ⇒ voucher kéo được cước xuống dưới mức tối thiểu, vi phạm R-FARE-07.
 *  - 'rain-times-peak': cộng phụ phí mưa TRƯỚC khi nhân cao điểm ⇒ 8.000 đ bị nhân 1,25, vi phạm R-FARE-05.
 */
import { FARE_TABLE, normalizeDistanceKm, type FareBreakdown } from '../../src/domain/fare';
import { itemsSubtotal } from '../../src/domain/food';
import { percentOf, roundUpToThousandDong, toDong, toMilli } from '../../src/domain/money';
import { parcelSurcharge } from '../../src/domain/parcel';
import { normalizePriceRequest } from '../../src/domain/pricing';
import type { Vehicle } from '../../src/domain/types';
import { voucherDiscountMilli, type VoucherTerms } from '../../src/domain/voucher';
import type { PriceFn } from './oracle';

export type Bug = 'minimum-before-voucher' | 'rain-times-peak';

function buggyFare(
  bug: Bug,
  input: { vehicle: Vehicle; distanceKm: number; parcelSurcharge: number; peak: boolean; raining: boolean; voucher: VoucherTerms | null },
): FareBreakdown {
  const distanceKm = normalizeDistanceKm(input.distanceKm);
  const p = FARE_TABLE[input.vehicle];
  const openingFee = toMilli(p.openingFee);
  const distanceCharge = toMilli(p.perKm * distanceKm);
  const surcharge = toMilli(input.parcelSurcharge);
  const base = openingFee + distanceCharge + surcharge;
  const rain = input.raining ? toMilli(8_000) : 0;

  let afterPeak: number;
  let beforeDiscount: number;
  if (bug === 'rain-times-peak') {
    beforeDiscount = input.peak ? percentOf(base + rain, 125) : base + rain; // LỖI: mưa bị nhân 1,25
    afterPeak = input.peak ? percentOf(base, 125) : base;
  } else {
    afterPeak = input.peak ? percentOf(base, 125) : base;
    beforeDiscount = afterPeak + rain;
  }

  const minimum = toMilli(p.minimumFare);
  let discount: number;
  let afterDiscount: number;
  let total: number;
  if (bug === 'minimum-before-voucher') {
    const raised = Math.max(beforeDiscount, minimum); // LỖI: bước 5 chạy trước bước 4
    discount = input.voucher ? voucherDiscountMilli(input.voucher, beforeDiscount) : 0;
    afterDiscount = raised - discount;
    total = roundUpToThousandDong(afterDiscount);
  } else {
    discount = input.voucher ? voucherDiscountMilli(input.voucher, beforeDiscount) : 0;
    afterDiscount = beforeDiscount - discount;
    total = roundUpToThousandDong(Math.max(afterDiscount, minimum));
  }

  return {
    vehicle: input.vehicle,
    distanceKm,
    openingFee: toDong(openingFee),
    distanceCharge: toDong(distanceCharge),
    parcelSurcharge: toDong(surcharge),
    base: toDong(base),
    afterPeak: toDong(afterPeak),
    rainSurcharge: toDong(rain),
    beforeDiscount: toDong(beforeDiscount),
    discount: toDong(discount),
    afterDiscount: toDong(afterDiscount),
    minimumFare: p.minimumFare,
    total,
  };
}

/** Lõi định giá dùng công thức có lỗi; phần kiểm đầu vào giữ nguyên của src. */
export function buggyPrice(bug: Bug): PriceFn {
  return (req, c) => {
    const n = normalizePriceRequest(req);
    if (n.kind === 'food') {
      const items = n.items ?? [];
      const subtotal = itemsSubtotal(items);
      const itemsDiscount = c.voucher ? toDong(voucherDiscountMilli(c.voucher, toMilli(subtotal))) : 0;
      const fee = buggyFare(bug, { vehicle: 'bike', distanceKm: n.distanceKm, parcelSurcharge: 0, peak: c.peak, raining: c.raining, voucher: null });
      return { fare: fee, itemsSubtotal: subtotal, itemsDiscount, deliveryFee: fee.total, discount: itemsDiscount, total: subtotal - itemsDiscount + fee.total };
    }
    const surcharge = n.parcelWeightKg === null ? 0 : parcelSurcharge(n.parcelWeightKg);
    const fare = buggyFare(bug, { vehicle: n.vehicle, distanceKm: n.distanceKm, parcelSurcharge: surcharge, ...c });
    return { fare, itemsSubtotal: null, itemsDiscount: null, deliveryFee: null, discount: fare.discount, total: fare.total };
  };
}
