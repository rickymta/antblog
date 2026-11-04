import { DomainError } from './errors';
import { percentOf, roundToTenth, roundUpToThousandDong, toDong, toMilli } from './money';
import type { Vehicle } from './types';
import { voucherDiscountMilli, type VoucherTerms } from './voucher';

/** Bảng giá theo loại xe (SPEC R-FARE-01). */
export const FARE_TABLE = {
  bike: { openingFee: 12_000, perKm: 4_500, minimumFare: 15_000 },
  car: { openingFee: 25_000, perKm: 11_000, minimumFare: 30_000 },
} as const satisfies Record<Vehicle, { openingFee: number; perKm: number; minimumFare: number }>;

export const MIN_DISTANCE_KM = 0.5;
export const MAX_DISTANCE_KM = 100;
export const PEAK_PERCENT = 125; // ×1,25 (SPEC R-FARE-04)
export const RAIN_SURCHARGE = 8_000; // SPEC R-FARE-05

/** Làm tròn 0,1 km half-up rồi kiểm [0,5; 100] (SPEC R-FARE-02). */
export function normalizeDistanceKm(raw: number): number {
  const km = roundToTenth(raw);
  if (!Number.isFinite(km) || km < MIN_DISTANCE_KM || km > MAX_DISTANCE_KM) {
    throw new DomainError(
      'DISTANCE_OUT_OF_RANGE',
      `Quãng đường phải từ ${MIN_DISTANCE_KM} đến ${MAX_DISTANCE_KM} km (sau khi làm tròn 0,1 km).`,
    );
  }
  return km;
}

export interface FareInput {
  vehicle: Vehicle;
  distanceKm: number; // chưa làm tròn — hàm tự chuẩn hoá
  parcelSurcharge?: number; // đồng, cộng vào cước cơ bản (SPEC R-PARCEL-04)
  peak: boolean;
  raining: boolean;
  voucher?: VoucherTerms | null;
}

/** Từng bước của công thức, đơn vị đồng (SPEC R-FARE-10). Giá trị trung gian có thể lẻ. */
export interface FareBreakdown {
  vehicle: Vehicle;
  distanceKm: number;
  openingFee: number;
  distanceCharge: number;
  parcelSurcharge: number;
  base: number;
  afterPeak: number;
  rainSurcharge: number;
  beforeDiscount: number;
  discount: number;
  afterDiscount: number;
  minimumFare: number;
  total: number;
}

/** Công thức cước theo thứ tự cố định của SPEC R-FARE-03. */
export function calculateFare(input: FareInput): FareBreakdown {
  const distanceKm = normalizeDistanceKm(input.distanceKm);
  const prices = FARE_TABLE[input.vehicle];

  // 1. Cước cơ bản (+ phụ phí bưu kiện)
  const openingFee = toMilli(prices.openingFee);
  const distanceCharge = toMilli(prices.perKm * distanceKm);
  const parcelSurcharge = toMilli(input.parcelSurcharge ?? 0);
  const base = openingFee + distanceCharge + parcelSurcharge;
  // 2. Cao điểm
  const afterPeak = input.peak ? percentOf(base, PEAK_PERCENT) : base;
  // 3. Mưa
  const rainSurcharge = input.raining ? toMilli(RAIN_SURCHARGE) : 0;
  const beforeDiscount = afterPeak + rainSurcharge;
  // 4. Voucher
  const discount = input.voucher ? voucherDiscountMilli(input.voucher, beforeDiscount) : 0;
  const afterDiscount = beforeDiscount - discount;
  // 5. Cước tối thiểu
  const minimumFare = toMilli(prices.minimumFare);
  const atLeastMinimum = Math.max(afterDiscount, minimumFare);
  // 6. Làm tròn lên 1.000 đ
  const total = roundUpToThousandDong(atLeastMinimum);

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
