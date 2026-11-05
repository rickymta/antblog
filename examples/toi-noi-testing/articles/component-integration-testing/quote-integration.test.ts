/**
 * TEST-013 — Component integration testing: quote.ts ↔ pricing.ts ↔ voucher.ts (+ time.ts, kho voucher, kho chuyến).
 * Chạy: npx vitest run articles/component-integration-testing --reporter=verbose
 *
 * Phần 0: từng component đúng khi kiểm riêng. Phần 1: ca integration trên src/ thật (đều đạt).
 * Phần 2: CÙNG ca trên 5 bản ghép lỗi — it.fails = kỳ vọng ca thất bại, kèm failure quan sát được.
 */
import { describe, expect, it } from 'vitest';
import type { AppDeps } from '../../src/app/deps';
import { createQuote, priceQuote } from '../../src/app/quote';
import { createTrip } from '../../src/app/trips';
import { calculateFare } from '../../src/domain/fare';
import { isPeakTime, vnTime } from '../../src/domain/time';
import { normalizeVoucherCode, voucherDiscountMilli } from '../../src/domain/voucher';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { defectivePriceQuote, type DefectId, type PriceFn } from './buggy-quote';

const AT_0900 = vnTime(2026, 11, 4, 9, 0); // ngoài cao điểm
const AT_1130 = vnTime(2026, 11, 4, 11, 30); // cao điểm (R-FARE-04)
const C001 = { role: 'customer' as const, id: 'C001' };

function system(at: Date, raining: 'SG'[] = []) {
  return createMemorySystem({ clock: new FixedClock(at), rainingCities: raining });
}

const err = (code: string) => expect.objectContaining({ code });

/** Mỗi ca integration nhắm một interface và một defect giao tiếp. */
interface IntegrationCase {
  id: string;
  defect: DefectId;
  check: (price: PriceFn) => Promise<void>;
}

const cases: IntegrationCase[] = [
  {
    id: 'IF-1 · quote → pricing → voucher: oracle R-FARE-09 đi qua cả chuỗi ⇒ 52.000 đ, giảm 12.925 đ',
    defect: 'D1-don-vi-tien',
    async check(price) {
      const s = system(AT_1130, ['SG']);
      const q = await price(s.deps, 'C001', { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4, voucherCode: 'GIAM20' }, AT_1130);
      expect({ discount: q.discount, total: q.total }).toEqual({ discount: 12_925, total: 52_000 });
    },
  },
  {
    id: 'IF-2 · số tiền A đưa cho voucher chưa làm tròn: HN 6,1 km = 39.450 đ < 40.000 ⇒ VOUCHER_MIN_AMOUNT (R-VOUCHER-06)',
    defect: 'D2-lam-tron-giua-chung',
    async check(price) {
      const s = system(AT_0900);
      const run = price(s.deps, 'C001', { kind: 'ride', cityCode: 'HN', vehicle: 'bike', distanceKm: 6.1, voucherCode: 'HN10' }, AT_0900);
      await expect(run).rejects.toThrow(err('VOUCHER_MIN_AMOUNT'));
    },
  },
  {
    id: 'IF-3 · mã voucher từ API: "" = không voucher, " giam20 " ≡ GIAM20 (R-VOUCHER-02)',
    defect: 'D3-ma-chua-chuan-hoa',
    async check(price) {
      const s = system(AT_0900);
      const base = { kind: 'ride' as const, cityCode: 'SG', vehicle: 'bike' as const, distanceKm: 7.4 };
      const empty = await price(s.deps, 'C001', { ...base, voucherCode: '' }, AT_0900);
      const spaced = await price(s.deps, 'C001', { ...base, voucherCode: ' giam20 ' }, AT_0900);
      expect([empty.voucherCode, empty.total, spaced.voucherCode, spaced.total]).toEqual([null, 46_000, 'GIAM20', 37_000]);
    },
  },
  {
    id: 'IF-4 · quote → time: 11:30 giờ VN là cao điểm dù máy chủ chạy UTC (R-GEN-02, R-FARE-04)',
    defect: 'D4-gio-may-chu',
    async check(price) {
      const s = system(AT_1130);
      const q = await price(s.deps, 'C001', { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 }, AT_1130);
      expect({ peak: q.peak, total: q.total }).toEqual({ peak: true, total: 57_000 });
    },
  },
  {
    id: 'IF-5 · thứ tự gọi pricing → kho chuyến: vừa thiếu tiền tối thiểu vừa hết lượt ⇒ báo VOUCHER_MIN_AMOUNT (R-VOUCHER-08)',
    defect: 'D5-thu-tu-kiem-tra',
    async check(price) {
      const s = system(AT_0900);
      // Dùng hết lượt HN10 hôm nay bằng đường thật: báo giá 7 km (43.500 đ ≥ 40.000) rồi đặt chuyến.
      const first = await createQuote(s.deps, C001, { kind: 'ride', cityCode: 'HN', vehicle: 'bike', distanceKm: 7, voucherCode: 'HN10' });
      await createTrip(s.deps, C001, { quoteId: first.id });
      const run = price(s.deps, 'C001', { kind: 'ride', cityCode: 'HN', vehicle: 'bike', distanceKm: 6.1, voucherCode: 'HN10' }, AT_0900);
      await expect(run).rejects.toThrow(err('VOUCHER_MIN_AMOUNT'));
    },
  },
];

describe('TEST-013 · 0 · từng component đúng khi kiểm riêng (component testing)', () => {
  it('fare.ts: oracle R-FARE-09 ⇒ 52.000 đ', () => {
    const terms = { code: 'GIAM20', percent: 20, maxDiscount: 30_000, minAmount: 0 };
    expect(calculateFare({ vehicle: 'bike', distanceKm: 7.4, peak: true, raining: true, voucher: terms }).total).toBe(52_000);
  });
  it('voucher.ts: 20% của 64.625.000 milli-đồng = 12.925.000 milli-đồng', () => {
    expect(voucherDiscountMilli({ code: 'GIAM20', percent: 20, maxDiscount: 30_000, minAmount: 0 }, 64_625_000)).toBe(12_925_000);
  });
  it('voucher.ts: chuẩn hoá mã " giam20 " ⇒ GIAM20, "" ⇒ null', () => {
    expect([normalizeVoucherCode(' giam20 '), normalizeVoucherCode('')]).toEqual(['GIAM20', null]);
  });
  it('time.ts: 11:30 giờ VN là cao điểm', () => {
    expect(isPeakTime(AT_1130)).toBe(true);
  });
});

describe('TEST-013 · 1 · ca integration trên src/ thật', () => {
  for (const c of cases) {
    it(c.id, () => c.check(priceQuote));
  }
});

describe('TEST-013 · 2 · cùng ca trên bản ghép lỗi', () => {
  for (const c of cases) {
    it.fails(`${c.defect} ⇒ ${c.id.slice(0, 4)} đỏ`, () => c.check(defectivePriceQuote[c.defect]));
  }

  const ride = (cityCode: string, distanceKm: number, voucherCode?: string) =>
    ({ kind: 'ride' as const, cityCode, vehicle: 'bike' as const, distanceKm, voucherCode });
  const quoteWith = (id: DefectId, deps: AppDeps, input: ReturnType<typeof ride>, at: Date) =>
    defectivePriceQuote[id](deps, 'C001', input, at);

  it('failure D1: giảm 12,925 đ thay vì 12.925 đ ⇒ khách trả 65.000 đ', async () => {
    const q = await quoteWith('D1-don-vi-tien', system(AT_1130, ['SG']).deps, ride('SG', 7.4, 'GIAM20'), AT_1130);
    expect({ discount: q.discount, total: q.total }).toEqual({ discount: 12.925, total: 65_000 });
  });
  it('failure D2: voucher HN10 được nhận, khách trả 36.000 đ', async () => {
    const q = await quoteWith('D2-lam-tron-giua-chung', system(AT_0900).deps, ride('HN', 6.1, 'HN10'), AT_0900);
    expect({ voucher: q.voucherCode, total: q.total }).toEqual({ voucher: 'HN10', total: 36_000 });
  });
  it('failure D3: ô voucher để trống ⇒ VOUCHER_NOT_FOUND', async () => {
    await expect(quoteWith('D3-ma-chua-chuan-hoa', system(AT_0900).deps, ride('SG', 7.4, ''), AT_0900)).rejects.toThrow(err('VOUCHER_NOT_FOUND'));
  });
  it('failure D4: 11:30 giờ VN bị coi là ngoài cao điểm ⇒ 46.000 đ', async () => {
    const q = await quoteWith('D4-gio-may-chu', system(AT_1130).deps, ride('SG', 7.4), AT_1130);
    expect({ peak: q.peak, total: q.total }).toEqual({ peak: false, total: 46_000 });
  });
  it('failure D5: báo VOUCHER_USAGE_LIMIT — khách tưởng chỉ cần chờ ngày mai', async () => {
    const s = system(AT_0900);
    const first = await createQuote(s.deps, C001, ride('HN', 7, 'HN10'));
    await createTrip(s.deps, C001, { quoteId: first.id });
    await expect(quoteWith('D5-thu-tu-kiem-tra', s.deps, ride('HN', 6.1, 'HN10'), AT_0900)).rejects.toThrow(err('VOUCHER_USAGE_LIMIT'));
  });
});
