/**
 * TCD-04 — bảng test case giá trị cho báo giá đặt xe/bưu kiện.
 * Cột `tongTinhTay` do người viết tính bằng tay từ SPEC (xem bảng trong bài); oracle tính lại độc lập.
 */
import type { DauVaoCuoc } from './oracle-cuoc';

export interface DongCuoc extends DauVaoCuoc {
  id: string;
  thanhPho: 'SG' | 'HN';
  truyVet: string;
  tongTinhTay: number | { code: string };
}

export const BANG_CUOC: DongCuoc[] = [
  { id: 'OR-01', thanhPho: 'SG', xe: 'bike', km: 7.4, caoDiem: true, mua: true, voucher: 'GIAM20', truyVet: 'R-FARE-09', tongTinhTay: 52_000 },
  { id: 'OR-02', thanhPho: 'SG', xe: 'bike', km: 5, caoDiem: false, mua: false, truyVet: 'R-FARE-01', tongTinhTay: 35_000 },
  { id: 'OR-03', thanhPho: 'SG', xe: 'car', km: 18.6, caoDiem: true, mua: false, truyVet: 'R-FARE-04, R-GEN-01', tongTinhTay: 287_000 },
  { id: 'OR-04', thanhPho: 'SG', xe: 'bike', km: 0.45, caoDiem: false, mua: false, truyVet: 'R-FARE-02, R-FARE-07', tongTinhTay: 15_000 },
  { id: 'OR-05', thanhPho: 'SG', xe: 'bike', km: 0.44, caoDiem: false, mua: false, truyVet: 'R-FARE-02', tongTinhTay: { code: 'DISTANCE_OUT_OF_RANGE' } },
  { id: 'OR-06', thanhPho: 'SG', xe: 'car', km: 100.04, caoDiem: false, mua: false, truyVet: 'R-FARE-02', tongTinhTay: 1_125_000 },
  { id: 'OR-07', thanhPho: 'SG', xe: 'bike', km: 3.3, caoDiem: true, mua: false, truyVet: 'R-FARE-04, R-FARE-10', tongTinhTay: 34_000 },
  { id: 'OR-08', thanhPho: 'SG', xe: 'bike', km: 0.5, caoDiem: false, mua: false, voucher: 'GIAM20', truyVet: 'R-FARE-07', tongTinhTay: 15_000 },
  { id: 'OR-09', thanhPho: 'SG', xe: 'car', km: 100, caoDiem: false, mua: false, voucher: 'GIAM20', truyVet: 'R-VOUCHER-06', tongTinhTay: 1_095_000 },
  { id: 'OR-10', thanhPho: 'SG', xe: 'bike', km: 3, kg: 6, caoDiem: false, mua: false, truyVet: 'R-PARCEL-03', tongTinhTay: 28_000 },
  { id: 'OR-11', thanhPho: 'SG', xe: 'bike', km: 3, kg: 6.1, caoDiem: true, mua: false, truyVet: 'R-PARCEL-04', tongTinhTay: 37_000 },
  { id: 'OR-12', thanhPho: 'SG', xe: 'car', km: 3, kg: 30, caoDiem: false, mua: false, truyVet: 'R-PARCEL-03', tongTinhTay: 108_000 },
  { id: 'OR-13', thanhPho: 'HN', xe: 'bike', km: 5, caoDiem: false, mua: false, voucher: 'HN10', truyVet: 'R-VOUCHER-06', tongTinhTay: { code: 'VOUCHER_MIN_AMOUNT' } },
  { id: 'OR-14', thanhPho: 'HN', xe: 'car', km: 5, caoDiem: false, mua: false, voucher: 'HN10', truyVet: 'R-VOUCHER-06', tongTinhTay: 72_000 },
  { id: 'OR-15', thanhPho: 'SG', xe: 'bike', km: 7.35, caoDiem: false, mua: true, truyVet: 'R-FARE-02, R-FARE-05', tongTinhTay: 54_000 },
  { id: 'OR-16', thanhPho: 'SG', xe: 'car', km: 18.6, caoDiem: true, mua: false, voucher: 'GIAM20', truyVet: 'R-FARE-06, R-VOUCHER-06', tongTinhTay: 257_000 },
];
