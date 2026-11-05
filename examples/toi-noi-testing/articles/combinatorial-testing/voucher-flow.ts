/**
 * Luồng kiểm voucher của báo giá (bản sao rút gọn của priceQuote trong src/app/quote.ts) và
 * bốn bản build CÓ LỖI GÀI SẴN để đo sức bắt lỗi của từng bộ test. Không sửa src/.
 *
 *   dung        — đúng như src/ (test đối chiếu khẳng định trùng kết quả với priceQuote thật)
 *   V1-trim     — bậc 1: quên bỏ khoảng trắng của mã (R-VOUCHER-02)
 *   V2-ngay-utc — bậc 2: so ngày hiệu lực theo ngày UTC thay vì ngày Việt Nam (R-GEN-02, R-VOUCHER-04)
 *   V3-pham-vi  — bậc 2: chép nhầm điều kiện "rỗng" của kinds sang cityCodes (R-VOUCHER-05)
 *   V4-luot-utc — bậc 3: đếm lượt dùng trong ngày UTC thay vì ngày Việt Nam (R-VOUCHER-07)
 */
import { DomainError } from '../../src/domain/errors';
import { normalizePriceRequest, priceRequest } from '../../src/domain/pricing';
import { isPeakTime, vnDate, vnDayRange } from '../../src/domain/time';
import { assertUsageAvailable, normalizeVoucherCode } from '../../src/domain/voucher';
import type { Outcome, Scenario } from './voucher-model';

export const VARIANTS = ['dung', 'V1-trim', 'V2-ngay-utc', 'V3-pham-vi', 'V4-luot-utc'] as const;
export type Variant = (typeof VARIANTS)[number];

const DAY_MS = 24 * 60 * 60 * 1000;

function utcDayRange(at: Date) {
  const start = new Date(Math.floor(at.getTime() / DAY_MS) * DAY_MS);
  return { start, end: new Date(start.getTime() + DAY_MS) };
}

export function evaluate(variant: Variant, s: Scenario): Outcome {
  try {
    const request = normalizePriceRequest(s.input);
    const raw = s.input.voucherCode;
    const code = variant === 'V1-trim' ? (raw ?? '').toUpperCase() || null : normalizeVoucherCode(raw);
    if (code === null) throw new Error('kịch bản luôn có mã voucher');
    if (code !== s.voucher.code) throw new DomainError('VOUCHER_NOT_FOUND', 'Không tìm thấy voucher.');

    const day = variant === 'V2-ngay-utc' ? s.at.toISOString().slice(0, 10) : vnDate(s.at);
    if (day < s.voucher.validFrom || day > s.voucher.validTo) throw new DomainError('VOUCHER_EXPIRED', 'Hết hạn.');
    const kindOk = s.voucher.kinds.length === 0 || s.voucher.kinds.includes(request.kind);
    const cityOk =
      variant === 'V3-pham-vi'
        ? s.voucher.kinds.length === 0 || s.voucher.cityCodes.includes(request.cityCode)
        : s.voucher.cityCodes.length === 0 || s.voucher.cityCodes.includes(request.cityCode);
    if (!kindOk || !cityOk) throw new DomainError('VOUCHER_NOT_APPLICABLE', 'Không áp dụng.');

    const price = priceRequest(request, { peak: isPeakTime(s.at), raining: false, voucher: s.voucher });
    const { start, end } = variant === 'V4-luot-utc' ? utcDayRange(s.at) : vnDayRange(s.at);
    assertUsageAvailable(s.voucher, s.uses.filter((u) => u >= start && u < end).length);
    return { ok: true, discount: price.discount };
  } catch (error) {
    if (error instanceof DomainError) return { ok: false, code: error.code };
    throw error;
  }
}

/** Test case thất bại trên bản build này? (kết quả khác kết quả mong đợi) */
export function fails(variant: Variant, s: Scenario): boolean {
  return JSON.stringify(evaluate(variant, s)) !== JSON.stringify(s.expected);
}
