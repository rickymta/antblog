/**
 * Bốn test case của tester cho bài TEST-001. Kết quả mong đợi (oracle) tính tay từ SPEC,
 * KHÔNG lấy từ code đang được kiểm thử.
 */
import type { FareInput } from '../../src/domain/fare';

const GIAM20 = { code: 'GIAM20', percent: 20, maxDiscount: 30_000, minAmount: 0 }; // SPEC Phụ lục A

export interface FareTestCase {
  id: string;
  title: string;
  basis: string; // mã yêu cầu trong SPEC
  input: FareInput;
  expectedTotal: number; // đồng
}

export const FARE_TEST_CASES: FareTestCase[] = [
  {
    id: 'TC-FARE-01',
    title: 'Xe máy 7,4 km, cao điểm, mưa, GIAM20',
    basis: 'R-FARE-09',
    input: { vehicle: 'bike', distanceKm: 7.4, peak: true, raining: true, voucher: GIAM20 },
    expectedTotal: 52_000, // 45.300 → 56.625 → 64.625 → 51.700 → làm tròn lên 52.000
  },
  {
    id: 'TC-FARE-02',
    title: 'Xe máy 3 km, giờ thường, trời khô',
    basis: 'R-FARE-01, R-FARE-08',
    input: { vehicle: 'bike', distanceKm: 3, peak: false, raining: false },
    expectedTotal: 26_000, // 12.000 + 4.500 × 3 = 25.500 → làm tròn lên 26.000
  },
  {
    id: 'TC-FARE-03',
    title: 'Xe máy 0,5 km, chạm cước tối thiểu',
    basis: 'R-FARE-07',
    input: { vehicle: 'bike', distanceKm: 0.5, peak: false, raining: false },
    expectedTotal: 15_000, // 14.250 < 15.000 → lấy 15.000
  },
  {
    id: 'TC-FARE-04',
    title: 'Ô tô 10 km, giờ thường, trời khô',
    basis: 'R-FARE-01',
    input: { vehicle: 'car', distanceKm: 10, peak: false, raining: false },
    expectedTotal: 135_000, // 25.000 + 11.000 × 10 = 135.000 (đã tròn nghìn)
  },
];
