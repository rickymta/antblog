/** Ca kiểm thử dùng chung cho bài TEST-022. Kết quả mong đợi tính theo SPEC R-FARE (đã kiểm bằng src/domain/fare.ts). */
import type { FareInput } from '../../src/domain/fare';

export const GIAM20 = { code: 'GIAM20', percent: 20, maxDiscount: 30_000, minAmount: 0 };

export interface FareCase {
  id: string;
  basis: string;
  title: string;
  input: FareInput;
  expectedTotal: number;
}

/** Bộ test «mỗi ca một điều kiện» — kiểu bộ test hay được viết đầu tiên. */
export const SINGLE_CONDITION_CASES: FareCase[] = [
  { id: 'TC-01', basis: 'R-FARE-01', title: 'xe máy 7,4 km, không điều kiện', input: { vehicle: 'bike', distanceKm: 7.4, peak: false, raining: false }, expectedTotal: 46_000 },
  { id: 'TC-02', basis: 'R-FARE-04', title: 'chỉ cao điểm', input: { vehicle: 'bike', distanceKm: 7.4, peak: true, raining: false }, expectedTotal: 57_000 },
  { id: 'TC-03', basis: 'R-FARE-05', title: 'chỉ mưa', input: { vehicle: 'bike', distanceKm: 7.4, peak: false, raining: true }, expectedTotal: 54_000 },
  { id: 'TC-04', basis: 'R-FARE-06', title: 'chỉ voucher GIAM20', input: { vehicle: 'bike', distanceKm: 7.4, peak: false, raining: false, voucher: GIAM20 }, expectedTotal: 37_000 },
];

/** Ca tổ hợp: chính ví dụ kiểm R-FARE-09. */
export const TC_COMBO: FareCase = {
  id: 'TC-05', basis: 'R-FARE-09', title: 'cao điểm + mưa + GIAM20',
  input: { vehicle: 'bike', distanceKm: 7.4, peak: true, raining: true, voucher: GIAM20 }, expectedTotal: 52_000,
};

/** Ca ô tô 18,6 km cao điểm: 25.000 + 11.000 × 18,6 = 229.600 ⇒ × 1,25 = 287.000 đúng bội số 1.000. */
export const TC_CAR: FareCase = {
  id: 'TC-06', basis: 'R-FARE-03, R-FARE-08', title: 'ô tô 18,6 km, cao điểm',
  input: { vehicle: 'car', distanceKm: 18.6, peak: true, raining: false }, expectedTotal: 287_000,
};
