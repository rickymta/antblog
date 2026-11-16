/**
 * TCD-02 — chạy năm test case R-ACCEPT trong `tc-accept.ts` qua HTTP (supertest + createApp, kho trong bộ nhớ).
 * Mỗi `it` dựng đúng tiền điều kiện của tài liệu, thực hiện đúng các bước, rồi so mọi kết quả mong đợi.
 * Test data đọc từ chính tài liệu test case — sửa tài liệu là sửa test.
 */
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';
import { GOOD } from './tc-accept';

const T0 = vnTime(2026, 11, 4, 9, 0); // 09:00:00 giờ VN = 2026-11-04T02:00:00.000Z
const T0_ISO = '2026-11-04T02:00:00.000Z';

let clock: FixedClock;
let system: LabSystem;
let app: ReturnType<typeof createApp>;
let T1: string;

const tc = (id: string) => GOOD.find((x) => x.id === id)!;
const td = (id: string) => tc(id).testData[0]!.values as { driver: string; etaMinutes: number };
const accept = (tripId: string, driver: string, etaMinutes: number) =>
  request(app).post(`/api/trips/${tripId}/accept`).set('x-user', `driver:${driver}`).send({ etaMinutes });
const driverStatus = async (id: string) => (await system.deps.drivers.findById(id))!.status;
const tripNow = async () => (await system.deps.trips.findById(T1))!;

/** Tiền điều kiện chung: reset, đồng hồ 09:00, T1 = chuyến SG xe máy 5 km của C001 ở `searching`. */
beforeEach(async () => {
  clock = new FixedClock(T0);
  system = createMemorySystem({ clock });
  await system.reset();
  app = createApp(system.deps, { logErrors: false });
  const quote = await request(app)
    .post('/api/quotes')
    .set('x-user', 'customer:C001')
    .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 })
    .expect(200);
  const trip = await request(app)
    .post('/api/trips')
    .set('x-user', 'customer:C001')
    .set('Idempotency-Key', 'tcd02-dat-chuyen-0001')
    .send({ quoteId: quote.body.quote.quoteId })
    .expect(201);
  T1 = trip.body.trip.id;
  expect(trip.body.trip.status).toBe('searching');
});

describe('TCD-02 · chạy test case R-ACCEPT', () => {
  it('TC-ACCEPT-001 · tài xế đủ điều kiện nhận được chuyến', async () => {
    const d = td('TC-ACCEPT-001');
    expect(await driverStatus(d.driver)).toBe('online'); // tiền điều kiện
    const res = await accept(T1, d.driver, d.etaMinutes); // bước 1
    expect(res.status).toBe(200);
    expect(res.body.trip).toMatchObject({ status: 'accepted', driverId: 'D101', etaMinutes: 7, acceptedAt: T0_ISO });
    expect(await driverStatus('D101')).toBe('busy'); // bước 2
  });

  it('TC-ACCEPT-002 · tài xế offline bị từ chối, chuyến không đổi', async () => {
    const d = td('TC-ACCEPT-002');
    expect(await driverStatus(d.driver)).toBe('offline');
    const res = await accept(T1, d.driver, d.etaMinutes);
    expect(res.status).toBe(409);
    expect(res.body.error.code).toBe('DRIVER_OFFLINE');
    expect(await tripNow()).toMatchObject({ status: 'searching', driverId: null, acceptedAt: null });
    expect(await driverStatus('D104')).toBe('offline');
  });

  it('TC-ACCEPT-003 · khác cả thành phố lẫn loại xe ⇒ CITY_MISMATCH được báo trước', async () => {
    const d = td('TC-ACCEPT-003');
    const res = await accept(T1, d.driver, d.etaMinutes);
    expect(res.status).toBe(422);
    expect(res.body.error.code).toBe('CITY_MISMATCH');
    expect(await tripNow()).toMatchObject({ status: 'searching', driverId: null });
  });

  it('TC-ACCEPT-004 · tài xế thứ hai nhận chuyến đã có người nhận ⇒ TRIP_ALREADY_TAKEN', async () => {
    await accept(T1, 'D101', 7).expect(200); // tiền điều kiện: D101 đã nhận T1, dựng trực tiếp — không gọi test case khác
    const d = td('TC-ACCEPT-004');
    expect(await driverStatus(d.driver)).toBe('online');
    const res = await accept(T1, d.driver, d.etaMinutes);
    expect(res.status).toBe(409);
    expect(res.body.error.code).toBe('TRIP_ALREADY_TAKEN');
    expect(await tripNow()).toMatchObject({ status: 'accepted', driverId: 'D101', etaMinutes: 7 });
    expect(await driverStatus('D102')).toBe('online');
  });

  it('TC-ACCEPT-005 · gửi lại lệnh nhận cho chuyến đang giữ ⇒ 200, giữ ETA và acceptedAt cũ', async () => {
    await accept(T1, 'D101', 7).expect(200); // tiền điều kiện
    clock.advanceMinutes(1); // 09:01:00
    const d = td('TC-ACCEPT-005');
    const res = await accept(T1, d.driver, d.etaMinutes);
    expect(res.status).toBe(200);
    expect(res.body.trip).toMatchObject({ status: 'accepted', driverId: 'D101', etaMinutes: 7, acceptedAt: T0_ISO });
  });
});
