/**
 * TCD-04 — test oracle độc lập cho cước đặt xe/bưu kiện (SPEC R-FARE-01…10, R-PARCEL-01…04, R-VOUCHER-06).
 *
 * Độc lập nghĩa là: không import gì từ `src/`, tham số chép từ SPEC, tiền là phân số BigInt (không float),
 * quãng đường/khối lượng đọc từ dạng thập phân của số rồi làm tròn half-up bằng số nguyên.
 * Kết quả là bảng chi tiết đủ 11 bước của R-FARE-10 để so TỪNG bước chứ không chỉ tổng.
 */
import { PhanSo } from './phan-so';

export type Xe = 'bike' | 'car';

const BANG_GIA: Record<Xe, { moCua: number; moiKm: number; toiThieu: number }> = {
  bike: { moCua: 12_000, moiKm: 4_500, toiThieu: 15_000 }, // R-FARE-01
  car: { moCua: 25_000, moiKm: 11_000, toiThieu: 30_000 },
};
const HE_SO_CAO_DIEM = PhanSo.of(125, 100); // R-FARE-04
const PHU_PHI_MUA = 8_000; // R-FARE-05

/** Voucher theo SPEC Phụ lục A (chỉ phần công thức cần). */
export const VOUCHER_SPEC: Record<string, { percent: number; tran: number; toiThieu: number }> = {
  GIAM20: { percent: 20, tran: 30_000, toiThieu: 0 },
  HN10: { percent: 10, tran: 15_000, toiThieu: 40_000 },
};

export interface DauVaoCuoc {
  xe: Xe;
  km: number;
  kg?: number; // có ⇒ bưu kiện
  caoDiem: boolean;
  mua: boolean;
  voucher?: string;
}

export type ChiTiet = Record<
  'openingFee' | 'distanceCharge' | 'parcelSurcharge' | 'base' | 'afterPeak' | 'rainSurcharge'
  | 'beforeDiscount' | 'discount' | 'afterDiscount' | 'minimumFare' | 'total',
  PhanSo
> & { distanceKm: PhanSo };

export class LoiNghiepVu extends Error {
  constructor(readonly code: string) { super(code); }
}

/** Làm tròn tới 0,1 half-up, trả số «phần mười» nguyên (R-FARE-02, R-PARCEL-01). */
export function phanMuoi(x: number): bigint {
  return PhanSo.tuSo(x).nhan(PhanSo.of(10)).lamTronNuaLen();
}

export function oracleCuoc(v: DauVaoCuoc): ChiTiet {
  const gia = BANG_GIA[v.xe];
  // R-FARE-02: làm tròn trước, rồi kiểm [0,5; 100]
  const km10 = phanMuoi(v.km);
  if (km10 < 5n || km10 > 1000n) throw new LoiNghiepVu('DISTANCE_OUT_OF_RANGE');
  // R-PARCEL-01…03
  let phuPhiBuuKien = 0n;
  if (v.kg !== undefined) {
    const kg10 = phanMuoi(v.kg);
    if (kg10 <= 0n || kg10 > 300n) throw new LoiNghiepVu('WEIGHT_OUT_OF_RANGE');
    if (kg10 > 200n && v.xe === 'bike') throw new LoiNghiepVu('VEHICLE_TOO_SMALL');
    if (kg10 > 50n) phuPhiBuuKien = 2_000n * PhanSo.of(kg10 - 50n, 10n).tran(); // «mỗi kg bắt đầu»
  }

  const openingFee = PhanSo.of(gia.moCua);
  const distanceCharge = PhanSo.of(gia.moiKm).nhan(PhanSo.of(km10, 10n));
  const parcelSurcharge = PhanSo.of(phuPhiBuuKien);
  const base = openingFee.cong(distanceCharge).cong(parcelSurcharge); // bước 1
  const afterPeak = v.caoDiem ? base.nhan(HE_SO_CAO_DIEM) : base; // bước 2
  const rainSurcharge = PhanSo.of(v.mua ? PHU_PHI_MUA : 0);
  const beforeDiscount = afterPeak.cong(rainSurcharge); // bước 3

  let discount = PhanSo.of(0); // bước 4 (R-FARE-06, R-VOUCHER-06)
  if (v.voucher) {
    const vc = VOUCHER_SPEC[v.voucher];
    if (!vc) throw new Error(`Voucher ${v.voucher} chưa chép từ SPEC`);
    if (beforeDiscount.nhoHon(PhanSo.of(vc.toiThieu))) throw new LoiNghiepVu('VOUCHER_MIN_AMOUNT');
    discount = beforeDiscount.nhan(PhanSo.of(vc.percent, 100)).nhoNhat(PhanSo.of(vc.tran));
  }
  const afterDiscount = beforeDiscount.tru(discount);
  const minimumFare = PhanSo.of(gia.toiThieu);
  const atLeast = afterDiscount.lonNhat(minimumFare); // bước 5
  const total = PhanSo.of(atLeast.nhan(PhanSo.of(1, 1000)).tran() * 1000n); // bước 6

  return {
    distanceKm: PhanSo.of(km10, 10n),
    openingFee, distanceCharge, parcelSurcharge, base, afterPeak, rainSurcharge,
    beforeDiscount, discount, afterDiscount, minimumFare, total,
  };
}

/** So mọi trường của bảng chi tiết hệ thống trả về với oracle; trả danh sách trường lệch (rỗng = khớp). */
export function soChiTiet(heThong: Record<string, unknown>, oracle: ChiTiet): string[] {
  const lech: string[] = [];
  for (const [k, mongDoi] of Object.entries(oracle)) {
    const thucTe = heThong[k];
    if (typeof thucTe !== 'number' || !PhanSo.tuSo(thucTe).bang(mongDoi)) {
      lech.push(`${k}: thực tế ${String(thucTe)}, mong đợi ${mongDoi.toString()}`);
    }
  }
  return lech;
}
