/**
 * TEST-045 «Condition Coverage» — quyết định «voucher áp được» và bốn atomic condition của nó
 * (SPEC R-VOUCHER-04, R-VOUCHER-05, R-VOUCHER-06):
 *
 *   apDuoc = trongHan && dungDichVu && dungThanhPho && duTien
 *
 * Có hai cách nhìn một test chạm vào các điều kiện:
 * - «bảng chân trị»: mọi điều kiện đều có giá trị (tính từ dữ liệu test theo SPEC);
 * - «được đánh giá»: chỉ những điều kiện mà code THẬT của lab đã tính tới trước khi ném lỗi.
 */
import { DomainError } from '../../src/domain/errors';
import { vnDate } from '../../src/domain/time';
import type { CityCode, TripKind } from '../../src/domain/types';
import { assertVoucherEligible, voucherDiscountMilli, type Voucher } from '../../src/domain/voucher';
import { toDong, toMilli } from '../../src/domain/money';

export const TEN_DIEU_KIEN = ['trongHan', 'dungDichVu', 'dungThanhPho', 'duTien'] as const;
export type TenDieuKien = (typeof TEN_DIEU_KIEN)[number];
export type GiaTriDieuKien = Record<TenDieuKien, boolean>;

/** Dữ liệu một test: thời điểm đặt, dịch vụ, thành phố, số tiền A (đồng). */
export interface DuLieuTest {
  id: string;
  at: Date;
  kind: TripKind;
  cityCode: CityCode;
  amountDong: number;
}

/** Giá trị bốn điều kiện tính THẲNG từ SPEC — độc lập với `src/` (đây là oracle). */
export function giaTriTheoSpec(v: Voucher, t: DuLieuTest): GiaTriDieuKien {
  const day = vnDate(t.at);
  return {
    trongHan: v.validFrom <= day && day <= v.validTo, // R-VOUCHER-04, tính cả hai đầu
    dungDichVu: v.kinds.length === 0 || v.kinds.includes(t.kind), // R-VOUCHER-05
    dungThanhPho: v.cityCodes.length === 0 || v.cityCodes.includes(t.cityCode), // R-VOUCHER-05
    duTien: t.amountDong >= v.minAmount, // R-VOUCHER-06: A = minAmount là hợp lệ
  };
}

export const quyetDinh = (g: GiaTriDieuKien): boolean =>
  g.trongHan && g.dungDichVu && g.dungThanhPho && g.duTien;

export interface KetQuaChayThat {
  apDuoc: boolean;
  ma: string | null; // mã lỗi DomainError, null nếu áp được
  tienGiamDong: number | null;
  /** Điều kiện code thật đã tính tới — suy từ cấu trúc assertVoucherEligible → voucherDiscountMilli. */
  daDanhGia: TenDieuKien[];
}

/** Chạy code THẬT của lab theo đúng thứ tự trong quote.ts: thời hạn/phạm vi rồi tiền tối thiểu. */
export function chayThat(v: Voucher, t: DuLieuTest): KetQuaChayThat {
  try {
    assertVoucherEligible(v, { kind: t.kind, cityCode: t.cityCode, at: t.at });
    const giam = voucherDiscountMilli(v, toMilli(t.amountDong));
    return { apDuoc: true, ma: null, tienGiamDong: toDong(giam), daDanhGia: [...TEN_DIEU_KIEN] };
  } catch (e) {
    if (!(e instanceof DomainError)) throw e;
    // VOUCHER_EXPIRED ném trước khi tính kindOk/cityOk; kindOk và cityOk được tính CẢ HAI
    // (gán vào biến) trước `if (!kindOk || !cityOk)`; minAmount chỉ kiểm khi đã qua hai bước trên.
    const daDanhGia: TenDieuKien[] =
      e.code === 'VOUCHER_EXPIRED'
        ? ['trongHan']
        : e.code === 'VOUCHER_NOT_APPLICABLE'
          ? ['trongHan', 'dungDichVu', 'dungThanhPho']
          : [...TEN_DIEU_KIEN];
    return { apDuoc: false, ma: e.code, tienGiamDong: null, daDanhGia };
  }
}
