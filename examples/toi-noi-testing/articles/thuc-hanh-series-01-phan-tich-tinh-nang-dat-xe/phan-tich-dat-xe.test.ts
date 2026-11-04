/**
 * TEST-S01-LAB — kiểm chứng các «sự thật» mà lời giải mẫu của bài thực hành dựa vào.
 * Mỗi `it` mang mã test condition (TCD-xx) của lời giải mẫu và mã yêu cầu SPEC (R-…).
 * Test chạy ở mức hệ thống của lab: gọi API HTTP qua supertest, kho trong bộ nhớ, đồng hồ giả.
 * Riêng «một người thắng» chạy thêm trên PGlite (PostgreSQL WASM, một kết nối).
 */
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import type { Actor } from '../../src/app/deps';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, createTrip, expireStaleSearches } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, createPgliteSystem, type LabSystem } from '../../src/infra/compose';

const C001 = 'customer:C001';
const ride = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 };

describe('TEST-S01-LAB · luồng đặt xe từ báo giá tới hoàn thành chuyến', () => {
  let clock: FixedClock;
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;

  beforeEach(() => {
    clock = new FixedClock(vnTime(2026, 11, 4, 9, 0)); // 09:00 giờ VN — ngoài cao điểm
    system = createMemorySystem({ clock, config: { payment: { timeoutMs: 50, maxRetries: 2 } } });
    app = createApp(system.deps, { logErrors: false });
  });

  const quote = (body: object = ride) => request(app).post('/api/quotes').set('x-user', C001).send(body);
  const book = (quoteId: string, key: string, extra: object = {}) =>
    request(app).post('/api/trips').set('x-user', C001).set('Idempotency-Key', key).send({ quoteId, ...extra });
  const driver = (id: string, tripId: string, step: string, body: object = {}) =>
    request(app).post(`/api/trips/${tripId}/${step}`).set('x-user', `driver:${id}`).send(body);

  it('TCD-01 · R-STATE-02/06, R-PAY-03: đi hết luồng chính, nhật ký đủ 7 trạng thái, thu VíXanh một lần', async () => {
    const q = (await quote().expect(200)).body.quote;
    const trip = (await book(q.quoteId, 'lab-key-0001', { paymentMethod: 'vixanh' }).expect(201)).body.trip;
    expect(trip).toMatchObject({ status: 'searching', fare: 46_000, paymentStatus: 'pending' });

    await driver('D101', trip.id, 'accept', { etaMinutes: 5 }).expect(200);
    for (const step of ['arrive', 'pickup', 'start']) await driver('D101', trip.id, step).expect(200);
    const done = (await driver('D101', trip.id, 'complete').expect(200)).body.trip;

    expect(done).toMatchObject({ status: 'completed', paymentStatus: 'paid' });
    const events = await system.deps.trips.listEvents(trip.id);
    expect(events.map((e) => e.status)).toEqual(['requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed']);
    expect(system.payments.chargedKeys()).toEqual([`trip:${trip.id}:fare`]);
    expect(system.payments.totalCharged()).toBe(46_000);
    expect((await system.deps.drivers.findById('D101'))?.status).toBe('online');
  });

  it('TCD-02 · R-QUOTE-01: còn hạn ở 4 phút 59,999 giây; đúng 5 phút ⇒ 422 QUOTE_EXPIRED', async () => {
    const q1 = (await quote().expect(200)).body.quote;
    clock.advanceMs(5 * 60_000 - 1);
    await book(q1.quoteId, 'lab-key-0002').expect(201);

    const q2 = (await quote().expect(200)).body.quote;
    clock.advanceMinutes(5);
    const res = await book(q2.quoteId, 'lab-key-0003');
    expect(res.status).toBe(422);
    expect(res.body.error.code).toBe('QUOTE_EXPIRED');
  });

  it('TCD-03 · R-QUOTE-04: giá của chuyến là ảnh chụp lúc báo giá, cao điểm và mưa đã hết vẫn không đổi', async () => {
    clock.set(vnTime(2026, 11, 4, 12, 58)); // cao điểm
    system.weather.setRaining(['SG']);
    const q = (await quote({ ...ride, voucherCode: 'GIAM20' }).expect(200)).body.quote;
    expect(q).toMatchObject({ total: 52_000, peak: true, raining: true }); // oracle R-FARE-09

    clock.set(vnTime(2026, 11, 4, 13, 1)); // hết cao điểm, báo giá còn hạn
    system.weather.setRaining([]);
    const trip = (await book(q.quoteId, 'lab-key-0004').expect(201)).body.trip;
    expect(trip).toMatchObject({ fare: 52_000, peak: true, raining: true, pricingVersion: 'fare-2026.1' });

    const fresh = (await request(app).post('/api/quotes').set('x-user', 'customer:C002').send({ ...ride, voucherCode: 'GIAM20' }).expect(200)).body.quote;
    expect(fresh).toMatchObject({ total: 37_000, peak: false, raining: false }); // cùng chuyến đi, báo giá mới
  });

  it('TCD-04 · R-IDEM-02 + R-QUOTE-03: gửi lại cùng khoá trả đúng chuyến cũ; khoá mới cho cùng báo giá ⇒ 409', async () => {
    const q = (await quote().expect(200)).body.quote;
    const first = await book(q.quoteId, 'lab-key-0005').expect(201);
    const replay = await book(q.quoteId, 'lab-key-0005').expect(201);
    expect(replay.headers['idempotent-replayed']).toBe('true');
    expect(replay.body.trip.id).toBe(first.body.trip.id);

    const other = await book(q.quoteId, 'lab-key-0006');
    expect(other.status).toBe(409);
    expect(other.body.error.code).toBe('QUOTE_ALREADY_USED');
  });

  it('TCD-05 · R-ACCEPT-03: D101 và D102 cùng bấm nhận ⇒ đúng một người thắng, người thua vẫn online', async () => {
    const q = (await quote().expect(200)).body.quote;
    const trip = (await book(q.quoteId, 'lab-key-0007').expect(201)).body.trip;

    const [a, b] = await Promise.all([
      driver('D101', trip.id, 'accept', { etaMinutes: 4 }),
      driver('D102', trip.id, 'accept', { etaMinutes: 6 }),
    ]);
    expect([a.status, b.status].sort()).toEqual([200, 409]);
    const loser = a.status === 409 ? a : b;
    const loserId = a.status === 409 ? 'D101' : 'D102';
    const winnerId = loserId === 'D101' ? 'D102' : 'D101';
    expect(loser.body.error.code).toBe('TRIP_ALREADY_TAKEN');
    expect((await system.deps.drivers.findById(loserId))?.status).toBe('online');
    expect((await system.deps.drivers.findById(winnerId))?.status).toBe('busy');
    expect((await system.deps.trips.findById(trip.id))?.driverId).toBe(winnerId);
  });

  it('TCD-06 · R-STATE-05: 4 phút 59 giây chưa huỷ; đủ 5 phút ⇒ hệ thống huỷ no_driver, phí 0 đ', async () => {
    const q = (await quote().expect(200)).body.quote;
    const trip = (await book(q.quoteId, 'lab-key-0008').expect(201)).body.trip;

    clock.advanceMs(5 * 60_000 - 1_000);
    expect(await expireStaleSearches(system.deps)).toBe(0);
    clock.advanceMs(1_000);
    expect(await expireStaleSearches(system.deps)).toBe(1);
    expect(await system.deps.trips.findById(trip.id)).toMatchObject({
      status: 'cancelled', cancelledBy: 'system', cancelReason: 'no_driver', cancellationFee: 0, paymentStatus: 'none',
    });
  });

  it('TCD-07 · R-PAY-06: VíXanh từ chối ⇒ chuyến vẫn completed, paymentStatus = failed, API 200', async () => {
    const q = (await quote().expect(200)).body.quote;
    const trip = (await book(q.quoteId, 'lab-key-0009', { paymentMethod: 'vixanh' }).expect(201)).body.trip;
    await driver('D101', trip.id, 'accept', { etaMinutes: 5 }).expect(200);
    for (const step of ['arrive', 'pickup', 'start']) await driver('D101', trip.id, step).expect(200);

    system.payments.setMode('decline');
    const done = await driver('D101', trip.id, 'complete');
    expect(done.status).toBe(200);
    expect(done.body.trip).toMatchObject({ status: 'completed', paymentStatus: 'failed' });
    expect(system.payments.calls).toHaveLength(1); // bị từ chối ⇒ không thử lại (R-PAY-04)
    expect(system.payments.totalCharged()).toBe(0);
  });

  it('TCD-08 · R-CANCEL-01/02: khách huỷ lúc 1 phút 59 giây ⇒ 0 đ; đúng 2 phút ⇒ 10.000 đ', async () => {
    const cancel = (tripId: string, key: string) =>
      request(app).post(`/api/trips/${tripId}/cancel`).set('x-user', C001).set('Idempotency-Key', key).send({});

    const t1 = (await book((await quote().expect(200)).body.quote.quoteId, 'lab-key-0010').expect(201)).body.trip;
    await driver('D101', t1.id, 'accept', { etaMinutes: 5 }).expect(200);
    clock.advanceMs(2 * 60_000 - 1_000);
    expect((await cancel(t1.id, 'lab-cancel-0001').expect(200)).body).toMatchObject({ cancellationFee: 0, feeReason: 'grace_period' });

    const t2 = (await book((await quote().expect(200)).body.quote.quoteId, 'lab-key-0011').expect(201)).body.trip;
    await driver('D101', t2.id, 'accept', { etaMinutes: 5 }).expect(200);
    clock.advanceMinutes(2);
    expect((await cancel(t2.id, 'lab-cancel-0002').expect(200)).body).toMatchObject({ cancellationFee: 10_000, feeReason: 'charged' });
  });

  it('Bài tập 2 · TCD-11 · R-STATE-03, R-API-10: hoàn thành khi mới accepted ⇒ 409, chuyến giữ nguyên; tài xế lạ ⇒ 404', async () => {
    const q = (await quote().expect(200)).body.quote;
    const trip = (await book(q.quoteId, 'lab-key-0012').expect(201)).body.trip;
    await driver('D101', trip.id, 'accept', { etaMinutes: 5 }).expect(200);

    const skip = await driver('D101', trip.id, 'complete');
    expect(skip.status).toBe(409);
    expect(skip.body.error.code).toBe('INVALID_TRANSITION');
    expect((await system.deps.trips.findById(trip.id))?.status).toBe('accepted');
    expect(await system.deps.trips.listEvents(trip.id)).toHaveLength(3); // requested, searching, accepted

    const stranger = await driver('D102', trip.id, 'arrive');
    expect(stranger.status).toBe(404);
    expect(stranger.body.error.code).toBe('TRIP_NOT_FOUND');
  });

  it('Ngoài phạm vi lab · R-GEN-06/R-API-14: gửi «điểm đón» bị bỏ qua — lab chỉ nhận quãng đường', async () => {
    const q = (await quote({ ...ride, pickup: '12 Lý Tự Trọng, Quận 1' }).expect(200)).body.quote;
    expect(q).not.toHaveProperty('pickup');
    expect(q.distanceKm).toBe(7.4);
  });
});

describe('TEST-S01-LAB · TCD-05 trên PGlite (PostgreSQL WASM, một kết nối)', () => {
  it('R-ACCEPT-03: hai lệnh nhận xen kẽ ở mức await ⇒ đúng một thành công', async () => {
    const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
    const system = await createPgliteSystem({ clock });
    const customer: Actor = { role: 'customer', id: 'C001' };
    const q = await createQuote(system.deps, customer, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 });
    const trip = await createTrip(system.deps, customer, { quoteId: q.id });

    const results = await Promise.allSettled([
      acceptTrip(system.deps, { role: 'driver', id: 'D101' }, trip.id, 4),
      acceptTrip(system.deps, { role: 'driver', id: 'D102' }, trip.id, 6),
    ]);
    const ok = results.filter((r) => r.status === 'fulfilled');
    const failed = results.filter((r): r is PromiseRejectedResult => r.status === 'rejected');
    expect(ok).toHaveLength(1);
    expect(failed).toHaveLength(1);
    expect(failed[0]?.reason).toMatchObject({ code: 'TRIP_ALREADY_TAKEN' });
    const statuses = [(await system.deps.drivers.findById('D101'))?.status, (await system.deps.drivers.findById('D102'))?.status].sort();
    expect(statuses).toEqual(['busy', 'online']);
  });
});
