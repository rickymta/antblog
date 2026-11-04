import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';

const C001 = 'customer:C001';
const C002 = 'customer:C002';
const D101 = 'driver:D101';
const ride = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 };

describe('smoke · API (supertest, kho trong bộ nhớ)', () => {
  let clock: FixedClock;
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;

  beforeEach(() => {
    clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
    system = createMemorySystem({ clock, config: { payment: { timeoutMs: 50, maxRetries: 2 } } });
    app = createApp(system.deps, { logErrors: false });
  });

  async function quoteId(body: object = ride, user = C001): Promise<string> {
    const res = await request(app).post('/api/quotes').set('x-user', user).send(body).expect(200);
    return res.body.quote.quoteId;
  }

  function book(id: string, key: string, user = C001, extra: object = {}) {
    return request(app).post('/api/trips').set('x-user', user).set('Idempotency-Key', key).send({ quoteId: id, ...extra });
  }

  it('R-FARE-09 qua API: 11:30 giờ VN, SG đang mưa, GIAM20 ⇒ tổng 52.000 đ, có quoteId + expiresAt', async () => {
    clock.set(vnTime(2026, 11, 4, 11, 30));
    system.weather.setRaining(['SG']);
    const res = await request(app).post('/api/quotes').set('x-user', C001).send({ ...ride, voucherCode: ' giam20 ' }).expect(200);
    expect(res.body.quote).toMatchObject({ total: 52_000, peak: true, raining: true, voucherCode: 'GIAM20', pricingVersion: 'fare-2026.1' });
    expect(new Date(res.body.quote.expiresAt).getTime() - clock.now().getTime()).toBe(5 * 60_000);
  });

  it('R-VALID-01: quãng đường ngoài khoảng ⇒ 422 DISTANCE_OUT_OF_RANGE; sai kiểu ⇒ 400', async () => {
    const out = await request(app).post('/api/quotes').set('x-user', C001).send({ ...ride, distanceKm: 100.05 });
    expect(out.status).toBe(422);
    expect(out.body.error.code).toBe('DISTANCE_OUT_OF_RANGE');
    const bad = await request(app).post('/api/quotes').set('x-user', C001).send({ ...ride, distanceKm: '7' });
    expect(bad.status).toBe(400);
    expect(bad.body.error.code).toBe('VALIDATION_ERROR');
  });

  it('R-IDEM-02/03: gửi lại cùng khoá trả đúng chuyến cũ; cùng khoá khác thân ⇒ 422', async () => {
    const id = await quoteId();
    const first = await book(id, 'key-0001').expect(201);
    const again = await book(id, 'key-0001').expect(201);
    expect(again.headers['idempotent-replayed']).toBe('true');
    expect(again.body.trip.id).toBe(first.body.trip.id);
    const reused = await book(id, 'key-0001', C001, { paymentMethod: 'vixanh' });
    expect(reused.status).toBe(422);
    expect(reused.body.error.code).toBe('IDEMPOTENCY_KEY_REUSED');
  });

  it('R-QUOTE-03: báo giá hết hạn ⇒ 422 QUOTE_EXPIRED', async () => {
    const id = await quoteId();
    clock.advanceMinutes(5);
    const res = await book(id, 'key-0002');
    expect(res.status).toBe(422);
    expect(res.body.error.code).toBe('QUOTE_EXPIRED');
  });

  it('R-API-05: khách khác đọc chuyến ⇒ 404 TRIP_NOT_FOUND (không lộ chuyến tồn tại)', async () => {
    const trip = (await book(await quoteId(), 'key-0003').expect(201)).body.trip;
    const res = await request(app).get(`/api/trips/${trip.id}`).set('x-user', C002);
    expect(res.status).toBe(404);
    expect(res.body.error.code).toBe('TRIP_NOT_FOUND');
    await request(app).get('/api/trips/khong-co').set('x-user', C002).expect(404);
  });

  it('R-API-03/04: thiếu x-user ⇒ 401; tài xế đặt chuyến ⇒ 403', async () => {
    expect((await request(app).post('/api/quotes').send(ride)).status).toBe(401);
    const res = await request(app).post('/api/quotes').set('x-user', D101).send(ride);
    expect(res.status).toBe(403);
    expect(res.body.error.code).toBe('FORBIDDEN');
  });

  it('R-API-13: yêu cầu tạo chuyến thứ 6 trong 60 giây ⇒ 429 kèm Retry-After', async () => {
    for (let i = 1; i <= 5; i++) await book(await quoteId(), `key-rate-${i}`);
    const res = await book(await quoteId(), 'key-rate-6');
    expect(res.status).toBe(429);
    expect(res.headers['retry-after']).toBe('60');
  });

  it('vòng đời đầy đủ + thu VíXanh khi hoàn thành (R-STATE-02, R-PAY-03)', async () => {
    const trip = (await book(await quoteId(), 'key-0004', C001, { paymentMethod: 'vixanh' }).expect(201)).body.trip;
    await request(app).post(`/api/trips/${trip.id}/accept`).set('x-user', D101).send({ etaMinutes: 5 }).expect(200);
    for (const step of ['arrive', 'pickup', 'start']) {
      await request(app).post(`/api/trips/${trip.id}/${step}`).set('x-user', D101).send({}).expect(200);
    }
    const done = await request(app).post(`/api/trips/${trip.id}/complete`).set('x-user', D101).send({}).expect(200);
    expect(done.body.trip).toMatchObject({ status: 'completed', paymentStatus: 'paid' });
    expect(system.payments.chargedKeys()).toEqual([`trip:${trip.id}:fare`]);
  });

  it('R-PAY-04: hai lần hết giờ rồi thành công ⇒ 3 lần gọi cùng khoá, trừ tiền một lần', async () => {
    const trip = (await book(await quoteId(), 'key-0005', C001, { paymentMethod: 'vixanh' }).expect(201)).body.trip;
    await request(app).post(`/api/trips/${trip.id}/accept`).set('x-user', D101).send({ etaMinutes: 5 }).expect(200);
    for (const step of ['arrive', 'pickup', 'start']) await request(app).post(`/api/trips/${trip.id}/${step}`).set('x-user', D101).send({});
    system.payments.failNext('lost_response', 'timeout');
    const done = await request(app).post(`/api/trips/${trip.id}/complete`).set('x-user', D101).send({}).expect(200);
    expect(done.body.trip.paymentStatus).toBe('paid');
    expect(new Set(system.payments.calls.map((c) => c.idempotencyKey)).size).toBe(1);
    expect(system.payments.calls).toHaveLength(3);
    expect(system.payments.totalCharged()).toBe(trip.fare);
  });

  it('R-ACCEPT-07: tài xế gửi lại lệnh nhận chuyến mình đang giữ ⇒ 200, phân công không đổi', async () => {
    const trip = (await book(await quoteId(), 'key-0006').expect(201)).body.trip;
    const first = await request(app).post(`/api/trips/${trip.id}/accept`).set('x-user', D101).send({ etaMinutes: 5 }).expect(200);
    clock.advanceMinutes(1);
    const again = await request(app).post(`/api/trips/${trip.id}/accept`).set('x-user', D101).send({ etaMinutes: 9 }).expect(200);
    expect(again.body.trip).toEqual(first.body.trip);
  });

  it('R-TEST-01: API kiểm thử không tồn tại khi không bật', async () => {
    expect((await request(app).post('/api/test/reset')).status).toBe(404);
  });
});
