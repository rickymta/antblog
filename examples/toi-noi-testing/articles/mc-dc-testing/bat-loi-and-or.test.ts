/**
 * TEST-047 · Decision coverage 100% vẫn để lọt lỗi `||` → `&&`; MC/DC bắt được.
 * Chạy riêng phần đo coverage của bản lỗi: xem README (lệnh có --coverage.include=…buggy-eligible.ts).
 */
import { describe, expect, it } from 'vitest';
import { assertVoucherEligible, type Voucher } from '../../src/domain/voucher';
import { assertVoucherEligibleBuggy } from './buggy-eligible';
import { AT, BASE, RIDE } from './voucher-cases';

const ctx = { kind: RIDE.kind, cityCode: RIDE.cityCode, at: AT } as const;
type Check = (v: Voucher, c: typeof ctx) => void;

/** Kết quả quan sát được: 'OK' hoặc mã lỗi. */
function outcome(check: Check, v: Voucher): string {
  try {
    check(v, ctx);
    return 'OK';
  } catch (e) {
    return (e as { code: string }).code;
  }
}

/** Bộ «decision coverage»: quyết định phạm vi chỉ nhận một ca đúng hết và một ca sai CẢ HAI; thêm một ca hết hạn. */
const DC_SET: [string, Voucher, string][] = [
  ['D1 đúng dịch vụ, đúng thành phố', BASE, 'OK'],
  ['D2 sai dịch vụ VÀ sai thành phố', { ...BASE, kinds: ['food'], cityCodes: ['HN'] }, 'VOUCHER_NOT_APPLICABLE'],
  ['D3 hết hạn (để phủ nốt câu lệnh throw đầu tiên)', { ...BASE, validFrom: '2026-11-01', validTo: '2026-11-03' }, 'VOUCHER_EXPIRED'],
];
/** Hai ca MC/DC thêm vào (M5, M7 của bộ chính): mỗi lần chỉ lật MỘT điều kiện. */
const MCDC_EXTRA: [string, Voucher, string][] = [
  ['M5 chỉ sai dịch vụ', { ...BASE, kinds: ['food'] }, 'VOUCHER_NOT_APPLICABLE'],
  ['M7 chỉ sai thành phố', { ...BASE, cityCodes: ['HN'] }, 'VOUCHER_NOT_APPLICABLE'],
];

describe('bộ decision coverage (3 ca) KHÔNG phân biệt được bản đúng và bản lỗi', () => {
  for (const [name, v, expected] of DC_SET) {
    it(`${name}: bản đúng = bản lỗi = ${expected}`, () => {
      expect([outcome(assertVoucherEligible, v), outcome(assertVoucherEligibleBuggy, v)]).toEqual([expected, expected]);
    });
  }
});

describe('ca MC/DC phát hiện lỗi: bản lỗi cho voucher lọt phạm vi', () => {
  for (const [name, v, expected] of MCDC_EXTRA) {
    it(`${name}: bản đúng ${expected}, bản lỗi OK ⇒ failure`, () => {
      expect([outcome(assertVoucherEligible, v), outcome(assertVoucherEligibleBuggy, v)]).toEqual([expected, 'OK']);
    });
  }
});
