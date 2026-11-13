import request from 'supertest';
import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { FixedClock } from '../../src/infra/clock';
import { createPgliteSystem, type LabSystem } from '../../src/infra/compose';
import { vnTime } from '../../src/domain/time';

describe('SYS-13 · SQL constraints, atomicity and reconciliation', () => {
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
  async function quote(extra = {}) {
    const r = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
      .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5, ...extra }).expect(200);
    return r.body.quote.quoteId as string;
  }
  async function book(qid: string) {
    const r = await request(app).post('/api/trips').set('x-user', 'customer:C001')
      .set('Idempotency-Key', 'sys13-book-001').send({ quoteId: qid }).expect(201);
    return (await system.deps.trips.findById(r.body.trip.id))!;
  }
  const anomalies = () => system.db!.query(`
    SELECT t.id FROM trips t LEFT JOIN quotes q ON q.id=t.quote_id
    WHERE q.id IS NULL OR t.customer_id IS DISTINCT FROM q.customer_id
       OR t.pricing_version IS DISTINCT FROM q.pricing_version
       OR t.fare IS DISTINCT FROM (q.details->>'total')::numeric
       OR q.used_by_trip_id IS DISTINCT FROM t.id`);

  it('DI-01 · snapshot and ordered initial audit agree with SQL', async () => {
    const trip = await book(await quote());
    expect(trip.fare).toBe(35_000);
    expect((await anomalies()).rows).toEqual([]);
    const events = await system.db!.query(
      'SELECT status, occurred_at FROM trip_events WHERE trip_id=$1 ORDER BY id', [trip.id]);
    expect(events.rows).toEqual([
      { status: 'requested', occurred_at: clock.now() },
      { status: 'searching', occurred_at: clock.now() },
    ]);
  });
  it('DI-02 · UNIQUE prevents a second trip for the same quote', async () => {
    const trip = await book(await quote());
    await expect(system.deps.trips.create({ ...trip, id: 'duplicate-trip' }, []))
      .rejects.toMatchObject({ code: '23505' });
    expect((await system.db!.query('SELECT id FROM trips')).rows).toEqual([{ id: trip.id }]);
  });
  it('DI-03 · failed second event rolls back trip and first event', async () => {
    const original = await book(await quote());
    const qid = await quote();
    const candidate = { ...original, id: 'rollback-trip', quoteId: qid };
    await expect(system.deps.trips.create(candidate, [
      { tripId: candidate.id, status: 'requested', occurredAt: clock.now() },
      { tripId: 'missing-trip', status: 'searching', occurredAt: clock.now() },
    ])).rejects.toMatchObject({ code: '23503' });
    expect(await system.deps.trips.findById(candidate.id)).toBeNull();
    expect(await system.deps.trips.listEvents(candidate.id)).toEqual([]);
    expect(await system.deps.trips.findById(original.id)).toEqual(original);
    expect(await system.deps.trips.listEvents(original.id)).toHaveLength(2);
  });
  it('DI-04 · CHECK and NOT NULL reject invalid writes without changing data', async () => {
    const trip = await book(await quote());
    await expect(system.db!.query('UPDATE trips SET status=$1 WHERE id=$2', ['unknown', trip.id]))
      .rejects.toMatchObject({ code: '23514' });
    await expect(system.db!.query('UPDATE trips SET fare=NULL WHERE id=$1', [trip.id]))
      .rejects.toMatchObject({ code: '23502' });
    expect(await system.deps.trips.findById(trip.id)).toEqual(trip);
  });
  it('DI-05 · fractional discount survives API, numeric, JSON and repository', async () => {
    clock.set(vnTime(2026, 11, 4, 11, 0));
    const trip = await book(await quote({ distanceKm: 0.5, voucherCode: 'GIAM20' }));
    expect(trip.discount).toBe(3562.5);
    expect(trip.fare).toBe(15_000);
    expect(trip.fareBreakdown.discount).toBe(3562.5);
    const sql = await system.db!.query(`SELECT discount=3562.5 AS exact,
      (fare_breakdown->>'discount')::numeric=discount AS json_agrees FROM trips WHERE id=$1`, [trip.id]);
    expect(sql.rows).toEqual([{ exact: true, json_agrees: true }]);
    expect((await anomalies()).rows).toEqual([]);
  });
  it('DI-06 · reconciliation detects deliberately corrupted fare', async () => {
    const trip = await book(await quote());
    expect((await anomalies()).rows).toEqual([]);
    await system.db!.query('UPDATE trips SET fare=fare+1000 WHERE id=$1', [trip.id]);
    expect((await anomalies()).rows).toEqual([{ id: trip.id }]);
  });
});
