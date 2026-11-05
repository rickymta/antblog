/**
 * TEST-039 · Lời giải bài tập 1: phủ nốt 4 ô còn thiếu của ma trận CRUD —
 * R và D trên khoá idempotency của F08 (khách huỷ) và F09 (tài xế huỷ).
 */
import request from 'supertest';
import { beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createPgliteSystem, type LabSystem } from '../../src/infra/compose';
import { CrudTracer } from './crud-tracer';

const tracer = new CrudTracer();
let system: LabSystem;
let app: ReturnType<typeof createApp>;

beforeAll(async () => {
  system = await createPgliteSystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
  app = createApp(tracer.deps(system.deps), { logErrors: false });
});
beforeEach(() => system.reset());

async function tripAcceptedBy(driver: string): Promise<string> {
  const q = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 });
  const t = await request(app).post('/api/trips').set('x-user', 'customer:C001').set('Idempotency-Key', `bt1-${q.body.quote.quoteId}`).send({ quoteId: q.body.quote.quoteId });
  await request(app).post(`/api/trips/${t.body.trip.id}/accept`).set('x-user', `driver:${driver}`).send({ etaMinutes: 5 });
  return t.body.trip.id as string;
}

describe.each([
  ['F08', 'customer:C001', 'cancelled'],
  ['F09', 'driver:D101', 'searching'],
])('%s · huỷ bởi %s', (fn, user, after) => {
  const cancel = (id: string, k: string) =>
    tracer.as(fn, () => request(app).post(`/api/trips/${id}/cancel`).set('x-user', user).set('Idempotency-Key', k).send({}));

  it('R: gửi lại cùng khoá ⇒ trả lại kết quả cũ', async () => {
    const id = await tripAcceptedBy('D101');
    await cancel(id, 'bt1-replay-01');
    const again = await cancel(id, 'bt1-replay-01');
    expect([again.status, again.get('Idempotent-Replayed'), again.body.trip.status]).toEqual([200, 'true', after]);
  });

  it('D: lỗi 5xx ⇒ khoá được thả, thử lại cùng khoá thành công', async () => {
    const id = await tripAcceptedBy('D101');
    tracer.failOnce('trips.updateIfStatus');
    expect((await cancel(id, 'bt1-fault-01')).status).toBe(500);
    const retry = await cancel(id, 'bt1-fault-01');
    expect([retry.status, retry.body.trip.status]).toEqual([200, after]);
  });
});

describe('4 ô đã được phủ', () => {
  it('F08/F09 × idempotency_keys × R/D', () => {
    const cells = ['F08', 'F09'].flatMap((fn) => ['R', 'D'].map((op) => `${fn}|idempotency_keys|${op}`));
    expect(cells.filter((c) => !tracer.hits.has(c))).toEqual([]);
  });
});
