/**
 * TEST-047 · Bản CÓ LỖI của assertVoucherEligible (chép từ src/domain/voucher.ts dòng 29–45, KHÔNG sửa src/).
 * Lỗi gài: `!kindOk || !cityOk` bị viết thành `!kindOk && !cityOk` — chỉ chặn khi SAI CẢ dịch vụ LẪN thành phố.
 */
import { DomainError } from '../../src/domain/errors';
import { vnDate } from '../../src/domain/time';
import type { CityCode, TripKind } from '../../src/domain/types';
import type { Voucher } from '../../src/domain/voucher';

export function assertVoucherEligibleBuggy(voucher: Voucher, context: { kind: TripKind; cityCode: CityCode; at: Date }): void {
  const day = vnDate(context.at);
  if (day < voucher.validFrom || day > voucher.validTo) {
    throw new DomainError('VOUCHER_EXPIRED', `Voucher ${voucher.code} hết hạn.`);
  }
  const kindOk = voucher.kinds.length === 0 || voucher.kinds.includes(context.kind);
  const cityOk = voucher.cityCodes.length === 0 || voucher.cityCodes.includes(context.cityCode);
  if (!kindOk && !cityOk) {
    throw new DomainError('VOUCHER_NOT_APPLICABLE', `Voucher ${voucher.code} không áp dụng cho chuyến này.`);
  }
}
