/**
 * TEST-040 · Domain testing cho miền nhiều biến (d, w, v) của báo giá bưu kiện — CTAL-TA v4.0 mục 3.1.1.
 * Cùng một oracle chạy trên: bộ điểm ON/OFF(/IN/OUT) ở mức component, cùng bộ đó qua API, và bảng 10.000 dòng.
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { expectedOutcome, RAW, domainPoints } from './mien-buu-kien';
import { failures, killMatrix, loadTable, SUITES, tableHits } from './phan-tich';
import { makeImpl, srcImpl } from './trien-khai';

const table = loadTable();

describe('TEST-040 · mã thật của lab trên từng bộ điểm (component)', () => {
  it.each(Object.entries(SUITES))('%s: mọi điểm khớp oracle', (_name, points) => {
    expect(failures(points, srcImpl)).toEqual([]);
  });

  it('bảng 10.000 dòng: mọi dòng khớp oracle', () => {
    expect(table).toHaveLength(10_000);
    expect(failures(table, srcImpl)).toEqual([]);
  });

  it('bản chép tham chiếu (makeImpl) khớp src trên mọi điểm và mọi dòng — mutant đáng tin', () => {
    const all = [...Object.values(SUITES).flat(), ...table];
    const ref = makeImpl();
    expect(all.filter((p) => JSON.stringify(ref(p.d, p.w, p.v)) !== JSON.stringify(srcImpl(p.d, p.w, p.v)))).toEqual([]);
  });
});

describe('TEST-040 · cùng bộ simplified 0,1 qua API POST /api/quotes (system)', () => {
  const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
  const app = createApp(system.deps, { logErrors: false });
  it.each(SUITES['Simplified 0,1'])('$kind · d=$d w=$w $v', async ({ d, w, v }) => {
    const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
      .send({ kind: 'parcel', cityCode: 'SG', vehicle: v, distanceKm: d, parcelWeightKg: w });
    const want = expectedOutcome(d, w, v);
    if (want.code === 'OK') expect([res.status, res.body.quote.fare.parcelSurcharge]).toEqual([200, want.surcharge]);
    else expect([res.status, res.body.error.code]).toEqual([422, want.code]);
  });
});

describe('TEST-040 · bộ nào bắt được defect nào (số điểm làm mutant lộ)', () => {
  it('ma trận diệt mutant', () => {
    const rows = Object.fromEntries(killMatrix(table).map(({ id, moTa: _m, ...rest }) => [id, Object.values(rest)]));
    //            simplified 0,1 · reliable 0,1 · simplified thô 0,01 · bảng 10.000
    expect(rows).toEqual({
      M1: [4, 4, 4, 0], M2: [4, 4, 4, 0], M3: [2, 2, 2, 39], M4: [2, 2, 2, 0], M5: [1, 1, 1, 0], M6: [3, 5, 3, 0],
      M7: [1, 1, 1, 881], M8: [0, 0, 14, 189], M9: [0, 0, 3, 143], M10: [0, 2, 0, 0], M11: [0, 0, 0, 0],
    });
  });

  it('bảng 10.000 dòng chỉ chạm 2/28 coverage item ON/OFF mức thô', () => {
    const hits = tableHits(table);
    expect(hits).toHaveLength(domainPoints(RAW, 0.01, false).length);
    expect(hits.filter((h) => h.hits > 0).map((h) => h.item)).toEqual([
      'ON  MIEN_PHI_xe_may w<5.05 ⇒ w=5.04',
      'OFF MIEN_PHI_xe_may w<5.05 ⇒ w=5.05',
    ]);
  });

  it('M11 (sai thứ tự kiểm) sống sót mọi bộ: cần điểm sai ở CẢ HAI biến', () => {
    const corner = { d: 0.4, w: 0, v: 'bike' as const };
    expect(expectedOutcome(corner.d, corner.w, corner.v).code).toBe('DISTANCE_OUT_OF_RANGE');
    expect(makeImpl({ weightFirst: true })(corner.d, corner.w, corner.v).code).toBe('WEIGHT_OUT_OF_RANGE');
  });
});
