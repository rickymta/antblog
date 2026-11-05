/**
 * TEST-032 · Vài ca biên ở mức system qua POST /api/quotes: biên có còn đúng sau khi đi qua zod, quy tắc đầu vào, tính cước?
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';

function appAt(h: number, m: number, s: number) {
  const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, h, m, s)) });
  return createApp(system.deps, { logErrors: false });
}

const at0900 = appAt(9, 0, 0);
const quote = (app: ReturnType<typeof appAt>, body: object) =>
  request(app).post('/api/quotes').set('x-user', 'customer:C001').send(body);

describe('Biên quãng đường (ride, xe máy, SG, 09:00)', () => {
  it.each([
    { km: 0.44, status: 422, total: undefined },
    { km: 0.45, status: 200, total: 15_000 }, // ⇒ 0,5 km: 14.250 ⇒ cước tối thiểu 15.000
    { km: 100.04, status: 200, total: 462_000 }, // ⇒ 100,0 km: 12.000 + 450.000
    { km: 100.05, status: 422, total: undefined },
  ])('BVA-API $km km ⇒ $status', async ({ km, status, total }) => {
    const res = await quote(at0900, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: km }).expect(status);
    if (total !== undefined) expect(res.body.quote.total).toBe(total);
    else expect(res.body.error.code).toBe('DISTANCE_OUT_OF_RANGE');
  });
});

describe('Biên khối lượng và ngưỡng xe máy (parcel, 3 km, SG, 09:00)', () => {
  it.each([
    { kg: 0.04, vehicle: 'car', status: 422, code: 'WEIGHT_OUT_OF_RANGE' },
    { kg: 0.05, vehicle: 'car', status: 200, code: undefined },
    { kg: 20.04, vehicle: 'bike', status: 200, code: undefined }, // ⇒ 20,0 kg vẫn xe máy
    { kg: 20.05, vehicle: 'bike', status: 422, code: 'VEHICLE_TOO_SMALL' }, // ⇒ 20,1 kg
    { kg: 30.04, vehicle: 'car', status: 200, code: undefined },
    { kg: 30.05, vehicle: 'car', status: 422, code: 'WEIGHT_OUT_OF_RANGE' },
  ])('BVA-API $kg kg $vehicle ⇒ $status', async ({ kg, vehicle, status, code }) => {
    const res = await quote(at0900, { kind: 'parcel', cityCode: 'SG', vehicle, distanceKm: 3, parcelWeightKg: kg }).expect(status);
    if (code) expect(res.body.error.code).toBe(code);
  });
});

describe('Biên giờ cao điểm qua báo giá (ride, xe máy 7,4 km, SG)', () => {
  it.each([
    { time: [10, 59, 59], peak: false, total: 46_000 },
    { time: [11, 0, 0], peak: true, total: 57_000 }, // 45.300 × 1,25 = 56.625 ⇒ 57.000
    { time: [12, 59, 59], peak: true, total: 57_000 },
    { time: [13, 0, 0], peak: false, total: 46_000 },
  ] as const)('BVA-API $time ⇒ peak $peak, total $total', async ({ time, peak, total }) => {
    const res = await quote(appAt(...time), { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 }).expect(200);
    expect(res.body.quote.peak).toBe(peak);
    expect(res.body.quote.total).toBe(total);
  });
});
