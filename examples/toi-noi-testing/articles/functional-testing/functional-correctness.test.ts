/**
 * TEST-017 · Functional correctness — cùng một test type (functional) ở hai test level:
 *  - component: hàm thuần parcelSurcharge (src/domain/parcel.ts)
 *  - system (qua API): POST /api/quotes với bưu kiện
 * Oracle: ví dụ trong SPEC R-PARCEL-03 và bảng giá R-FARE-01.
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { parcelSurcharge } from '../../src/domain/parcel';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';

// R-PARCEL-03: w ≤ 5 ⇒ 0; 5,1 ⇒ 2.000; 6,0 ⇒ 2.000; 6,1 ⇒ 4.000; 30 ⇒ 50.000
const ORACLE = [
  { id: 'FC-01', weightKg: 5.0, surcharge: 0, total: 58_000 },
  { id: 'FC-02', weightKg: 5.1, surcharge: 2_000, total: 60_000 },
  { id: 'FC-03', weightKg: 6.0, surcharge: 2_000, total: 60_000 },
  { id: 'FC-04', weightKg: 6.1, surcharge: 4_000, total: 62_000 },
  { id: 'FC-05', weightKg: 30, surcharge: 50_000, total: 108_000 },
];

function newApp(at = vnTime(2026, 11, 4, 9, 0), raining: ('SG' | 'HN')[] = []) {
  const system = createMemorySystem({ clock: new FixedClock(at), rainingCities: raining });
  return createApp(system.deps, { logErrors: false });
}

describe('Mức component · parcelSurcharge (R-PARCEL-03)', () => {
  it.each(ORACLE)('$id: $weightKg kg ⇒ phụ phí $surcharge đ', ({ weightKg, surcharge }) => {
    expect(parcelSurcharge(weightKg)).toBe(surcharge);
  });
});

describe('Mức system qua API · POST /api/quotes, ô tô 3 km, SG, 09:00, không mưa', () => {
  const app = newApp();
  // Cước cơ bản ô tô 3 km = 25.000 + 11.000 × 3 = 58.000 đ (R-FARE-01), cộng phụ phí rồi làm tròn lên 1.000 đ
  it.each(ORACLE)('$id: $weightKg kg ⇒ fare.parcelSurcharge $surcharge, total $total', async (row) => {
    const res = await request(app)
      .post('/api/quotes')
      .set('x-user', 'customer:C001')
      .send({ kind: 'parcel', cityCode: 'SG', vehicle: 'car', distanceKm: 3, parcelWeightKg: row.weightKg })
      .expect(200);
    expect(res.body.quote.fare.parcelSurcharge).toBe(row.surcharge);
    expect(res.body.quote.total).toBe(row.total);
  });

  it('FC-06 (đầu vào sai): 30,05 kg làm tròn thành 30,1 ⇒ 422 WEIGHT_OUT_OF_RANGE (R-PARCEL-01)', async () => {
    const res = await request(app)
      .post('/api/quotes')
      .set('x-user', 'customer:C001')
      .send({ kind: 'parcel', cityCode: 'SG', vehicle: 'car', distanceKm: 3, parcelWeightKg: 30.05 })
      .expect(422);
    expect(res.body.error.code).toBe('WEIGHT_OUT_OF_RANGE');
  });

  it('FC-07 (đầu vào sai): 20,1 kg đi xe máy ⇒ 422 VEHICLE_TOO_SMALL (R-PARCEL-02)', async () => {
    const res = await request(app)
      .post('/api/quotes')
      .set('x-user', 'customer:C001')
      .send({ kind: 'parcel', cityCode: 'SG', vehicle: 'bike', distanceKm: 3, parcelWeightKg: 20.1 })
      .expect(422);
    expect(res.body.error.code).toBe('VEHICLE_TOO_SMALL');
  });
});

describe('Tính nhất quán của bảng chi tiết (R-FARE-03, R-FARE-08, R-FARE-10)', () => {
  const peakRain = newApp(vnTime(2026, 11, 4, 11, 30), ['SG']);
  const cases = {
    'FC-08 R-FARE-09 (xe máy 7,4 km, cao điểm, mưa, GIAM20)': {
      body: { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4, voucherCode: 'GIAM20' },
      total: 52_000,
    },
    'FC-09 bưu kiện ô tô 3 km, 6,1 kg, cao điểm, mưa': {
      body: { kind: 'parcel', cityCode: 'SG', vehicle: 'car', distanceKm: 3, parcelWeightKg: 6.1 },
      total: 86_000, // (58.000 + 4.000) × 1,25 + 8.000 = 85.500 ⇒ 86.000
    },
    'FC-10 đồ ăn 2,5 km (phí giao), cao điểm, mưa': {
      body: { kind: 'food', cityCode: 'SG', distanceKm: 2.5, items: [{ name: 'Phở', unitPrice: 50000, quantity: 1 }] },
      total: 88_000, // phí giao (12.000 + 11.250) × 1,25 + 8.000 = 37.062,5 ⇒ 38.000; + 50.000 tiền món
    },
  } as const;

  it.each(Object.entries(cases))('%s: các bước khớp nhau và khớp tổng', async (_name, c) => {
    const res = await request(peakRain).post('/api/quotes').set('x-user', 'customer:C001').send(c.body).expect(200);
    const f = res.body.quote.fare;
    expect(f.base).toBeCloseTo(f.openingFee + f.distanceCharge + f.parcelSurcharge, 6);
    expect(f.beforeDiscount).toBeCloseTo(f.afterPeak + f.rainSurcharge, 6);
    expect(f.afterDiscount).toBeCloseTo(f.beforeDiscount - f.discount, 6);
    expect(f.total).toBe(Math.ceil(Math.max(f.afterDiscount, f.minimumFare) / 1000) * 1000);
    expect(res.body.quote.total).toBe(c.total);
  });
});
