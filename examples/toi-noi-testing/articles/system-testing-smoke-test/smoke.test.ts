import request from 'supertest';
import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createPgliteSystem, type LabSystem } from '../../src/infra/compose';

describe('SYS-01 · release smoke qua API và PGlite', () => {
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;
  beforeAll(async () => {
    system = await createPgliteSystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
  });
  beforeEach(async () => {
    await system.reset();
    app = createApp(system.deps, { logErrors: false });
  });
  afterAll(async () => { await system?.db?.close(); });

  it('SM-01 · tiến trình API phản hồi health đúng hợp đồng', async () => {
    const res = await request(app).get('/api/health').expect(200);
    expect(res.body).toEqual({ status: 'ok' });
  });

  it('SM-02 · R-API-03: thao tác nghiệp vụ thiếu danh tính bị chặn', async () => {
    const res = await request(app).post('/api/quotes').send({
      kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5,
    }).expect(401);
    expect(res.body.error.code).toBe('UNAUTHENTICATED');
  });

  it('SM-03 · R-STATE-02/R-PAY-03: đặt → hoàn tất, lưu dữ liệu và thu đúng một lần', async () => {
    const quote = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
      .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }).expect(200);
    expect(quote.body.quote.total).toBe(35_000);
    const created = await request(app).post('/api/trips').set('x-user', 'customer:C001')
      .set('Idempotency-Key', 'sys01-smoke-001')
      .send({ quoteId: quote.body.quote.quoteId, paymentMethod: 'vixanh' }).expect(201);
    const id: string = created.body.trip.id;
    expect(id).toEqual(expect.any(String));
    expect(created.body.trip.status).toBe('searching');
    await request(app).post(`/api/trips/${id}/accept`).set('x-user', 'driver:D101')
      .send({ etaMinutes: 5 }).expect(200);
    for (const step of ['arrive', 'pickup', 'start', 'complete']) {
      await request(app).post(`/api/trips/${id}/${step}`).set('x-user', 'driver:D101').send({}).expect(200);
    }
    const read = await request(app).get(`/api/trips/${id}`).set('x-user', 'customer:C001').expect(200);
    expect(read.body.trip).toMatchObject({ id, status: 'completed', fare: 35_000, paymentStatus: 'paid' });
    expect(await system.deps.trips.findById(id)).toMatchObject({ status: 'completed', paymentStatus: 'paid' });
    expect(system.payments.chargedKeys()).toEqual([`trip:${id}:fare`]);
    expect(system.payments.calls).toHaveLength(1);
    expect(system.payments.totalCharged()).toBe(35_000);
  });
});
