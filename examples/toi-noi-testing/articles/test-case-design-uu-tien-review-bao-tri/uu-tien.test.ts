/**
 * TCD-09 · Kiểm các chiến lược ưu tiên của CTFL 5.1.5 trên bộ 15 test case R-PAY + R-IDEM.
 * Mọi thứ tự in trong bài viết mục 4.2 được khẳng định ở đây.
 * Chạy: npx vitest run articles/test-case-design-uu-tien-review-bao-tri --reporter=verbose
 */
import { describe, expect, it } from 'vitest';
import { CONDITIONS, REQUIREMENT_PRIORITY, RISKS, TEST_CASES } from './bo-test-case';
import {
  additionalCoverage,
  cumulativeCoverage,
  requirementsBased,
  respectDependencies,
  riskBased,
  riskExposureCovered,
  risksTouched,
  totalCoverage,
  violations,
} from './uu-tien';

const short = (order: string[]) => order.map((id) => id.replace('TC-', ''));

describe('Dữ liệu đầu vào', () => {
  it('15 test case, 19 test condition; 18 condition có ca phủ, TCOND-IDEM-08 chưa có', () => {
    const covered = new Set(TEST_CASES.flatMap((t) => t.covers));
    expect(TEST_CASES).toHaveLength(15);
    expect(CONDITIONS).toHaveLength(19);
    expect(covered.size).toBe(18);
    expect(CONDITIONS.filter((c) => !covered.has(c.id)).map((c) => c.id)).toEqual(['TCOND-IDEM-08']);
  });

  it('Sổ rủi ro: tổng mức rủi ro 30 (6 + 6 + 4 + 4 + 3 + 3 + 2 + 2)', () => {
    expect(RISKS.map((r) => r.likelihood * r.impact)).toEqual([6, 6, 4, 4, 3, 3, 2, 2]);
  });
});

describe('Risk-based (CTFL 5.1.5)', () => {
  const raw = riskBased(TEST_CASES, RISKS);

  it('thứ tự thuần theo mức rủi ro vi phạm phụ thuộc: IDEM-007 đứng trước PAY-006', () => {
    expect(short(raw)).toEqual([
      'IDEM-002', 'IDEM-007', 'PAY-002', 'PAY-005', 'IDEM-003', 'IDEM-004', 'PAY-003', 'PAY-004',
      'IDEM-006', 'PAY-001', 'PAY-006', 'IDEM-001', 'IDEM-005', 'PAY-007', 'PAY-008',
    ]);
    expect(violations(raw, TEST_CASES)).toEqual(['TC-IDEM-007 chạy trước TC-PAY-006']);
  });

  it('sau khi sửa theo phụ thuộc: PAY-006 (mức 3) được kéo lên ngay trước IDEM-007 (mức 6)', () => {
    const order = respectDependencies(raw, TEST_CASES);
    expect(short(order)).toEqual([
      'IDEM-002', 'PAY-006', 'IDEM-007', 'PAY-002', 'PAY-005', 'IDEM-003', 'IDEM-004', 'PAY-003',
      'PAY-004', 'IDEM-006', 'PAY-001', 'IDEM-001', 'IDEM-005', 'PAY-007', 'PAY-008',
    ]);
    expect(violations(order, TEST_CASES)).toEqual([]);
  });

  it('5 ca đầu chỉ chạm 3 rủi ro (RR-01, RR-02, RR-05) — mức rủi ro được chạm 15/30', () => {
    const order = respectDependencies(raw, TEST_CASES);
    expect(risksTouched(order, TEST_CASES, 5)).toEqual(['RR-01', 'RR-02', 'RR-05']);
    expect(riskExposureCovered(order, TEST_CASES, RISKS, 5)).toBe(15);
  });
});

describe('Requirements-based (CTFL 5.1.5)', () => {
  it('ưu tiên yêu cầu 1 → 2 → 3, hoà thì theo mã; sửa phụ thuộc như trên', () => {
    const order = respectDependencies(requirementsBased(TEST_CASES, REQUIREMENT_PRIORITY), TEST_CASES);
    expect(short(order)).toEqual([
      'IDEM-002', 'PAY-006', 'IDEM-007', 'PAY-002', 'PAY-003', 'PAY-004', 'PAY-005', 'IDEM-003',
      'IDEM-004', 'PAY-001', 'IDEM-001', 'IDEM-005', 'IDEM-006', 'PAY-007', 'PAY-008',
    ]);
    expect(risksTouched(order, TEST_CASES, 5)).toEqual(['RR-01', 'RR-02', 'RR-03', 'RR-05']);
    expect(riskExposureCovered(order, TEST_CASES, RISKS, 5)).toBe(19);
  });
});

describe('Coverage-based (CTFL 5.1.5)', () => {
  it('tổng coverage: PAY-003 (3 item) đầu tiên; đường phủ tích luỹ có một bậc phẳng ở PAY-002', () => {
    const order = respectDependencies(totalCoverage(TEST_CASES), TEST_CASES);
    expect(short(order).slice(0, 5)).toEqual(['PAY-003', 'IDEM-002', 'PAY-006', 'IDEM-007', 'PAY-004']);
    expect(cumulativeCoverage(order, TEST_CASES)).toEqual([3, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 16, 17, 18]);
    expect(riskExposureCovered(order, TEST_CASES, RISKS, 5)).toBe(19);
  });

  it('additional coverage: phần phủ thêm 3, 2, 2, rồi 1…; PAY-002 phủ thêm 0 nên xuống cuối', () => {
    const { order, gains } = additionalCoverage(TEST_CASES);
    expect(gains).toEqual([3, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0]);
    expect(short(order)).toEqual([
      'PAY-003', 'IDEM-002', 'PAY-005', 'IDEM-001', 'IDEM-003', 'IDEM-004', 'IDEM-005', 'IDEM-006',
      'IDEM-007', 'PAY-001', 'PAY-004', 'PAY-006', 'PAY-007', 'PAY-008', 'PAY-002',
    ]);
    const fixed = respectDependencies(order, TEST_CASES);
    expect(short(fixed).slice(7, 10)).toEqual(['IDEM-006', 'PAY-006', 'IDEM-007']);
    expect(cumulativeCoverage(fixed, TEST_CASES)).toEqual([3, 5, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 18]);
    expect(risksTouched(fixed, TEST_CASES, 5)).toEqual(['RR-01', 'RR-02', 'RR-03', 'RR-04', 'RR-08']);
    expect(riskExposureCovered(fixed, TEST_CASES, RISKS, 5)).toBe(22);
  });
});

describe('Phụ thuộc', () => {
  it('vòng phụ thuộc bị phát hiện, không lặng lẽ sinh lịch sai', () => {
    const cyclic = TEST_CASES.map((t) => (t.id === 'TC-PAY-006' ? { ...t, dependsOn: ['TC-IDEM-007'] } : t));
    expect(() => respectDependencies(riskBased(cyclic, RISKS), cyclic)).toThrow(/Vòng phụ thuộc/);
  });
});
