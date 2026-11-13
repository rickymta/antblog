import request from 'supertest';
import { afterAll, beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { FixedClock } from '../../src/infra/clock';
import { createPgliteSystem, type LabSystem } from '../../src/infra/compose';
import { vnTime } from '../../src/domain/time';

describe('SYS-04 · API + PostgreSQL WASM + payment port', () => {
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;
  beforeAll(async () => {
    system = await createPgliteSystem({
      clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)),
      config: { payment: { timeoutMs: 50, maxRetries: 2 } },
    });
  });
  beforeEach(async () => {
    await system.reset();
    app = createApp(system.deps, { logErrors: false });
  });
  afterAll(async () => { await system?.db?.close(); });

  async function quoteId() {
    const q = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
      .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }).expect(200);
    return q.body.quote.quoteId as string;
  }
  const book = (id: string, key = 'sys04-book-001') => request(app).post('/api/trips')
    .set('x-user', 'customer:C001').set('Idempotency-Key', key)
    .send({ quoteId: id, paymentMethod: 'vixanh' });
  async function inTransit() {
    const created = await book(await quoteId()).expect(201);
    const id: string = created.body.trip.id;
    await request(app).post(`/api/trips/${id}/accept`).set('x-user', 'driver:D101').send({ etaMinutes: 5 }).expect(200);
    for (const step of ['arrive', 'pickup', 'start']) {
      await request(app).post(`/api/trips/${id}/${step}`).set('x-user', 'driver:D101').send({}).expect(200);
    }
    return id;
  }
  const complete = (id: string) => request(app).post(`/api/trips/${id}/complete`)
    .set('x-user', 'driver:D101').send({});

  it('IN-01 · R-QUOTE-03/R-IDEM-02: replay không tạo thêm chuyến hoặc nhật ký', async () => {
    const qid = await quoteId();
    const first = await book(qid).expect(201);
    const again = await book(qid).expect(201);
    expect(again.headers['idempotent-replayed']).toBe('true');
    expect(again.body).toEqual(first.body);
    const tripId = first.body.trip.id;
    const trips = await system.db!.query('SELECT id, fare, payment_status FROM trips WHERE quote_id=$1', [qid]);
    expect(trips.rows).toEqual([{ id: tripId, fare: 35_000, payment_status: 'pending' }]);
    const quote = await system.db!.query('SELECT used_by_trip_id FROM quotes WHERE id=$1', [qid]);
    expect(quote.rows).toEqual([{ used_by_trip_id: tripId }]);
    const events = await system.db!.query('SELECT status FROM trip_events WHERE trip_id=$1 ORDER BY id', [tripId]);
    expect(events.rows).toEqual([{ status: 'requested' }, { status: 'searching' }]);
  });

  it('IN-02 · schema: foreign key từ chối nhật ký không có chuyến', async () => {
    await expect(system.db!.query(
      "INSERT INTO trip_events(trip_id,status,occurred_at) VALUES ($1,'requested',$2)",
      ['missing-trip', '2026-11-04T02:00:00Z'],
    )).rejects.toMatchObject({ code: '23503' });
    const rows = await system.db!.query('SELECT count(*)::int AS n FROM trip_events');
    expect(rows.rows).toEqual([{ n: 0 }]);
  });

  it.each([
    { mode: 'none' as const, attempts: 1, status: 'paid', charged: 35_000 },
    { mode: 'lost_response' as const, attempts: 2, status: 'paid', charged: 35_000 },
    { mode: 'decline' as const, attempts: 1, status: 'failed', charged: 0 },
    { mode: 'timeout' as const, attempts: 3, status: 'failed', charged: 0 },
    { mode: 'error' as const, attempts: 3, status: 'failed', charged: 0 },
  ])('IN-03 · R-PAY-03/04/06: $mode → $status, $attempts lần gọi', async ({ mode, attempts, status, charged }) => {
    const id = await inTransit();
    if (mode === 'lost_response') system.payments.failNext('lost_response');
    else system.payments.setMode(mode);
    const res = await complete(id).expect(200);
    expect(res.body.trip).toMatchObject({ status: 'completed', paymentStatus: status });
    const stored = await system.db!.query('SELECT status, payment_status FROM trips WHERE id=$1', [id]);
    expect(stored.rows).toEqual([{ status: 'completed', payment_status: status }]);
    const driver = await system.db!.query("SELECT status FROM drivers WHERE id='D101'");
    expect(driver.rows).toEqual([{ status: 'online' }]);
    expect(system.payments.calls).toHaveLength(attempts);
    for (const call of system.payments.calls) {
      expect(call).toMatchObject({ idempotencyKey: `trip:${id}:fare`, customerId: 'C001', amount: 35_000 });
    }
    expect(system.payments.totalCharged()).toBe(charged);
    expect(system.payments.chargedKeys()).toEqual(charged ? [`trip:${id}:fare`] : []);
    const events = await system.db!.query("SELECT count(*)::int AS n FROM trip_events WHERE trip_id=$1 AND status='completed'", [id]);
    expect(events.rows).toEqual([{ n: 1 }]);
  });
});
