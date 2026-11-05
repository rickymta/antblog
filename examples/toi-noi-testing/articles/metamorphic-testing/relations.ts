/**
 * TEST-042 · Metamorphic relation (MR) của thuật toán cước (R-FARE-03…06) + bộ sinh source test case.
 *
 * Mỗi MR = cách biến đổi một source test case thành follow-up test case + quan hệ giữa hai kết quả.
 * Không MR nào cần biết tổng tiền «đúng» của từng ca — đó là lý do MT chạy được khi thiếu oracle chính xác.
 */
import fc from 'fast-check';
import { DomainError } from '../../src/domain/errors';
import type { PriceRequest, PriceResult } from '../../src/domain/pricing';
import type { Conditions, PriceFn } from './variants';

/** Source test case: chỉ sinh trong miền HỢP LỆ (quãng đường 0,45–95 km để còn chỗ tăng). */
export interface Source {
  req: PriceRequest;
  meters: number;
  c: Conditions;
}

const voucherArb = fc.option(
  fc.record({
    code: fc.constant('MT'),
    percent: fc.integer({ min: 1, max: 50 }),
    maxDiscount: fc.integer({ min: 1, max: 100 }).map((k) => k * 1_000),
    minAmount: fc.integer({ min: 0, max: 50 }).map((k) => k * 1_000),
  }),
  { nil: null },
);

export const sourceArb: fc.Arbitrary<Source> = fc
  .record({
    kind: fc.constantFrom<'ride' | 'parcel' | 'food'>('ride', 'parcel', 'food'),
    city: fc.constantFrom('SG', 'HN', 'DN', 'HP', 'CT', 'NT'),
    vehicle: fc.constantFrom<'bike' | 'car'>('bike', 'car'),
    meters: fc.integer({ min: 450, max: 95_000 }),
    grams: fc.integer({ min: 50, max: 30_049 }),
    items: fc.array(
      fc.record({ name: fc.constant('Món'), unitPrice: fc.integer({ min: 30, max: 200 }).map((k) => k * 1_000), quantity: fc.integer({ min: 1, max: 3 }) }),
      { minLength: 1, maxLength: 3 },
    ),
    peak: fc.boolean(),
    raining: fc.boolean(),
    voucher: voucherArb,
  })
  .map(({ kind, city, vehicle, meters, grams, items, peak, raining, voucher }): Source => {
    const distanceKm = meters / 1_000;
    const c = { peak, raining, voucher };
    if (kind === 'ride') return { meters, c, req: { kind, cityCode: city, vehicle, distanceKm } };
    if (kind === 'parcel') {
      const v = grams > 20_049 ? 'car' : vehicle; // R-PARCEL-02: > 20 kg phải ô tô
      return { meters, c, req: { kind, cityCode: city, vehicle: v, distanceKm, parcelWeightKg: grams / 1_000 } };
    }
    return { meters, c, req: { kind, cityCode: city, distanceKm, items } };
  });

/** Chạy test item; lỗi nghiệp vụ (vd VOUCHER_MIN_AMOUNT) ⇒ null = cặp không dùng được. */
export function tryPrice(price: PriceFn, req: PriceRequest, c: Conditions): PriceResult | null {
  try {
    return price(req, c);
  } catch (e) {
    if (e instanceof DomainError) return null;
    throw e;
  }
}

export interface Pair {
  source: PriceResult;
  followUp: PriceResult;
}

export interface MR<T> {
  id: string;
  text: string;
  arb: fc.Arbitrary<T>;
  /** Trả cặp kết quả hoặc null nếu một trong hai ca không chạy được (bỏ qua cặp). */
  run(price: PriceFn, t: T): Pair | null;
  holds(p: Pair, t: T): boolean;
}

const pair = (price: PriceFn, a: [PriceRequest, Conditions], b: [PriceRequest, Conditions]): Pair | null => {
  const source = tryPrice(price, ...a);
  const followUp = tryPrice(price, ...b);
  return source && followUp ? { source, followUp } : null;
};

/**
 * MR1 (R-FARE-01…03): tăng quãng đường, giữ nguyên mọi thứ khác ⇒ tổng không giảm.
 * Follow-up tăng ĐÚNG một bước 0,1 km (bước nhỏ nhất còn ý nghĩa sau làm tròn R-FARE-02).
 */
export const MR1: MR<[Source, number]> = {
  id: 'MR1',
  text: 'quãng đường tăng ⇒ tổng không giảm',
  arb: fc.tuple(sourceArb, fc.constant(100)),
  run: (price, [s, delta]) => pair(price, [s.req, s.c], [{ ...s.req, distanceKm: (s.meters + delta) / 1_000 }, s.c]),
  holds: (p) => p.followUp.total >= p.source.total,
};

/** MR2 (R-FARE-05): bật mưa ⇒ số tiền TRƯỚC voucher tăng đúng 8.000 đ (kể cả giờ cao điểm). */
export const MR2: MR<Source> = {
  id: 'MR2',
  text: 'bật mưa ⇒ «trước giảm» tăng đúng 8.000 đ',
  arb: sourceArb,
  run: (price, s) => pair(price, [s.req, { ...s.c, raining: false }], [s.req, { ...s.c, raining: true }]),
  holds: (p) => p.followUp.fare.beforeDiscount - p.source.fare.beforeDiscount === 8_000,
};

/** MR3 (R-FARE-04): cùng yêu cầu ở giờ cao điểm ⇒ tổng không rẻ hơn giờ thường. */
export const MR3: MR<Source> = {
  id: 'MR3',
  text: 'giờ cao điểm ⇒ tổng không rẻ hơn',
  arb: sourceArb,
  run: (price, s) => pair(price, [s.req, { ...s.c, peak: false }], [s.req, { ...s.c, peak: true }]),
  holds: (p) => p.followUp.total >= p.source.total,
};

/**
 * MR4 (R-FARE-06) — lời giải Bài tập 1: không mưa, voucher không chạm trần, không mức tối thiểu;
 * chuyển sang cao điểm ⇒ tiền giảm nhân đúng 1,25 (vì voucher tính trên số tiền SAU cao điểm).
 */
export const MR4: MR<Source> = {
  id: 'MR4',
  text: 'cao điểm, voucher không trần ⇒ tiền giảm × 1,25',
  arb: fc
    .tuple(sourceArb, fc.integer({ min: 1, max: 50 }))
    .filter(([s]) => s.req.kind !== 'food')
    .map(([s, percent]) => ({ ...s, c: { peak: false, raining: false, voucher: { code: 'MT', percent, maxDiscount: 10_000_000, minAmount: 0 } } })),
  run: (price, s) => pair(price, [s.req, s.c], [s.req, { ...s.c, peak: true }]),
  holds: (p) => Math.abs(p.followUp.fare.discount - 1.25 * p.source.fare.discount) < 1e-6,
};

/** MR1 với bước tăng lớn 0,1–5 km — cùng quan hệ, follow-up «xa» source hơn (so sánh ở MT-06). */
export const MR1_BUOC_LON: MR<[Source, number]> = { ...MR1, id: 'MR1-lớn', arb: fc.tuple(sourceArb, fc.integer({ min: 100, max: 5_000 })) };

/** MR SAI: «bật mưa ⇒ TỔNG tăng đúng 8.000 đ» — bỏ quên voucher, cước tối thiểu. */
export const MR_SAI: MR<Source> = { ...MR2, id: 'MR-sai', text: 'bật mưa ⇒ TỔNG tăng đúng 8.000 đ', holds: (p) => p.followUp.total - p.source.total === 8_000 };

/** MR YẾU: «bật mưa ⇒ tổng không giảm» — đúng nhưng không phân biệt được 8.000 với 10.000 đ. */
export const MR_YEU: MR<Source> = { ...MR2, id: 'MR-yếu', text: 'bật mưa ⇒ tổng không giảm', holds: (p) => p.followUp.total >= p.source.total };

/** Ghép MR thành property: source + follow-up = một test procedure, đánh giá chung. */
export function mrProperty<T>(price: PriceFn, mr: MR<T>) {
  return fc.property(mr.arb, (t) => {
    const p = mr.run(price, t);
    fc.pre(p !== null);
    return mr.holds(p!, t);
  });
}
