/**
 * TEST-041 · Random Testing — bộ sinh đầu vào ngẫu nhiên cho lõi định giá (src/domain/pricing.ts).
 *
 * Hai phân phối (CTAL-TA v4.0 mục 3.1.3):
 *  - `unguidedCase`: phân phối CỐ ĐỊNH, không theo thói quen dùng (usage-agnostic) — cho mục đích verification.
 *  - `profileCase`: phân phối theo operational profile — cho mục đích validation. Tỉ lệ dịch vụ và tỉ lệ giờ
 *    cao điểm lấy từ hồ sơ dự án; các tỉ lệ còn lại (mưa, quãng đường, loại xe, voucher) là GIẢ ĐỊNH.
 *
 * Quãng đường và khối lượng sinh dưới dạng số nguyên mét / gam rồi mới chia 1.000, để oracle
 * (oracle.ts) kiểm biên bằng số nguyên mà không đi qua hàm làm tròn của chính sản phẩm.
 */
import fc from 'fast-check';
import type { FoodItem } from '../../src/domain/food';
import type { PriceRequest } from '../../src/domain/pricing';
import type { Vehicle } from '../../src/domain/types';
import type { VoucherTerms } from '../../src/domain/voucher';

export interface QuoteCase {
  req: PriceRequest;
  meters: number; // quãng đường gốc, mét
  grams: number | null; // khối lượng gốc của bưu kiện, gam
  peak: boolean;
  raining: boolean;
  voucher: VoucherTerms | null;
}

const vehicleArb = fc.constantFrom<Vehicle>('bike', 'car');
const cityArb = fc.constantFrom('SG', 'HN', 'DN', 'HP', 'CT', 'NT', 'DL'); // DL không được phục vụ

const itemArb: fc.Arbitrary<FoodItem> = fc.record({
  name: fc.constant('Món thử'),
  unitPrice: fc.integer({ min: 1, max: 200 }).map((k) => k * 1_000),
  quantity: fc.integer({ min: 1, max: 5 }),
});

const randomVoucher: fc.Arbitrary<VoucherTerms | null> = fc.option(
  fc.record({
    code: fc.constant('RND'),
    percent: fc.integer({ min: 1, max: 50 }),
    maxDiscount: fc.integer({ min: 1, max: 100 }).map((k) => k * 1_000),
    minAmount: fc.integer({ min: 0, max: 200 }).map((k) => k * 1_000),
  }),
  { nil: null },
);

function buildCase(
  kind: 'ride' | 'parcel' | 'food',
  cityCode: string,
  vehicle: Vehicle,
  meters: number,
  grams: number,
  items: FoodItem[],
  peak: boolean,
  raining: boolean,
  voucher: VoucherTerms | null,
): QuoteCase {
  const distanceKm = meters / 1_000;
  const base = { meters, peak, raining, voucher };
  switch (kind) {
    case 'ride':
      return { ...base, grams: null, req: { kind, cityCode, vehicle, distanceKm } };
    case 'parcel':
      return { ...base, grams, req: { kind, cityCode, vehicle, distanceKm, parcelWeightKg: grams / 1_000 } };
    case 'food':
      return { ...base, grams: null, req: { kind, cityCode, distanceKm, items } };
  }
}

/** Phân phối cố định, quãng đường 0 – maxMeters mét. */
export const uniformCase = (maxMeters: number): fc.Arbitrary<QuoteCase> => fc
  .tuple(
    fc.constantFrom<'ride' | 'parcel' | 'food'>('ride', 'parcel', 'food'),
    cityArb,
    vehicleArb,
    fc.integer({ min: 0, max: maxMeters }),
    fc.integer({ min: 0, max: 35_000 }),
    fc.array(itemArb, { minLength: 1, maxLength: 5 }),
    fc.boolean(),
    fc.boolean(),
    randomVoucher,
  )
  .map((t) => buildCase(...t));

/** Phân phối cố định trên toàn miền đầu vào, kể cả vùng không hợp lệ (0–120 km, 0–35 kg). */
export const unguidedCase = uniformCase(120_000);

/**
 * Operational profile (validation). Hồ sơ dự án: 0,8 tr đặt xe + 0,9 tr đồ ăn + 0,3 tr bưu kiện mỗi ngày;
 * hai khung cao điểm chiếm 50% chuyến. GIẢ ĐỊNH: mưa 20%; quãng đường 70% từ 1–8 km, 25% từ 8–25 km,
 * 5% từ 25–100 km; đặt xe 75% xe máy; bưu kiện 80% ≤ 5 kg; 30% đơn dùng voucher mẫu của lab.
 */
const GIAM20: VoucherTerms = { code: 'GIAM20', percent: 20, maxDiscount: 30_000, minAmount: 0 };
const FOOD15: VoucherTerms = { code: 'FOOD15', percent: 15, maxDiscount: 25_000, minAmount: 50_000 };

const profileMeters = fc.oneof(
  { arbitrary: fc.integer({ min: 1_000, max: 8_000 }), weight: 70 },
  { arbitrary: fc.integer({ min: 8_001, max: 25_000 }), weight: 25 },
  { arbitrary: fc.integer({ min: 25_001, max: 100_000 }), weight: 5 },
);
const profileGrams = fc.oneof(
  { arbitrary: fc.integer({ min: 100, max: 5_000 }), weight: 80 },
  { arbitrary: fc.integer({ min: 5_001, max: 30_000 }), weight: 20 },
);
const weighted = <T>(yes: T, no: T, yesWeight: number, noWeight: number) =>
  fc.oneof({ arbitrary: fc.constant(yes), weight: yesWeight }, { arbitrary: fc.constant(no), weight: noWeight });

export const profileCase: fc.Arbitrary<QuoteCase> = fc
  .tuple(
    fc.oneof(
      { arbitrary: fc.constant<'ride'>('ride'), weight: 8 },
      { arbitrary: fc.constant<'food'>('food'), weight: 9 },
      { arbitrary: fc.constant<'parcel'>('parcel'), weight: 3 },
    ),
    fc.constantFrom('SG', 'HN', 'DN', 'HP', 'CT', 'NT'),
    weighted<Vehicle>('bike', 'car', 3, 1),
    profileMeters,
    profileGrams,
    fc.array(itemArb, { minLength: 1, maxLength: 4 }),
    weighted(true, false, 1, 1),
    weighted(true, false, 1, 4),
    weighted(true, false, 3, 7),
  )
  .map(([kind, city, vehicle, meters, grams, items, peak, raining, useVoucher]) => {
    // Bưu kiện > 20 kg thì người dùng thật chọn ô tô (app không cho chọn xe máy).
    const v: Vehicle = kind === 'parcel' && grams > 20_049 ? 'car' : vehicle;
    const voucher = useVoucher ? (kind === 'food' ? FOOD15 : GIAM20) : null;
    return buildCase(kind, city, v, meters, grams, items, peak, raining, voucher);
  });
