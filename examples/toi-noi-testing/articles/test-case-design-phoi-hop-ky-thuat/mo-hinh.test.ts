/**
 * TCD-06 · Kiểm chính các mô hình: oracle, độ phủ của từng kỹ thuật, rút gọn tham lam giữ nguyên độ phủ.
 */
import { describe, expect, it } from 'vitest';
import {
  BVA_SUITE, DT_SUITE, EP_SUITE, MERGED_POSITIVES, POOL, TECHNIQUE_SUITE, coverage, coverageItems, d,
  expectedOutcome, greedyMinimize, tenthsHalfUp, type QuoteCase,
} from './mo-hinh';

describe('Oracle suy từ SPEC (tính tay để đối chiếu)', () => {
  it.each([
    ['0.44', 4], ['0.45', 5], ['100.04', 1000], ['100.05', 1001], ['0.04', 0], ['0.05', 1],
    ['5.04', 50], ['5.05', 51], ['20.05', 201], ['-1', -10], ['7.4', 74],
  ] as const)('làm tròn half-up %s ⇒ %i phần mười', (dec, tenths) => {
    expect(tenthsHalfUp(dec)).toBe(tenths);
  });

  it('năm tổng tiền tính tay khớp oracle', () => {
    const byId = (id: string) => POOL.find((c) => c.id === id)!;
    expect(expectedOutcome(byId('EP-01'))).toEqual({ status: 200, total: 46_000 }); // 12.000 + 4.500 × 7,4 = 45.300
    expect(expectedOutcome(byId('BVA-09'))).toEqual({ status: 200, total: 76_000 }); // 45.300 + 2.000 × 15 = 75.300
    expect(expectedOutcome(byId('BVA-11'))).toEqual({ status: 200, total: 107_000 }); // 25.000 + 81.400 = 106.400
    expect(expectedOutcome(byId('BVA-12'))).toEqual({ status: 200, total: 109_000 }); // 106.400 + 2.000
    const gh = MERGED_POSITIVES.find((c) => c.note === 'parcel, car, 100.04 km, 30.04 kg')!;
    expect(expectedOutcome(gh)).toEqual({ status: 200, total: 1_175_000 }); // 25.000 + 1.100.000 + 50.000
  });
});

describe('Độ phủ của từng bộ ca theo từng kỹ thuật (EP 20 phân vùng · BVA 12 giá trị · DT 6 cột)', () => {
  it('bộ EP (13 ca): EP 20/20, BVA 0/12, DT 3/6', () => {
    expect(EP_SUITE).toHaveLength(13);
    expect(coverage(EP_SUITE)).toEqual({ ep: [20, 20], bva: [0, 12], dt: [3, 6], all: [23, 38] });
  });
  it('bộ BVA (12 ca): EP 14/20, BVA 12/12, DT 3/6', () => {
    expect(BVA_SUITE).toHaveLength(12);
    expect(coverage(BVA_SUITE)).toEqual({ ep: [14, 20], bva: [12, 12], dt: [3, 6], all: [29, 38] });
  });
  it('bộ DT (6 ca): EP 10/20, BVA 0/12, DT 6/6', () => {
    expect(DT_SUITE).toHaveLength(6);
    expect(coverage(DT_SUITE)).toEqual({ ep: [10, 20], bva: [0, 12], dt: [6, 6], all: [16, 38] });
  });
  it('gộp ba bộ (31 ca) phủ đủ 38/38 item', () => {
    expect(TECHNIQUE_SUITE).toHaveLength(31);
    expect(coverage(TECHNIQUE_SUITE).all).toEqual([38, 38]);
  });
});

describe('Rút gọn tham lam', () => {
  const final = greedyMinimize(POOL);

  it('kho 73 ca (31 ca kỹ thuật + 42 ca dương ghép) ⇒ 18 ca, vẫn 38/38', () => {
    expect([MERGED_POSITIVES.length, POOL.length]).toEqual([42, 73]);
    expect(final).toHaveLength(18);
    expect(coverage(final)).toEqual({ ep: [20, 20], bva: [12, 12], dt: [6, 6], all: [38, 38] });
  });

  it('không ca nào trong bộ cuối là thừa: bỏ bất kỳ ca nào cũng mất ít nhất một item', () => {
    for (const c of final) expect(coverage(final.filter((x) => x !== c)).all[0]).toBeLessThan(38);
  });

  it('bộ cuối có 6 ca dương và 12 ca âm; mỗi ca âm phủ tối đa một phân vùng EP (single fault)', () => {
    const neg = final.filter((c) => expectedOutcome(c).status !== 200);
    expect([final.length - neg.length, neg.length]).toEqual([6, 12]);
    for (const c of neg) expect(coverageItems(c).filter((i) => i.startsWith('EP:')).length).toBeLessThanOrEqual(1);
  });

  it('không có kho ca ghép, rút gọn chỉ còn 19 ca', () => {
    expect(greedyMinimize(TECHNIQUE_SUITE)).toHaveLength(19);
  });

  it('đặt 4 ca biên quãng đường lên nền bưu kiện thay vì đặt xe ⇒ còn 17 ca', () => {
    const parcelBase = (c: QuoteCase): QuoteCase =>
      c.source === 'BVA' && c.note.startsWith('QD-') ? { ...c, kind: 'parcel', vehicle: 'car', parcelWeightKg: d('3') } : c;
    const alt = greedyMinimize(POOL.map(parcelBase));
    expect(alt).toHaveLength(17);
    expect(coverage(alt).all).toEqual([38, 38]);
  });
});
