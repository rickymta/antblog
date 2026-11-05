/**
 * TEST-029 · Static Analysis — xác nhận bằng test động: cảnh báo tĩnh nào là defect THẬT.
 * Mỗi ca chạy trên mã thật (src/) phải đạt; ca chạy trên bản smelly dùng `it.fails` (PHẢI thất bại
 * vì bản smelly vi phạm SPEC). Chạy: npx vitest run articles/static-analysis-phan-tich-code-tinh
 *
 * Bản smelly được nạp bằng import động với đường dẫn là biến, để `npm run typecheck` của lab không
 * kéo tệp .mts (cố ý có lỗi kiểu) vào chương trình.
 */
import { describe, expect, it } from 'vitest';
import { DomainError } from '../../src/domain/errors';
import { toMilli } from '../../src/domain/money';
import { vnTime } from '../../src/domain/time';
import { assertUsageAvailable, assertVoucherEligible, voucherDiscountMilli, type Voucher } from '../../src/domain/voucher';
import type { CityCode, TripKind } from '../../src/domain/types';

/** Chép lại kiểu đầu vào của checkVoucher (không import tĩnh tệp .mts). */
interface VoucherCheckInput {
  voucher: Voucher | undefined; kind: TripKind; cityCode: CityCode; at: Date; amount: number; usedToday: number;
}

const smellyPath = './voucher-smelly.mts';
const { checkVoucher } = (await import(smellyPath)) as {
  checkVoucher: (input: VoucherCheckInput, verbose?: boolean) => number;
};

const v = (patch: Partial<Voucher> = {}): Voucher => ({
  code: 'GIAM20', percent: 20, maxDiscount: 15_000, minAmount: 30_000,
  kinds: [], cityCodes: [], validFrom: '2026-11-01', validTo: '2026-11-30', perCustomerPerDay: 1,
  ...patch,
});
const base = (patch: Partial<VoucherCheckInput> = {}): VoucherCheckInput => ({
  voucher: v(), kind: 'ride', cityCode: 'SG', at: vnTime(2026, 11, 4, 9, 0), amount: 52_000, usedToday: 0, ...patch,
});
const codeOf = (fn: () => unknown): string | null => {
  try { fn(); return null; } catch (e) { return e instanceof DomainError ? e.code : 'KHAC'; }
};

describe('C1 · dòng 33 no-unnecessary-condition — kinds rỗng nghĩa là mọi dịch vụ (R-VOUCHER-01)', () => {
  it('mã thật: kinds = [] ⇒ áp dụng được cho ride', () => {
    expect(codeOf(() => assertVoucherEligible(v(), { kind: 'ride', cityCode: 'SG', at: base().at }))).toBeNull();
  });
  it.fails('smelly: kinds = [] ⇒ phải áp dụng được (bản smelly ném VOUCHER_NOT_APPLICABLE)', () => {
    expect(codeOf(() => checkVoucher(base()))).toBeNull();
  });
});

describe('C2 · dòng 47 no-useless-assignment — trần maxDiscount (R-VOUCHER-06)', () => {
  it('mã thật: 20% × 100.000 đ = 20.000 đ, bị chặn ở trần 15.000 đ', () => {
    expect(voucherDiscountMilli(v(), toMilli(100_000))).toBe(toMilli(15_000));
  });
  it.fails('smelly: tiền giảm phải là 15.000 đ (bản smelly trả 20.000 đ)', () => {
    expect(checkVoucher(base({ amount: 100_000 }))).toBe(15_000);
  });
});

describe('C3 · dòng 28 no-unused-vars — ngày theo giờ Việt Nam (R-VOUCHER-04, R-GEN-02)', () => {
  const at = vnTime(2026, 12, 1, 6, 30); // 06:30 giờ VN ngày 01/12 = 23:30 UTC ngày 30/11
  it('mã thật: 01/12 giờ VN đã quá validTo 30/11 ⇒ VOUCHER_EXPIRED', () => {
    expect(codeOf(() => assertVoucherEligible(v(), { kind: 'ride', cityCode: 'SG', at }))).toBe('VOUCHER_EXPIRED');
  });
  it.fails('smelly: phải VOUCHER_EXPIRED (bản smelly so ngày UTC 30/11 nên vẫn cho dùng)', () => {
    expect(codeOf(() => checkVoucher(base({ at })))).toBe('VOUCHER_EXPIRED');
  });
});

describe('C4 · dòng 54 TS2454 — lượt dùng: usedToday = perCustomerPerDay là hết lượt (R-VOUCHER-07)', () => {
  it('mã thật: đã dùng 1/1 lượt ⇒ VOUCHER_USAGE_LIMIT', () => {
    expect(codeOf(() => assertUsageAvailable(v(), 1))).toBe('VOUCHER_USAGE_LIMIT');
  });
  it.fails('smelly: phải VOUCHER_USAGE_LIMIT (bản smelly dùng > nên cho qua)', () => {
    expect(codeOf(() => checkVoucher(base({ usedToday: 1 })))).toBe('VOUCHER_USAGE_LIMIT');
  });
});
