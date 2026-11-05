/**
 * Bộ test dùng chung cho ba bản build của bài TEST-019 «Confirmation Testing và Regression Testing».
 *
 * - Confirmation (CF-xx): chạy lại các test từng thất bại vì defect làm tròn (CF-01…04)
 *   và test MỚI thêm để phủ phần code mà bản sửa đụng tới (CF-05, CF-06).
 * - Regression (RG-xx): các test cũ được CHỌN theo impact analysis — mọi đường đi qua
 *   `roundToTenth`: quãng đường (R-FARE-02), khối lượng (R-PARCEL-01), phí giao đồ ăn (R-FOOD-04),
 *   ở cấp component (gọi hàm domain) và cấp hệ thống (gọi API qua supertest).
 *
 * Mỗi tệp *.test.ts thay `roundToTenth` bằng phiên bản của bản build tương ứng (vi.mock) rồi gọi
 * defineSuite. Ca thất bại đã biết được đánh dấu `it.fails` để `npm test` của lab vẫn xanh;
 * đặt SHOW_RED=1 để thấy màu đỏ thật.
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { calculateFare, normalizeDistanceKm } from '../../src/domain/fare';
import { priceFood } from '../../src/domain/food';
import { normalizeWeightKg, parcelSurcharge } from '../../src/domain/parcel';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';

const showRed = process.env.SHOW_RED === '1';

/** Kết quả hoặc mã lỗi nghiệp vụ — để so sánh được cả hai loại kết quả mong đợi. */
function outcome(fn: () => unknown): unknown {
  try {
    return fn();
  } catch (error) {
    return { error: (error as { code?: string }).code ?? String(error) };
  }
}

/** Gọi POST /api/quotes lúc 09:00 giờ VN 04/11/2026 (ngoài cao điểm), không mưa. */
async function quote(body: object) {
  const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
  const app = createApp(system.deps, { logErrors: false });
  const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send(body);
  return { status: res.status, quote: res.body.quote };
}

const bike = (distanceKm: number) => ({ vehicle: 'bike' as const, distanceKm, peak: false, raining: false });

export function defineSuite(build: string, knownFailures: ReadonlySet<string>): void {
  const t = (id: string) => (knownFailures.has(id) && !showRed ? it.fails : it);

  describe(`${build} · confirmation testing`, () => {
    t('CF-01')('CF-01 (chạy lại) R-FARE-02: 7,35 km ⇒ 7,4 km', () => {
      expect(outcome(() => normalizeDistanceKm(7.35))).toBe(7.4);
    });
    t('CF-02')('CF-02 (chạy lại) R-FARE-01/08: xe máy 7,35 km, 09:00, không mưa ⇒ 46.000 đ', () => {
      expect(calculateFare(bike(7.35)).total).toBe(46_000);
    });
    t('CF-03')('CF-03 (chạy lại) R-FARE-02: 100,05 km ⇒ DISTANCE_OUT_OF_RANGE', () => {
      expect(outcome(() => normalizeDistanceKm(100.05))).toEqual({ error: 'DISTANCE_OUT_OF_RANGE' });
    });
    t('CF-04')('CF-04 (chạy lại, cấp hệ thống) POST /api/quotes xe máy 7,35 km ⇒ total 46.000', async () => {
      const r = await quote({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.35 });
      expect([r.status, r.quote?.distanceKm, r.quote?.total]).toEqual([200, 7.4, 46_000]);
    });
    t('CF-05')('CF-05 (mới) R-PARCEL-01/03: 5,05 kg ⇒ 5,1 kg, phụ phí 2.000 đ', () => {
      const kg = normalizeWeightKg(5.05);
      expect([kg, parcelSurcharge(kg)]).toEqual([5.1, 2_000]);
    });
    t('CF-06')('CF-06 (mới) R-FARE-02: quét 996 giá trị 0,45…99,95 dạng x,x5 ⇒ luôn lên', () => {
      const mismatches: string[] = [];
      for (let tenths = 5; tenths <= 1000; tenths++) {
        const raw = Number(`${Math.floor((tenths - 1) / 10)}.${(tenths - 1) % 10}5`); // 0.45, 0.55 … 99.95
        const got = outcome(() => normalizeDistanceKm(raw));
        if (got !== tenths / 10) mismatches.push(`${raw}⇒${JSON.stringify(got)}`);
      }
      expect(mismatches.slice(0, 3), `${mismatches.length} giá trị lệch`).toEqual([]);
    });
  });

  describe(`${build} · regression testing (chọn theo impact analysis)`, () => {
    t('RG-01')('RG-01 R-FARE-02: 0,44 km ⇒ DISTANCE_OUT_OF_RANGE', () => {
      expect(outcome(() => normalizeDistanceKm(0.44))).toEqual({ error: 'DISTANCE_OUT_OF_RANGE' });
    });
    t('RG-02')('RG-02 R-FARE-02: 0,45 km ⇒ 0,5 km', () => {
      expect(outcome(() => normalizeDistanceKm(0.45))).toBe(0.5);
    });
    t('RG-03')('RG-03 R-FARE-02: 100,04 km ⇒ 100,0 km', () => {
      expect(outcome(() => normalizeDistanceKm(100.04))).toBe(100);
    });
    t('RG-04')('RG-04 R-FARE-09: oracle 7,4 km, cao điểm, mưa, GIAM20 ⇒ 52.000 đ', () => {
      const voucher = { code: 'GIAM20', percent: 20, maxDiscount: 30_000, minAmount: 0 };
      expect(calculateFare({ vehicle: 'bike', distanceKm: 7.4, peak: true, raining: true, voucher }).total).toBe(52_000);
    });
    t('RG-05')('RG-05 R-PARCEL-01: 0,04 kg ⇒ WEIGHT_OUT_OF_RANGE', () => {
      expect(outcome(() => normalizeWeightKg(0.04))).toEqual({ error: 'WEIGHT_OUT_OF_RANGE' });
    });
    t('RG-06')('RG-06 R-PARCEL-01: 30,04 kg ⇒ 30,0 kg', () => {
      expect(outcome(() => normalizeWeightKg(30.04))).toBe(30);
    });
    t('RG-07')('RG-07 R-PARCEL-03: 6,0 kg ⇒ 2.000 đ; 6,1 kg ⇒ 4.000 đ', () => {
      expect([6.0, 6.1].map((w) => parcelSurcharge(normalizeWeightKg(w)))).toEqual([2_000, 4_000]);
    });
    t('RG-08')('RG-08 R-FOOD-04: đồ ăn 2,01 km, món 50.000 đ ⇒ phí giao 21.000, tổng 71.000', () => {
      const food = priceFood({ distanceKm: 2.01, items: [{ name: 'Cơm tấm', unitPrice: 50_000, quantity: 1 }], peak: false, raining: false });
      expect([food.deliveryFee.total, food.total]).toEqual([21_000, 71_000]);
    });
    t('RG-09')('RG-09 (cấp hệ thống) POST /api/quotes đồ ăn 2,01 km ⇒ deliveryFee 21.000', async () => {
      const r = await quote({ kind: 'food', cityCode: 'SG', distanceKm: 2.01, items: [{ name: 'Cơm tấm', unitPrice: 50_000, quantity: 1 }] });
      expect([r.status, r.quote?.deliveryFee, r.quote?.total]).toEqual([200, 21_000, 71_000]);
    });
  });
}
