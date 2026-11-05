/**
 * TEST-031 · Mô hình phân vùng tương đương cho POST /api/quotes (dịch vụ ride).
 * Ba tham số: quãng đường (R-FARE-02), mã thành phố (R-GEN-03), loại xe (R-GEN-04).
 * Mỗi ca test khai báo nó phủ những phân vùng nào ⇒ đo được EP coverage và Each Choice coverage.
 */

export type Param = 'distanceKm' | 'cityCode' | 'vehicle';

export interface Partition {
  id: string;
  param: Param;
  valid: boolean;
  description: string;
}

export const PARTITIONS: Partition[] = [
  { id: 'D1', param: 'distanceKm', valid: false, description: 'số, sau làm tròn 0,1 km nhỏ hơn 0,5 (gồm 0 và số âm)' },
  { id: 'D2', param: 'distanceKm', valid: true, description: 'số, sau làm tròn nằm trong [0,5; 100]' },
  { id: 'D3', param: 'distanceKm', valid: false, description: 'số, sau làm tròn lớn hơn 100' },
  { id: 'D4', param: 'distanceKm', valid: false, description: 'không phải số JSON (chuỗi, null, thiếu trường)' },
  { id: 'C1', param: 'cityCode', valid: true, description: 'một trong 6 mã SG, HN, DN, HP, CT, NT' },
  { id: 'C2', param: 'cityCode', valid: false, description: 'chuỗi 1–10 ký tự nhưng không phải mã được phục vụ' },
  { id: 'C3', param: 'cityCode', valid: false, description: 'sai định dạng: rỗng, quá 10 ký tự, không phải chuỗi, thiếu' },
  { id: 'V1', param: 'vehicle', valid: true, description: 'bike' },
  { id: 'V2', param: 'vehicle', valid: true, description: 'car' },
  { id: 'V3', param: 'vehicle', valid: false, description: 'giá trị khác bike/car' },
];

export interface ApiCase {
  id: string;
  covers: string[]; // id phân vùng mà ca này thật sự kiểm
  body: Record<string, unknown>;
  status: number;
  code?: string; // mã lỗi mong đợi
  field?: string; // trường phải xuất hiện trong error.details (lỗi định dạng)
  total?: number; // tổng tiền mong đợi (ca hợp lệ)
}

const ride = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 };

/**
 * Bộ ca Each Choice: 2 ca hợp lệ phủ mọi phân vùng hợp lệ, rồi MỖI phân vùng không hợp lệ một ca riêng,
 * các tham số còn lại giữ hợp lệ (tránh che lỗi — defect masking).
 * Thời điểm 09:00 giờ VN, không mưa, không voucher.
 */
export const API_CASES: ApiCase[] = [
  // xe máy 7,4 km: 12.000 + 4.500 × 7,4 = 45.300 ⇒ 46.000 đ
  { id: 'EP-01', covers: ['D2', 'C1', 'V1'], body: { ...ride }, status: 200, total: 46_000 },
  // ô tô 50 km ở HN: 25.000 + 11.000 × 50 = 575.000 đ
  { id: 'EP-02', covers: ['D2', 'C1', 'V2'], body: { ...ride, cityCode: 'HN', vehicle: 'car', distanceKm: 50 }, status: 200, total: 575_000 },
  { id: 'EP-03', covers: ['D1'], body: { ...ride, distanceKm: 0.2 }, status: 422, code: 'DISTANCE_OUT_OF_RANGE' },
  { id: 'EP-04', covers: ['D3'], body: { ...ride, distanceKm: 150 }, status: 422, code: 'DISTANCE_OUT_OF_RANGE' },
  { id: 'EP-05', covers: ['D4'], body: { ...ride, distanceKm: '7.4' }, status: 400, code: 'VALIDATION_ERROR', field: 'distanceKm' },
  { id: 'EP-06', covers: ['C2'], body: { ...ride, cityCode: 'DL' }, status: 422, code: 'CITY_NOT_SUPPORTED' },
  { id: 'EP-07', covers: ['C3'], body: { ...ride, cityCode: '' }, status: 400, code: 'VALIDATION_ERROR', field: 'cityCode' },
  { id: 'EP-08', covers: ['V3'], body: { ...ride, vehicle: 'truck' }, status: 400, code: 'VALIDATION_ERROR', field: 'vehicle' },
];

/** EP coverage = số phân vùng được ít nhất một ca phủ ÷ tổng số phân vùng (CTFL v4.0.1 mục 4.2.1). */
export function epCoverage(partitions: Partition[], cases: Pick<ApiCase, 'covers'>[]) {
  const exercised = new Set(cases.flatMap((c) => c.covers));
  const hit = partitions.filter((p) => exercised.has(p.id));
  return {
    exercised: hit.length,
    total: partitions.length,
    percent: Math.round((hit.length / partitions.length) * 1000) / 10,
    missing: partitions.filter((p) => !exercised.has(p.id)).map((p) => p.id),
  };
}
