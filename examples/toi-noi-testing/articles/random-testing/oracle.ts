/**
 * TEST-041 · Oracle tự động cho đầu vào ngẫu nhiên.
 *
 * Không tính lại cước (sẽ thành chép lại code). Thay vào đó kiểm các TÍNH CHẤT mà SPEC bắt buộc
 * với mọi đầu vào — một partial oracle:
 *   O1 không NaN/Infinity, tổng là số nguyên đồng (R-GEN-01)
 *   O2 tổng cước là bội số 1.000 đ (R-FARE-08)
 *   O3 tổng cước ≥ cước tối thiểu của loại xe (R-FARE-07)
 *   O4 tổng cước − max(sau giảm, tối thiểu) nằm trong [0; 1.000) — làm tròn LÊN, không quá một bậc (R-FARE-08)
 *   O5 0 ≤ tiền giảm ≤ trần voucher (R-VOUCHER-06)
 *   O6 đơn đồ ăn: tổng = tiền món − giảm + phí giao (R-FOOD-01)
 * và một oracle độc lập cho phía lỗi: mã lỗi đầu tiên dự kiến theo R-VALID-01, tính bằng số nguyên mét/gam.
 */
import { DomainError } from '../../src/domain/errors';
import { FARE_TABLE, type FareBreakdown } from '../../src/domain/fare';
import { itemsSubtotal } from '../../src/domain/food';
import { normalizePriceRequest, priceRequest, type PriceRequest, type PriceResult } from '../../src/domain/pricing';
import { CITY_CODES } from '../../src/domain/types';
import type { VoucherTerms } from '../../src/domain/voucher';
import type { QuoteCase } from './generators';

/** Hệ thống được kiểm: nhận yêu cầu thô + điều kiện, trả giá hoặc ném DomainError. */
export type PriceFn = (
  req: PriceRequest,
  conditions: { peak: boolean; raining: boolean; voucher: VoucherTerms | null },
) => PriceResult;

function fareViolations(f: FareBreakdown, voucher: VoucherTerms | null, label: string): string[] {
  const v: string[] = [];
  const min = FARE_TABLE[f.vehicle].minimumFare;
  const numbers = Object.values(f).filter((x): x is number => typeof x === 'number');
  if (!numbers.every(Number.isFinite)) v.push(`O1 ${label}: có NaN/Infinity`);
  if (!Number.isInteger(f.total)) v.push(`O1 ${label}: tổng ${f.total} không nguyên`);
  if (f.total % 1_000 !== 0) v.push(`O2 ${label}: tổng ${f.total} không chia hết 1.000`);
  if (f.total < min) v.push(`O3 ${label}: tổng ${f.total} < tối thiểu ${min}`);
  const gap = f.total - Math.max(f.afterDiscount, min);
  if (!(gap >= 0 && gap < 1_000)) v.push(`O4 ${label}: tổng − max(sau giảm, tối thiểu) = ${gap}`);
  if (f.discount < 0 || (voucher && f.discount > voucher.maxDiscount) || (!voucher && f.discount !== 0)) {
    v.push(`O5 ${label}: tiền giảm ${f.discount} sai phạm vi`);
  }
  return v;
}

/** Lõi định giá thật của lab. */
export const srcPrice: PriceFn = (req, c) => priceRequest(normalizePriceRequest(req), c);

/** Vi phạm của một kết quả thành công (rỗng = đạt). */
export function resultViolations(c: QuoteCase, r: PriceResult): string[] {
  if (c.req.kind !== 'food') return fareViolations(r.fare, c.voucher, 'cước');
  const v = fareViolations(r.fare, null, 'phí giao'); // voucher không trừ vào phí giao (R-FOOD-04)
  const subtotal = itemsSubtotal(c.req.items);
  if (r.total !== subtotal - (r.itemsDiscount ?? NaN) + (r.deliveryFee ?? NaN)) v.push('O6: tổng đơn đồ ăn sai công thức');
  if (!Number.isFinite(r.total)) v.push('O1: tổng đơn đồ ăn NaN');
  return v;
}

/** Mã lỗi đầu tiên mà SPEC R-VALID-01 dự kiến cho phần đầu vào (null = đầu vào hợp lệ). */
export function expectedInputError(c: QuoteCase): string | null {
  if (!(CITY_CODES as readonly string[]).includes(c.req.cityCode)) return 'CITY_NOT_SUPPORTED';
  if (c.meters < 450 || c.meters >= 100_050) return 'DISTANCE_OUT_OF_RANGE'; // làm tròn 0,1 km half-up
  if (c.req.kind === 'parcel' && c.grams !== null) {
    if (c.grams < 50 || c.grams >= 30_050) return 'WEIGHT_OUT_OF_RANGE';
    if (c.grams >= 20_050 && c.req.vehicle === 'bike') return 'VEHICLE_TOO_SMALL';
  }
  if (c.req.kind === 'food' && itemsSubtotal(c.req.items) < 30_000) return 'SUBTOTAL_TOO_LOW';
  return null;
}

export type Outcome = { kind: 'ok'; result: PriceResult } | { kind: 'error'; code: string };

export function run(price: PriceFn, c: QuoteCase): Outcome {
  try {
    return { kind: 'ok', result: price(c.req, { peak: c.peak, raining: c.raining, voucher: c.voucher }) };
  } catch (e) {
    if (e instanceof DomainError) return { kind: 'error', code: e.code };
    throw e; // TypeError, RangeError… = failure: hệ thống sập thay vì báo lỗi nghiệp vụ
  }
}

/** Oracle đầy đủ cho một ca: vi phạm (rỗng = đạt). */
export function judge(price: PriceFn, c: QuoteCase): { outcome: Outcome; violations: string[] } {
  const outcome = run(price, c);
  const expected = expectedInputError(c);
  if (outcome.kind === 'error') {
    const ok = outcome.code === expected || (expected === null && outcome.code === 'VOUCHER_MIN_AMOUNT' && c.voucher !== null);
    return { outcome, violations: ok ? [] : [`mã lỗi ${outcome.code}, dự kiến ${expected ?? 'thành công'}`] };
  }
  if (expected !== null) return { outcome, violations: [`thành công, dự kiến ${expected}`] };
  return { outcome, violations: resultViolations(c, outcome.result) };
}
