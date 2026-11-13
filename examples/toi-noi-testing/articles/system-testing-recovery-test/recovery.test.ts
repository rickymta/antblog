import request from 'supertest';
import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { FixedClock } from '../../src/infra/clock';
import { createPgliteSystem, type LabSystem } from '../../src/infra/compose';
import { vnTime } from '../../src/domain/time';

describe('SYS-14 · dependency recovery and unfinished reconciliation', () => {
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;
  beforeAll(async () => {
    system = await createPgliteSystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)),
      config: { payment: { timeoutMs: 30, maxRetries: 2 } } });
  });
  beforeEach(async () => { await system.reset(); app = createApp(system.deps, { logErrors: false }); });
  afterAll(async () => { await system?.db?.close(); });
  async function inTransit(key: string) {
    const q = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
      .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }).expect(200);
    const r = await request(app).post('/api/trips').set('x-user', 'customer:C001')
      .set('Idempotency-Key', key).send({ quoteId: q.body.quote.quoteId, paymentMethod: 'vixanh' }).expect(201);
    const id = r.body.trip.id as string;
    await request(app).post(`/api/trips/${id}/accept`).set('x-user', 'driver:D101').send({ etaMinutes: 5 }).expect(200);
    for (const step of ['arrive', 'pickup', 'start']) {
      await request(app).post(`/api/trips/${id}/${step}`).set('x-user', 'driver:D101').send({}).expect(200);
    }
    return id;
  }
  const complete = (id: string) => request(app).post(`/api/trips/${id}/complete`)
    .set('x-user', 'driver:D101').send({});
  async function stored(id: string, payment: string) {
    expect((await system.db!.query('SELECT status,payment_status FROM trips WHERE id=$1', [id])).rows)
      .toEqual([{ status: 'completed', payment_status: payment }]);
    expect((await system.db!.query("SELECT count(*)::int AS n FROM trip_events WHERE trip_id=$1 AND status='completed'", [id])).rows)
      .toEqual([{ n: 1 }]);
    expect((await system.db!.query("SELECT status FROM drivers WHERE id='D101'")).rows).toEqual([{ status: 'online' }]);
  }

  it.each(['error', 'timeout', 'lost_response'] as const)(
    'RC-01 · one %s then recovery within retry budget', async mode => {
      const id = await inTransit('sys14-transient');
      system.payments.failNext(mode);
      await complete(id).expect(200);
      await stored(id, 'paid');
      expect(system.payments.calls).toHaveLength(2);
      expect(system.payments.calls.map(c => c.idempotencyKey)).toEqual([`trip:${id}:fare`, `trip:${id}:fare`]);
      expect(system.payments.totalCharged()).toBe(35_000);
      expect(system.payments.chargedKeys()).toEqual([`trip:${id}:fare`]);
    });

  it('RC-02 · new work recovers after outage; old failed payment remains unresolved', async () => {
    const oldId = await inTransit('sys14-outage-old');
    system.payments.setMode('timeout');
    await complete(oldId).expect(200);
    await stored(oldId, 'failed');
    expect(system.payments.calls).toHaveLength(3);
    expect(system.payments.totalCharged()).toBe(0);
    system.payments.setMode('none');
    const newId = await inTransit('sys14-outage-new');
    await complete(newId).expect(200);
    await stored(newId, 'paid');
    await stored(oldId, 'failed');
    await complete(oldId).expect(409);
    expect(system.payments.calls).toHaveLength(4);
    expect(system.payments.chargedKeys()).toEqual([`trip:${newId}:fare`]);
    expect(system.payments.totalCharged()).toBe(35_000);
  });

  it('RC-03 · persistent lost responses leave a charged-but-failed discrepancy', async () => {
    const id = await inTransit('sys14-ambiguous');
    system.payments.setMode('lost_response');
    await complete(id).expect(200);
    await stored(id, 'failed');
    expect(system.payments.calls).toHaveLength(3);
    expect(system.payments.chargedKeys()).toEqual([`trip:${id}:fare`]);
    expect(system.payments.totalCharged()).toBe(35_000);
    system.payments.setMode('none');
    await complete(id).expect(409);
    await stored(id, 'failed');
    expect(system.payments.calls).toHaveLength(3);
    expect(system.payments.totalCharged()).toBe(35_000);
  });
});
