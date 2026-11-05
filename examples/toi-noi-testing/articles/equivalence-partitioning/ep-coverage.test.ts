/**
 * TEST-031 · Đo EP coverage (CTFL v4.0.1 mục 4.2.1) của hai bộ ca trên cùng mô hình 10 phân vùng.
 */
import { describe, expect, it } from 'vitest';
import { API_CASES, PARTITIONS, epCoverage } from './ep-model';

describe('EP coverage', () => {
  it('Bộ Each Choice (8 ca EP-01…08) phủ 10/10 phân vùng = 100%', () => {
    const result = epCoverage(PARTITIONS, API_CASES);
    expect(result).toEqual({ exercised: 10, total: 10, percent: 100, missing: [] });
  });

  it('Bộ «thường gặp» (1 ca đúng + 2 ca sai quãng đường) chỉ phủ 5/10 = 50%', () => {
    const naive = API_CASES.filter((c) => ['EP-01', 'EP-03', 'EP-04'].includes(c.id));
    const result = epCoverage(PARTITIONS, naive);
    expect(result.percent).toBe(50);
    expect(result.missing).toEqual(['D4', 'C2', 'C3', 'V2', 'V3']);
  });

  it('Gộp hai phân vùng không hợp lệ vào một ca KHÔNG được tính phủ cả hai (xem ca EP-X)', () => {
    // Ca EP-X chỉ thực sự phủ C2 — khai báo trung thực thì coverage không bị thổi phồng
    const honest = epCoverage(PARTITIONS, [{ covers: ['C2'] }]);
    const inflated = epCoverage(PARTITIONS, [{ covers: ['C2', 'D1'] }]);
    expect([honest.exercised, inflated.exercised]).toEqual([1, 2]);
  });
});
