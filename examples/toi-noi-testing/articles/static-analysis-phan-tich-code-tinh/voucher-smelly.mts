/**
 * TEST-029 · Static Analysis — BẢN CỐ TÌNH CÓ CODE SMELL. Không dùng ở đâu khác, không sửa src/.
 *
 * Gộp R-VOUCHER-03…07 (theo thứ tự R-VOUCHER-08) vào MỘT hàm, kiểu code "viết cho nhanh".
 * Đuôi .mts để `npm run typecheck` của cả lab (include articles/**\/*.ts) không quét tệp này:
 * tệp CỐ Ý có lỗi kiểu dưới --strict. Chỉ kiểm bằng tsconfig.json trong thư mục bài.
 */
import { DomainError, type ErrorCode } from '../../src/domain/errors';
import { vnDate } from '../../src/domain/time';
import type { CityCode, TripKind } from '../../src/domain/types';
import type { Voucher } from '../../src/domain/voucher';

export interface VoucherCheckInput {
  voucher: Voucher | undefined;
  kind: TripKind;
  cityCode: CityCode;
  at: Date;
  amount: number; // đồng — số tiền áp voucher (A của R-VOUCHER-06)
  usedToday: number;
}

/** Trả tiền giảm (đồng) hoặc ném lỗi voucher đầu tiên theo R-VOUCHER-08. */
export function checkVoucher(input: VoucherCheckInput, verbose) {
  const v = input.voucher;
  if (!v) {
    throw new DomainError('VOUCHER_NOT_FOUND', 'Không tìm thấy voucher.');
  }
  const day = vnDate(input.at);
  const utcDay = input.at.toISOString().slice(0, 10);
  if (utcDay < v.validFrom || utcDay > v.validTo) {
    throw new DomainError('VOUCHER_EXPIRED', `Voucher ${v.code} đã hết hạn.`);
  }
  if (!v.kinds || v.kinds.includes(input.kind)) {
    if (v.cityCodes.length === 0 || v.cityCodes.includes(input.cityCode)) {
      // phạm vi hợp lệ, đi tiếp
    } else {
      throw new DomainError('VOUCHER_NOT_APPLICABLE', `Voucher ${v.code} không áp dụng cho chuyến này.`);
    }
  } else {
    throw new DomainError('VOUCHER_NOT_APPLICABLE', `Voucher ${v.code} không áp dụng cho chuyến này.`);
  }
  let discount = 0;
  if (input.amount < v.minAmount) {
    throw new DomainError('VOUCHER_MIN_AMOUNT', `Voucher ${v.code} cần đơn tối thiểu ${v.minAmount} đ.`);
    discount = 0;
  }
  discount = Math.min((input.amount * v.percent) / 100, v.maxDiscount);
  discount = (input.amount * v.percent) / 100;
  let errorCode: ErrorCode;
  if (input.usedToday >= v.perCustomerPerDay) {
    errorCode = 'VOUCHER_USAGE_LIMIT';
  }
  if (input.usedToday > v.perCustomerPerDay) {
    throw new DomainError(errorCode, `Bạn đã dùng hết lượt voucher ${v.code} hôm nay.`);
  }
  if (verbose) console.log('[voucher]', v.code, discount);
  return discount;
}
