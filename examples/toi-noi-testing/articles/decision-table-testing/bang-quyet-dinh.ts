/**
 * TEST-033 — Bảng quyết định (decision table) cho việc huỷ chuyến (SPEC R-CANCEL-01…06, R-STATE-02/04/05).
 * Bảng được viết thành DỮ LIỆU để test vừa kiểm chính bảng (đủ, không chồng lấn, checksum)
 * vừa dùng bảng làm oracle cho ca kiểm thử. Không sửa src/.
 */
import type { FeeReason } from '../../src/domain/cancel';

// ---------------------------------------------------------------- điều kiện (conditions)

/** C1 — Ai gửi lệnh huỷ. «Tài xế» = tài xế đang được gán cho chuyến (R-API-11). */
export type NguoiHuy = 'khach' | 'tai_xe' | 'he_thong';
/** C2 — Trạng thái lúc huỷ. «khac» = requested, picked_up, in_transit, completed, cancelled. */
export type TrangThai = 'searching' | 'accepted' | 'arriving' | 'khac';
/** C5 — Loại xe của chuyến (đơn đồ ăn luôn là xe máy). */
export type Xe = 'bike' | 'car';

export interface Combo {
  nguoiHuy: NguoiHuy;
  trangThai: TrangThai;
  /** C3 — now − acceptedAt ≥ 2 phút (R-CANCEL-02). */
  du2Phut: boolean;
  /** C4 — now > acceptedAt + etaMinutes + 10 phút (R-CANCEL-04). */
  taiXeTre: boolean;
  xe: Xe;
}

export const GIA_TRI = {
  nguoiHuy: ['khach', 'tai_xe', 'he_thong'] as const,
  trangThai: ['searching', 'accepted', 'arriving', 'khac'] as const,
  du2Phut: [false, true] as const,
  taiXeTre: [false, true] as const,
  xe: ['bike', 'car'] as const,
};

// ---------------------------------------------------------------- hành động (actions)

export type KetQua =
  | { loai: 'huy'; phi: number; feeReason: FeeReason | null; lyDoHuy?: 'no_driver' } // chuyến → cancelled
  | { loai: 've_searching' } // tài xế huỷ: chuyến mở lại, bỏ gán tài xế (R-STATE-04)
  | { loai: 'tu_choi'; ma: 'INVALID_TRANSITION' }; // chuyến giữ nguyên (R-STATE-03)

// ---------------------------------------------------------------- quy tắc (rules = cột)

const _ = '-' as const;
type Or<T> = T | typeof _;

export interface Rule {
  id: string;
  when: { nguoiHuy: NguoiHuy; trangThai: TrangThai; du2Phut: Or<boolean>; taiXeTre: Or<boolean>; xe: Or<Xe> };
  then: KetQua;
  /** Bộ giá trị cụ thể chọn để chạy (thay mỗi «–» bằng một giá trị). */
  dai: Combo;
}

const huy = (phi: number, feeReason: FeeReason | null): KetQua => ({ loai: 'huy', phi, feeReason });
const TU_CHOI: KetQua = { loai: 'tu_choi', ma: 'INVALID_TRANSITION' };
const c = (nguoiHuy: NguoiHuy, trangThai: TrangThai, du2Phut: boolean, taiXeTre: boolean, xe: Xe): Combo => ({
  nguoiHuy, trangThai, du2Phut, taiXeTre, xe,
});

/** Bảng đã rút gọn: 13 cột. */
export const RULES: readonly Rule[] = [
  { id: 'DT-01', when: { nguoiHuy: 'khach', trangThai: 'searching', du2Phut: false, taiXeTre: false, xe: _ }, then: huy(0, 'searching'), dai: c('khach', 'searching', false, false, 'bike') },
  { id: 'DT-02', when: { nguoiHuy: 'khach', trangThai: 'accepted', du2Phut: false, taiXeTre: false, xe: _ }, then: huy(0, 'grace_period'), dai: c('khach', 'accepted', false, false, 'car') },
  { id: 'DT-03', when: { nguoiHuy: 'khach', trangThai: 'accepted', du2Phut: true, taiXeTre: false, xe: 'bike' }, then: huy(10_000, 'charged'), dai: c('khach', 'accepted', true, false, 'bike') },
  { id: 'DT-04', when: { nguoiHuy: 'khach', trangThai: 'accepted', du2Phut: true, taiXeTre: false, xe: 'car' }, then: huy(20_000, 'charged'), dai: c('khach', 'accepted', true, false, 'car') },
  { id: 'DT-05', when: { nguoiHuy: 'khach', trangThai: 'accepted', du2Phut: true, taiXeTre: true, xe: _ }, then: huy(0, 'driver_late'), dai: c('khach', 'accepted', true, true, 'car') },
  { id: 'DT-06', when: { nguoiHuy: 'khach', trangThai: 'arriving', du2Phut: _, taiXeTre: false, xe: 'bike' }, then: huy(10_000, 'charged'), dai: c('khach', 'arriving', false, false, 'bike') },
  { id: 'DT-07', when: { nguoiHuy: 'khach', trangThai: 'arriving', du2Phut: _, taiXeTre: false, xe: 'car' }, then: huy(20_000, 'charged'), dai: c('khach', 'arriving', true, false, 'car') },
  { id: 'DT-08', when: { nguoiHuy: 'khach', trangThai: 'arriving', du2Phut: true, taiXeTre: true, xe: _ }, then: huy(0, 'driver_late'), dai: c('khach', 'arriving', true, true, 'bike') },
  { id: 'DT-09', when: { nguoiHuy: 'khach', trangThai: 'khac', du2Phut: _, taiXeTre: _, xe: _ }, then: TU_CHOI, dai: c('khach', 'khac', true, false, 'bike') },
  { id: 'DT-10', when: { nguoiHuy: 'tai_xe', trangThai: 'accepted', du2Phut: _, taiXeTre: _, xe: _ }, then: { loai: 've_searching' }, dai: c('tai_xe', 'accepted', false, false, 'bike') },
  { id: 'DT-11', when: { nguoiHuy: 'tai_xe', trangThai: 'arriving', du2Phut: _, taiXeTre: _, xe: _ }, then: { loai: 've_searching' }, dai: c('tai_xe', 'arriving', true, false, 'car') },
  { id: 'DT-12', when: { nguoiHuy: 'tai_xe', trangThai: 'khac', du2Phut: _, taiXeTre: _, xe: _ }, then: TU_CHOI, dai: c('tai_xe', 'khac', true, false, 'bike') },
  { id: 'DT-13', when: { nguoiHuy: 'he_thong', trangThai: 'searching', du2Phut: false, taiXeTre: false, xe: _ }, then: { loai: 'huy', phi: 0, feeReason: null, lyDoHuy: 'no_driver' }, dai: c('he_thong', 'searching', false, false, 'car') },
];

// ---------------------------------------------------------------- bảng đầy đủ, tính khả thi, so khớp

/** Mọi tổ hợp của bảng đầy đủ: 3 × 4 × 2 × 2 × 2 = 96 cột. */
export function fullTable(): Combo[] {
  const out: Combo[] = [];
  for (const nguoiHuy of GIA_TRI.nguoiHuy)
    for (const trangThai of GIA_TRI.trangThai)
      for (const du2Phut of GIA_TRI.du2Phut)
        for (const taiXeTre of GIA_TRI.taiXeTre)
          for (const xe of GIA_TRI.xe) out.push({ nguoiHuy, trangThai, du2Phut, taiXeTre, xe });
  return out;
}

/** Lý do một tổ hợp KHÔNG THỂ xảy ra (infeasible), hoặc null nếu xảy ra được. */
export function infeasibleReason(x: Combo): string | null {
  if (!x.du2Phut && x.taiXeTre) return 'trễ nghĩa là đã qua ≥ ETA + 10 ≥ 11 phút ⇒ chắc chắn đủ 2 phút';
  if (x.trangThai === 'searching' && x.du2Phut) return 'chuyến searching không có acceptedAt (R-STATE-04 xoá khi tài xế huỷ)';
  if (x.nguoiHuy === 'tai_xe' && x.trangThai === 'searching') return 'chuyến searching chưa có tài xế được gán';
  if (x.nguoiHuy === 'he_thong' && x.trangThai !== 'searching') return 'hệ thống chỉ huỷ chuyến searching quá 5 phút (R-STATE-05)';
  return null;
}

export function matches(rule: Rule, x: Combo): boolean {
  const w = rule.when;
  return (
    w.nguoiHuy === x.nguoiHuy &&
    w.trangThai === x.trangThai &&
    (w.du2Phut === _ || w.du2Phut === x.du2Phut) &&
    (w.taiXeTre === _ || w.taiXeTre === x.taiXeTre) &&
    (w.xe === _ || w.xe === x.xe)
  );
}

/** Checksum theo CTAL-TA 3.3.1: mỗi «–» nhân điểm của cột với số giá trị của điều kiện đó. */
export function checksum(rules: readonly Rule[] = RULES): number {
  return rules.reduce((sum, r) => {
    let score = 1;
    if (r.when.du2Phut === _) score *= GIA_TRI.du2Phut.length;
    if (r.when.taiXeTre === _) score *= GIA_TRI.taiXeTre.length;
    if (r.when.xe === _) score *= GIA_TRI.xe.length;
    return sum + score;
  }, 0);
}

export function nhan(x: Combo): string {
  const t = (b: boolean) => (b ? 'T' : 'F');
  return `${x.nguoiHuy}/${x.trangThai}/≥2′:${t(x.du2Phut)}/trễ:${t(x.taiXeTre)}/${x.xe}`;
}
