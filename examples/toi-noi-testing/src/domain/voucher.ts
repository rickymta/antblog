import { DomainError } from './errors';
import { formatDong, percentOf, toMilli } from './money';
import { vnDate } from './time';
import type { CityCode, TripKind } from './types';

/** Voucher giảm giá (SPEC R-VOUCHER-01). Ngày dạng `YYYY-MM-DD` theo giờ Việt Nam. */
export interface Voucher {
  code: string;
  percent: number; // 1–50
  maxDiscount: number; // đồng
  minAmount: number; // đồng
  kinds: TripKind[]; // rỗng = mọi dịch vụ
  cityCodes: CityCode[]; // rỗng = mọi thành phố
  validFrom: string;
  validTo: string;
  perCustomerPerDay: number;
}

/** Phần voucher mà công thức cước cần. */
export type VoucherTerms = Pick<Voucher, 'code' | 'percent' | 'maxDiscount' | 'minAmount'>;

/** Bỏ khoảng trắng hai đầu và viết hoa; chuỗi rỗng ⇒ không có voucher (SPEC R-VOUCHER-02). */
export function normalizeVoucherCode(raw: string | null | undefined): string | null {
  const code = (raw ?? '').trim().toUpperCase();
  return code === '' ? null : code;
}

/** Thời hạn và phạm vi áp dụng (SPEC R-VOUCHER-04, R-VOUCHER-05). */
export function assertVoucherEligible(
  voucher: Voucher,
  context: { kind: TripKind; cityCode: CityCode; at: Date },
): void {
  const day = vnDate(context.at);
  if (day < voucher.validFrom || day > voucher.validTo) {
    throw new DomainError(
      'VOUCHER_EXPIRED',
      `Voucher ${voucher.code} chỉ dùng được từ ${voucher.validFrom} đến ${voucher.validTo}.`,
    );
  }
  const kindOk = voucher.kinds.length === 0 || voucher.kinds.includes(context.kind);
  const cityOk = voucher.cityCodes.length === 0 || voucher.cityCodes.includes(context.cityCode);
  if (!kindOk || !cityOk) {
    throw new DomainError('VOUCHER_NOT_APPLICABLE', `Voucher ${voucher.code} không áp dụng cho chuyến này.`);
  }
}

/**
 * Tiền giảm (milli-đồng) trên số tiền A (milli-đồng) — SPEC R-VOUCHER-06.
 * A < minAmount ⇒ VOUCHER_MIN_AMOUNT; tiền giảm = min(percent% × A, maxDiscount).
 */
export function voucherDiscountMilli(terms: VoucherTerms, amountMilli: number): number {
  if (amountMilli < toMilli(terms.minAmount)) {
    throw new DomainError(
      'VOUCHER_MIN_AMOUNT',
      `Voucher ${terms.code} cần đơn tối thiểu ${formatDong(terms.minAmount)}.`,
    );
  }
  return Math.min(percentOf(amountMilli, terms.percent), toMilli(terms.maxDiscount));
}

/** Lượt dùng trong ngày (SPEC R-VOUCHER-07): `usedToday` = số chuyến chưa huỷ đã dùng mã hôm nay. */
export function assertUsageAvailable(voucher: Voucher, usedToday: number): void {
  if (usedToday >= voucher.perCustomerPerDay) {
    throw new DomainError(
      'VOUCHER_USAGE_LIMIT',
      `Bạn đã dùng hết ${voucher.perCustomerPerDay} lượt voucher ${voucher.code} hôm nay.`,
    );
  }
}
