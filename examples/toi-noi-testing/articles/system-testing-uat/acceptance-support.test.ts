import request from 'supertest';
import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { FixedClock } from '../../src/infra/clock';
import { createPgliteSystem, type LabSystem } from '../../src/infra/compose';
import { vnTime } from '../../src/domain/time';

describe('SYS-15 · acceptance evidence support, not business sign-off', () => {
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;
  const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
  beforeAll(async () => { system = await createPgliteSystem({ clock }); });
  beforeEach(async () => {
    clock.set(vnTime(2026, 11, 4, 9, 0));
    await system.reset();
    app = createApp(system.deps, { logErrors: false });
  });
  afterAll(async () => { await system?.db?.close(); });
  async function quote() {
    const r = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
      .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }).expect(200);
    expect(r.body.quote.total).toBe(35_000);
    return r.body.quote.quoteId as string;
  }
  const book = (quoteId: string) => request(app).post('/api/trips').set('x-user', 'customer:C001')
    .set('Idempotency-Key', 'uat-book-001').send({ quoteId, paymentMethod: 'cash' });
  const accept = (id: string) => request(app).post(`/api/trips/${id}/accept`)
    .set('x-user', 'driver:D101').send({ etaMinutes: 5 });

  it('UA-01 · agreed fare, replay and complete cash journey', async () => {
    const qid = await quote();
    const first = await book(qid).expect(201);
    const replay = await book(qid).expect(201);
    expect(replay.body).toEqual(first.body);
    const id = first.body.trip.id as string;
    await accept(id).expect(200);
    for (const step of ['arrive', 'pickup', 'start', 'complete']) {
      await request(app).post(`/api/trips/${id}/${step}`).set('x-user', 'driver:D101').send({}).expect(200);
    }
    const view = await request(app).get(`/api/trips/${id}`).set('x-user', 'customer:C001').expect(200);
    expect(view.body.trip).toMatchObject({ id, fare: 35_000, status: 'completed', paymentStatus: 'cash' });
    expect((await system.db!.query('SELECT count(*)::int AS n FROM trips')).rows).toEqual([{ n: 1 }]);
    expect((await system.deps.trips.listEvents(id)).map(e => e.status))
      .toEqual(['requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed']);
    expect(system.payments.calls).toHaveLength(0);
    expect((await system.deps.drivers.findById('D101'))?.status).toBe('online');
  });

  it.each([
    { minutes: 1, fee: 0, reason: 'grace_period' },
    { minutes: 2, fee: 10_000, reason: 'charged' },
  ])('UA-02 · cancel at $minutes minutes costs $fee', async ({ minutes, fee, reason }) => {
    const r = await book(await quote()).expect(201);
    const id = r.body.trip.id as string;
    await accept(id).expect(200);
    clock.advanceMinutes(minutes);
    const cancelled = await request(app).post(`/api/trips/${id}/cancel`).set('x-user', 'customer:C001')
      .set('Idempotency-Key', 'uat-cancel-001').send({ reason: 'Thay đổi kế hoạch' }).expect(200);
    expect(cancelled.body).toMatchObject({ cancellationFee: fee, feeReason: reason,
      trip: { status: 'cancelled', cancellationFee: fee } });
    expect((await system.deps.trips.findById(id))?.cancellationFee).toBe(fee);
    expect((await system.deps.drivers.findById('D101'))?.status).toBe('online');
    expect(system.payments.calls).toHaveLength(0);
  });

  it('UA-03 · expired offer creates no trip', async () => {
    const qid = await quote();
    clock.advanceMinutes(5);
    const r = await book(qid).expect(422);
    expect(r.body.error.code).toBe('QUOTE_EXPIRED');
    expect((await system.db!.query('SELECT count(*)::int AS n FROM trips')).rows).toEqual([{ n: 0 }]);
    expect((await system.deps.quotes.findById(qid))?.usedByTripId).toBeNull();
    expect(system.payments.calls).toHaveLength(0);
  });
});
