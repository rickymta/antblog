/**
 * Automated test script cho TC-PARCEL-001…005, chạy trên bộ dữ liệu TD-PARCEL-01.
 * Mặc định kiểm mã thật (src/domain/parcel.ts). Đặt TOINOI_PARCEL_IMPL=buggy để chạy
 * cùng bộ test trên bản cài đặt có lỗi (buggy-parcel.ts) và lấy test log làm bằng chứng cho DEF-0042.
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { calculateFare } from '../../src/domain/fare';
import { assertVehicleFits, normalizeWeightKg, parcelSurcharge } from '../../src/domain/parcel';
import { vnTime } from '../../src/domain/time';
import type { Vehicle } from '../../src/domain/types';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { buggyParcelSurcharge } from './buggy-parcel';
import data from './test-data/td-parcel-01.json';

const useBuggy = process.env.TOINOI_PARCEL_IMPL === 'buggy';
const surchargeOf = useBuggy ? buggyParcelSurcharge : parcelSurcharge;

type Row = (typeof data.rows)[number];

/** Ghép đúng các bước của báo giá bưu kiện (R-VALID-01, R-FARE-03 bước 1) với hàm phụ phí đang kiểm. */
function quoteParcel(row: Row) {
  const weight = normalizeWeightKg(row.weightKg); // R-PARCEL-01
  assertVehicleFits(weight, row.vehicle as Vehicle); // R-PARCEL-02
  const surcharge = surchargeOf(weight); // R-PARCEL-03
  const fare = calculateFare({
    vehicle: row.vehicle as Vehicle,
    distanceKm: data.context.distanceKm,
    parcelSurcharge: surcharge, // R-PARCEL-04: cộng vào cước cơ bản nên bị nhân cao điểm
    peak: row.peak,
    raining: data.context.raining,
  });
  return { surcharge, total: fare.total };
}

describe(`TD-PARCEL-01 · phụ phí bưu kiện (${useBuggy ? 'bản LỖI buggy-parcel.ts' : 'mã thật src/'})`, () => {
  const okRows = data.rows.filter((r) => 'total' in r.expected);
  const errorRows = data.rows.filter((r) => 'error' in r.expected);

  it.each(okRows)('$testCase / $id: $weightKg kg, $vehicle, cao điểm=$peak', (row) => {
    expect(quoteParcel(row)).toEqual(row.expected);
  });

  it.each(errorRows)('$testCase / $id: $weightKg kg, $vehicle ⇒ lỗi', (row) => {
    const code = (row.expected as { error: string }).error;
    expect(() => quoteParcel(row)).toThrow(expect.objectContaining({ code }));
  });
});

describe.skipIf(useBuggy)('TC-PARCEL-002 qua API (mã thật) — POST /api/quotes', () => {
  it('TD-04: 6,0 kg xe máy 3 km lúc 09:00 ⇒ phụ phí 2.000 đ, tổng 28.000 đ', async () => {
    const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
    const app = createApp(system.deps, { logErrors: false });
    const res = await request(app)
      .post('/api/quotes')
      .set('x-user', 'customer:C001')
      .send({ kind: 'parcel', cityCode: 'SG', vehicle: 'bike', distanceKm: 3, parcelWeightKg: 6.0 })
      .expect(200);
    expect(res.body.quote).toMatchObject({ parcelWeightKg: 6, peak: false, raining: false, total: 28_000 });
    expect(res.body.quote.fare.parcelSurcharge).toBe(2_000);
  });
});
