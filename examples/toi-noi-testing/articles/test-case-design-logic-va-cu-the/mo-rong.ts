/**
 * TCD-03 — mở rộng test case logic thành test case cụ thể.
 *
 * Hàm `moRong` ghép mỗi test case logic với các dòng test data của nó, điền tiền điều kiện cụ thể
 * (thời điểm, thời tiết), thân yêu cầu HTTP và kết quả mong đợi TÍNH TỪ SPEC. Không import gì từ `src/`
 * ngoài kiểu dữ liệu: tham số giá chép từ SPEC (R-FARE-01, -04, -05, -07, -08; R-FOOD-03; Phụ lục A),
 * để kết quả mong đợi không phụ thuộc vào chính mã đang được kiểm.
 */
import { BANG_DU_LIEU, TEST_CASE_LOGIC, tienMon, type DongDuLieu, type Loi, type TestCaseLogic } from './test-case-logic';

// ---- tham số chép từ SPEC
const XE_MAY = { moCua: 12_000, moiKm: 4_500, toiThieu: 15_000 }; // R-FARE-01
const PHU_PHI_MUA = 8_000; // R-FARE-05
const TIEN_MON_TOI_THIEU = 30_000; // R-FOOD-03
const VOUCHER: Record<string, { percent: number; tran: number; toiThieu: number }> = {
  GIAM20: { percent: 20, tran: 30_000, toiThieu: 0 }, // Phụ lục A
  FOOD15: { percent: 15, tran: 25_000, toiThieu: 50_000 },
};

/** Thời điểm cụ thể (giờ Việt Nam, UTC+7) cho «cao điểm» / «ngoài cao điểm» — R-FARE-04, R-GEN-02. */
export const THOI_DIEM = {
  ngoaiCaoDiem: { iso: '2026-11-04T09:00:00+07:00', moTa: '04/11/2026 09:00 giờ VN' },
  caoDiem: { iso: '2026-11-04T11:30:00+07:00', moTa: '04/11/2026 11:30 giờ VN' },
} as const;

/**
 * Phí giao theo công thức xe máy (R-FOOD-04): tính bằng số nguyên «phần tư đồng» để ×1,25 không sinh
 * sai số dấu phẩy động. Quãng đường của bảng dữ liệu đã có đúng một chữ số thập phân.
 */
export function phiGiaoTheoSpec(km: number, caoDiem: boolean, mua: boolean): number {
  const phanMuoiKm = Math.round(km * 10);
  if (Math.abs(km * 10 - phanMuoiKm) > 1e-9) throw new Error(`Dữ liệu ${km} km phải có tối đa một chữ số thập phân`);
  const coBan = XE_MAY.moCua + (XE_MAY.moiKm / 10) * phanMuoiKm; // đồng, nguyên vì 4.500/10 = 450
  let phanTu = coBan * 4; // đơn vị 1/4 đồng
  if (caoDiem) phanTu = coBan * 5; // ×1,25 = ×5/4
  if (mua) phanTu += PHU_PHI_MUA * 4;
  phanTu = Math.max(phanTu, XE_MAY.toiThieu * 4); // không có voucher trên phí giao
  return Math.ceil(phanTu / 4_000) * 1_000; // làm tròn lên 1.000 đ
}

export function tienGiamTheoSpec(ma: string | undefined, mon: number): number | Loi {
  if (!ma) return 0;
  const v = VOUCHER[ma];
  if (!v) throw new Error(`Bảng dữ liệu dùng voucher ${ma} chưa có trong SPEC Phụ lục A`);
  if (mon < v.toiThieu) return { http: 422, code: 'VOUCHER_MIN_AMOUNT' };
  const giam = (v.percent * mon) / 100;
  if (!Number.isInteger(giam)) throw new Error('R-FOOD-05: tiền giảm phải là số nguyên đồng');
  return Math.min(giam, v.tran);
}

export type MongDoi =
  | { http: 200; itemsSubtotal: number; itemsDiscount: number; deliveryFee: number; total: number; vehicle: 'bike' }
  | { http: 400 | 422; code: string };

export interface TestCaseCuThe {
  id: string; // TC-FOOD-nnn/TD-nn
  logic: string;
  tieuDe: string;
  truyVet: string[];
  tienDieuKien: { dongHo: string; moTaDongHo: string; thanhPhoMua: string[] };
  /** Thân `POST /api/quotes`, khách C001. */
  body: Record<string, unknown>;
  mongDoi: MongDoi;
}

function mongDoiTheoSpec(tc: TestCaseLogic, d: DongDuLieu): MongDoi {
  if (tc.loi) return tc.loi; // ca âm: mã lỗi đã cụ thể ở mức logic
  const mon = tienMon(d);
  if (mon < TIEN_MON_TOI_THIEU) return { http: 422, code: 'SUBTOTAL_TOO_LOW' };
  const giam = tienGiamTheoSpec(d.voucher, mon);
  if (typeof giam !== 'number') return giam;
  const phi = phiGiaoTheoSpec(d.km, d.caoDiem, d.mua);
  return { http: 200, itemsSubtotal: mon, itemsDiscount: giam, deliveryFee: phi, total: mon - giam + phi, vehicle: 'bike' };
}

/** Một test case logic × các dòng dữ liệu của nó ⇒ các test case cụ thể. */
export function moRong(tc: TestCaseLogic, bang: readonly DongDuLieu[] = BANG_DU_LIEU): TestCaseCuThe[] {
  const dong = bang.filter((d) => d.tc === tc.id);
  if (dong.length === 0) throw new Error(`${tc.id} chưa có dòng test data nào`);
  return dong.map((d) => {
    if (!tc.rangBuoc(d)) throw new Error(`${d.id} không thuộc lớp mà ${tc.id} mô tả`);
    const thoiDiem = d.caoDiem ? THOI_DIEM.caoDiem : THOI_DIEM.ngoaiCaoDiem;
    const body: Record<string, unknown> = { kind: 'food', cityCode: 'SG', distanceKm: d.km, items: d.mon };
    if (d.vehicle) body.vehicle = d.vehicle;
    if (d.voucher) body.voucherCode = d.voucher;
    return {
      id: `${tc.id}/${d.id}`,
      logic: tc.id,
      tieuDe: tc.tieuDe,
      truyVet: tc.truyVet,
      tienDieuKien: { dongHo: thoiDiem.iso, moTaDongHo: thoiDiem.moTa, thanhPhoMua: d.mua ? ['SG'] : [] },
      body,
      mongDoi: mongDoiTheoSpec(tc, d),
    };
  });
}

export const TAT_CA_CU_THE: TestCaseCuThe[] = TEST_CASE_LOGIC.flatMap((tc) => moRong(tc));
