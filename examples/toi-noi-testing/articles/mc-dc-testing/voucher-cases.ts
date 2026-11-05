/**
 * TEST-047 · Dữ liệu và bộ chạy cho quyết định «voucher được áp» (SPEC R-VOUCHER-03…07).
 *
 *   V = C1 ∧ C2 ∧ C3 ∧ (C4 ∨ C5) ∧ (C6 ∨ C7) ∧ C8 ∧ C9
 *
 * C1 tìm thấy mã · C2 ngày ≥ validFrom · C3 ngày ≤ validTo · C4 kinds rỗng · C5 kinds chứa dịch vụ
 * C6 cityCodes rỗng · C7 cityCodes chứa thành phố · C8 A ≥ minAmount · C9 lượt đã dùng < perCustomerPerDay
 */
import type { AppDeps } from '../../src/app/deps';
import { createQuote, priceQuote, type QuoteDetails } from '../../src/app/quote';
import { createTrip } from '../../src/app/trips';
import { vnDate, vnTime } from '../../src/domain/time';
import type { Voucher } from '../../src/domain/voucher';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { labSeed, voucher } from '../../src/infra/seed';
import type { CondValue } from './mcdc';

/** 09:00 ngày 04/11/2026 giờ VN: ngoài cao điểm; không thành phố nào mưa. */
export const AT = vnTime(2026, 11, 4, 9, 0);
export const DAY = vnDate(AT); // '2026-11-04'
/** Xe máy SG 7,4 km: 12.000 + 4.500 × 7,4 = 45.300 đ = số tiền A áp voucher (R-FARE-03 bước 3). */
export const RIDE = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 } as const;
export const A_DONG = 45_300;
export const C001 = { role: 'customer' as const, id: 'C001' };

/** Ca gốc: mọi điều kiện đều ĐÚNG, chọn ở biên (minAmount = A, validFrom = validTo = ngày đặt). */
export const BASE: Voucher = voucher({
  code: 'MCDC10',
  percent: 10,
  maxDiscount: 20_000,
  minAmount: A_DONG,
  kinds: ['ride', 'parcel'],
  cityCodes: ['SG', 'HN'],
  validFrom: DAY,
  validTo: DAY,
  perCustomerPerDay: 1,
});

export type Expected = { total: number; discount: number } | { error: string };

export interface VoucherCase {
  id: string;
  /** Ghi chú điều kiện bị lật so với ca gốc. */
  note: string;
  overrides: Partial<Voucher>;
  /** Mã khách gõ; mặc định là mã của voucher. */
  typedCode?: string;
  /** Số chuyến chưa huỷ đã dùng mã hôm nay (tạo thật bằng createTrip trước khi báo giá). */
  usedBefore?: number;
  expected: Expected;
}

const OK = { total: 41_000, discount: 4_530 }; // 45.300 − 4.530 = 40.770 ⇒ làm tròn lên 41.000 (R-FARE-08)

/** 10 ca = N + 1 với N = 9 điều kiện. */
export const MCDC_CASES: VoucherCase[] = [
  { id: 'M1', note: 'ca gốc — mọi điều kiện đúng', overrides: {}, expected: OK },
  { id: 'M2', note: 'C1 sai: không có mã', overrides: {}, typedCode: 'KHONGCO', expected: { error: 'VOUCHER_NOT_FOUND' } },
  { id: 'M3', note: 'C2 sai: chưa tới validFrom', overrides: { validFrom: '2026-11-05', validTo: '2026-11-30' }, expected: { error: 'VOUCHER_EXPIRED' } },
  { id: 'M4', note: 'C3 sai: đã qua validTo', overrides: { validFrom: '2026-11-01', validTo: '2026-11-03' }, expected: { error: 'VOUCHER_EXPIRED' } },
  { id: 'M5', note: 'C4 sai, C5 sai: kinds = [food]', overrides: { kinds: ['food'] }, expected: { error: 'VOUCHER_NOT_APPLICABLE' } },
  { id: 'M6', note: 'C4 đúng: kinds rỗng (mọi dịch vụ)', overrides: { kinds: [] }, expected: OK },
  { id: 'M7', note: 'C6 sai, C7 sai: cityCodes = [HN]', overrides: { cityCodes: ['HN'] }, expected: { error: 'VOUCHER_NOT_APPLICABLE' } },
  { id: 'M8', note: 'C6 đúng: cityCodes rỗng (mọi thành phố)', overrides: { cityCodes: [] }, expected: OK },
  { id: 'M9', note: 'C8 sai: A = 45.300 < minAmount 45.301', overrides: { minAmount: A_DONG + 1 }, expected: { error: 'VOUCHER_MIN_AMOUNT' } },
  { id: 'M10', note: 'C9 sai: đã dùng 1/1 lượt hôm nay', overrides: {}, usedBefore: 1, expected: { error: 'VOUCHER_USAGE_LIMIT' } },
];

export function voucherOf(c: VoucherCase): Voucher {
  return { ...BASE, ...c.overrides };
}

/** Giá trị 9 điều kiện tính THẲNG từ dữ liệu ca theo SPEC (không đọc code) — kiểm thiết kế của bộ test. */
export function conditionsOf(c: VoucherCase): CondValue[] {
  const v = voucherOf(c);
  const typed = c.typedCode ?? v.code;
  if (typed !== v.code) return [false, null, null, null, null, null, null, null, null];
  return [
    true,
    DAY >= v.validFrom,
    DAY <= v.validTo,
    v.kinds.length === 0,
    v.kinds.includes(RIDE.kind),
    v.cityCodes.length === 0,
    v.cityCodes.includes(RIDE.cityCode),
    A_DONG >= v.minAmount,
    (c.usedBefore ?? 0) < v.perCustomerPerDay,
  ];
}

/** Quyết định V theo SPEC, có ngắt mạch như code (điều kiện không đánh giá = null). */
export function specDecision(cs: readonly CondValue[]): boolean {
  const [c1, c2, c3, c4, c5, c6, c7, c8, c9] = cs;
  return c1 === true && c2 === true && c3 === true && (c4 === true || c5 === true) && (c6 === true || c7 === true) && c8 === true && c9 === true;
}

/** Dựng hệ thống chỉ có voucher của ca, tiêu trước `usedBefore` lượt bằng chuyến thật. */
export async function systemFor(c: VoucherCase): Promise<AppDeps> {
  const seed = { ...labSeed(), vouchers: [voucherOf(c)] };
  const { deps } = createMemorySystem({ clock: new FixedClock(AT), seed });
  for (let i = 0; i < (c.usedBefore ?? 0); i++) {
    const quote = await createQuote(deps, C001, { ...RIDE, voucherCode: voucherOf(c).code });
    await createTrip(deps, C001, { quoteId: quote.id });
  }
  return deps;
}

/** Chạy báo giá thật qua src/app/quote.ts và trả kết quả dạng so sánh được. */
export async function runCase(c: VoucherCase): Promise<Expected> {
  const deps = await systemFor(c);
  try {
    const q: QuoteDetails = await priceQuote(deps, C001.id, { ...RIDE, voucherCode: c.typedCode ?? voucherOf(c).code }, AT);
    return { total: q.total, discount: q.discount };
  } catch (e) {
    return { error: (e as { code?: string }).code ?? String(e) };
  }
}
