/**
 * TCD-10 · Capstone: bộ test case «đặt chuyến từ báo giá» chạy qua API HTTP thật (Express + supertest)
 * với API kiểm thử /api/test/* (reset, clock, weather) như trên một môi trường test.
 *
 * - Thứ tự chạy = test execution schedule risk-based đã sửa phụ thuộc (hàm của TCD-09).
 * - Mỗi test case cụ thể ghi kết quả; ca cuối sinh RTM vào reports/tcd-10/ và in tóm tắt.
 * - Ca có defect đã biết chạy bằng it.fails (xanh khi defect còn). SHOW_RED=1 để thấy đỏ thật.
 *
 * Chạy: npx vitest run articles/test-case-design-thuc-hanh-dat-chuyen --reporter=verbose
 */
import { mkdirSync, writeFileSync } from 'node:fs';
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import type { AppDeps } from '../../src/app/deps';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { respectDependencies, riskBased } from '../test-case-design-uu-tien-review-bao-tri/uu-tien';
import { buildRtm, renderMarkdown, type Execution } from './rtm';
import { RISKS, TD, TEST_CASES } from './thiet-ke';

const C001 = 'customer:C001';
const C002 = 'customer:C002';
const T0 = '2026-11-04T09:00:00.000+07:00';

const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
const system = createMemorySystem({ clock });
const control = { reset: () => system.reset(), setNow: (d: Date) => clock.set(d), weather: system.weather, payments: system.payments };
const appWith = (overrides: Partial<AppDeps> = {}) => createApp({ ...system.deps, ...overrides }, { logErrors: false, testControl: control });
const app = appWith();
type App = ReturnType<typeof createApp>;

beforeEach(async () => {
  await request(app).post('/api/test/reset').expect(200); // hậu điều kiện của ca trước không rò sang ca sau
  await at(T0);
});

// ------------------------------------------------------------------ bước dùng chung (qua API)
async function at(iso: string) {
  await request(app).post('/api/test/clock').send({ now: iso }).expect(200);
}
async function rain(cities: string[]) {
  await request(app).post('/api/test/weather').send({ raining: cities }).expect(200);
}
async function quote(body: object = TD.RIDE, user = C001, a: App = app) {
  return (await request(a).post('/api/quotes').set('x-user', user).send(body).expect(200)).body.quote;
}
function book(key: string | null, body: object | string, user: string | null = C001, a: App = app) {
  let req = request(a).post('/api/trips');
  if (user) req = req.set('x-user', user);
  if (key) req = req.set('Idempotency-Key', key);
  if (typeof body === 'string') req = req.set('content-type', 'application/json');
  return req.send(body);
}
const cancel = (id: string, key: string) =>
  request(app).post(`/api/trips/${id}/cancel`).set('x-user', C001).set('Idempotency-Key', key).send({}).expect(200);
const waiting = async () =>
  (await request(app).get('/api/driver/available-trips').set('x-user', 'driver:D101').expect(200)).body.trips as { id: string; paymentMethod: string }[];
const err = (res: request.Response) => [res.status, res.body?.error?.code];
/** Tiêu hết 5 lượt tần suất của khách bằng yêu cầu thân rỗng (400 VALIDATION_ERROR vẫn được đếm). */
async function exhaustRateLimit(user = C001) {
  for (let i = 1; i <= 5; i++) expect(err(await book(`spam-key-${i}`, {}, user))).toEqual([400, 'VALIDATION_ERROR']);
}
function gate() {
  let open!: () => void;
  const opened = new Promise<void>((resolve) => (open = resolve));
  return { open, opened };
}

// ------------------------------------------------------------------ ghi kết quả cho RTM
const results: Execution[] = [];
function reg(tc: string, label: string, fn: () => Promise<void>) {
  const defect = TEST_CASES.find((t) => t.id === tc)?.knownDefect;
  const body = async () => {
    try {
      await fn();
      results.push({ tc, name: label, outcome: 'pass' });
    } catch (error) {
      results.push({ tc, name: label, outcome: 'fail' });
      throw error;
    }
  };
  if (defect && !process.env.SHOW_RED) it.fails(`${label} [${defect}]`, body);
  else it(label, body);
}

// ------------------------------------------------------------------ cài đặt từng test case logic
const IMPL: Record<string, () => void> = {
  'TC-BOOK-001': () => reg('TC-BOOK-001', 'TD-01 · không gửi paymentMethod', async () => {
    const q = await quote();
    const trip = (await book('dat-chuyen-0001', { quoteId: q.quoteId }).expect(201)).body.trip;
    expect(trip).toMatchObject({
      status: 'searching', customerId: 'C001', driverId: null, quoteId: q.quoteId, pricingVersion: 'fare-2026.1',
      fare: TD.RIDE_FARE, discount: 0, voucherCode: null, peak: false, raining: false,
      paymentMethod: 'cash', paymentStatus: 'cash', requestedAt: '2026-11-04T02:00:00.000Z',
    });
    expect(trip.fare).toBe(q.total);
    expect(trip.fareBreakdown).toEqual(q.fare);
    const events = await system.deps.trips.listEvents(trip.id);
    expect(events.map((e) => [e.status, e.occurredAt.toISOString()])).toEqual([
      ['requested', '2026-11-04T02:00:00.000Z'],
      ['searching', '2026-11-04T02:00:00.000Z'],
    ]);
    expect(system.payments.calls).toHaveLength(0);
  }),

  'TC-BOOK-002': () => reg('TC-BOOK-002', 'TD-01 · paymentMethod vixanh', async () => {
    const q = await quote();
    const trip = (await book('dat-chuyen-0002', { quoteId: q.quoteId, paymentMethod: 'vixanh' }).expect(201)).body.trip;
    expect(trip).toMatchObject({ paymentMethod: 'vixanh', paymentStatus: 'pending' });
    expect(system.payments.calls).toHaveLength(0);
  }),

  'TC-BOOK-003': () => reg('TC-BOOK-003', 'TD-03 · báo giá 12:58 mưa, đặt 13:01 hết mưa', async () => {
    await at('2026-11-04T12:58:00.000+07:00');
    await rain(['SG']);
    const q = await quote(TD.RIDE_GIAM20);
    expect(q).toMatchObject({ total: TD.PEAK_RAIN_FARE, discount: TD.PEAK_RAIN_DISCOUNT, peak: true, raining: true });
    await at('2026-11-04T13:01:00.000+07:00');
    await rain([]);
    const control = await quote(TD.RIDE_GIAM20); // điều kiện thí nghiệm thật sự đã đổi
    expect(control).toMatchObject({ total: TD.RIDE_GIAM20_FARE, peak: false, raining: false });
    const trip = (await book('dat-chuyen-0003', { quoteId: q.quoteId }).expect(201)).body.trip;
    expect(trip).toMatchObject({ fare: TD.PEAK_RAIN_FARE, discount: TD.PEAK_RAIN_DISCOUNT, peak: true, raining: true, voucherCode: 'GIAM20' });
    expect(trip.fareBreakdown).toEqual(q.fare);
  }),

  'TC-BOOK-004': () => reg('TC-BOOK-004', 'TD-01 · thân có fare 1.000, total 1.000, discount 45.000', async () => {
    const q = await quote();
    const trip = (await book('dat-chuyen-0004', { quoteId: q.quoteId, fare: 1_000, total: 1_000, discount: 45_000 }).expect(201)).body.trip;
    expect(trip).toMatchObject({ fare: TD.RIDE_FARE, discount: 0 });
  }),

  'TC-BOOK-010': () => reg('TC-BOOK-010', 'không tồn tại vs của C002 ⇒ hai thân lỗi giống hệt', async () => {
    const other = await quote(TD.RIDE, C002);
    const missing = await book('dat-chuyen-0010', { quoteId: 'khong-ton-tai-01' });
    const foreign = await book('dat-chuyen-0011', { quoteId: other.quoteId });
    expect(err(missing)).toEqual([422, 'QUOTE_NOT_FOUND']);
    expect(foreign.status).toBe(422);
    expect(foreign.body).toEqual(missing.body);
  }),

  'TC-BOOK-011': () => {
    for (const row of [
      { td: 'TD-11a', now: '2026-11-04T09:04:59.999+07:00', status: 201, code: undefined, trips: 1 },
      { td: 'TD-11b', now: '2026-11-04T09:05:00.000+07:00', status: 422, code: 'QUOTE_EXPIRED', trips: 0 },
    ]) {
      reg('TC-BOOK-011', `${row.td} · đặt lúc ${row.now.slice(11, 23)} ⇒ ${row.status}`, async () => {
        const q = await quote();
        await at(row.now);
        expect(err(await book('dat-chuyen-0012', { quoteId: q.quoteId }))).toEqual([row.status, row.code]);
        expect(await waiting()).toHaveLength(row.trips);
      });
    }
  },

  'TC-BOOK-012': () => reg('TC-BOOK-012', 'khoá thứ hai cho cùng báo giá ⇒ 409', async () => {
    const q = await quote();
    await book('dat-chuyen-0013', { quoteId: q.quoteId }).expect(201);
    expect(err(await book('dat-chuyen-0014', { quoteId: q.quoteId }))).toEqual([409, 'QUOTE_ALREADY_USED']);
    expect(await waiting()).toHaveLength(1);
  }),

  'TC-BOOK-013': () => reg('TC-BOOK-013', 'hai khoá, cùng đọc báo giá trước khi ai đánh dấu', async () => {
    const real = system.deps.quotes;
    const bothRead = gate();
    let reads = 0;
    const quotes = {
      save: real.save.bind(real),
      markUsed: real.markUsed.bind(real),
      findById: async (id: string) => {
        const found = await real.findById(id);
        if (++reads === 2) bothRead.open();
        await bothRead.opened;
        return found;
      },
    };
    const racing = appWith({ quotes });
    const q = await quote(TD.RIDE, C001, racing);
    const res = await Promise.all([
      book('dat-chuyen-0015', { quoteId: q.quoteId }, C001, racing).then((r) => r),
      book('dat-chuyen-0016', { quoteId: q.quoteId }, C001, racing).then((r) => r),
    ]);
    expect(res.map(err).sort()).toEqual([[201, undefined], [409, 'QUOTE_ALREADY_USED']]);
    expect(await waiting()).toHaveLength(1);
  }),

  'TC-BOOK-014': () => {
    reg('TC-BOOK-014', 'cặp 1 · của C002 + hết hạn ⇒ QUOTE_NOT_FOUND', async () => {
      const other = await quote(TD.RIDE, C002);
      await at('2026-11-04T09:05:00.000+07:00');
      expect(err(await book('dat-chuyen-0017', { quoteId: other.quoteId }))).toEqual([422, 'QUOTE_NOT_FOUND']);
    });
    reg('TC-BOOK-014', 'cặp 2 · hết hạn + đã dùng ⇒ QUOTE_EXPIRED', async () => {
      const q = await quote();
      await book('dat-chuyen-0018', { quoteId: q.quoteId }).expect(201);
      await at('2026-11-04T09:05:00.000+07:00');
      expect(err(await book('dat-chuyen-0019', { quoteId: q.quoteId }))).toEqual([422, 'QUOTE_EXPIRED']);
    });
    reg('TC-BOOK-014', 'cặp 3 · đã dùng + hết lượt GIAM20 ⇒ QUOTE_ALREADY_USED', async () => {
      const q = await quote(TD.RIDE_GIAM20);
      await book('dat-chuyen-0020', { quoteId: q.quoteId }).expect(201);
      const probe = await request(app).post('/api/quotes').set('x-user', C001).send(TD.RIDE_GIAM20);
      expect(err(probe)).toEqual([422, 'VOUCHER_USAGE_LIMIT']); // vi phạm thứ hai thật sự có mặt
      expect(err(await book('dat-chuyen-0021', { quoteId: q.quoteId }))).toEqual([409, 'QUOTE_ALREADY_USED']);
    });
  },

  'TC-BOOK-020': () => reg('TC-BOOK-020', 'TD-02 · hai báo giá GIAM20, đặt cái thứ hai', async () => {
    const a = await quote(TD.RIDE_GIAM20);
    const b = await quote(TD.RIDE_GIAM20);
    await book('dat-chuyen-0022', { quoteId: a.quoteId }).expect(201);
    expect(err(await book('dat-chuyen-0023', { quoteId: b.quoteId }))).toEqual([422, 'VOUCHER_USAGE_LIMIT']);
    expect(await waiting()).toHaveLength(1);
  }),

  'TC-BOOK-021': () => reg('TC-BOOK-021', 'TD-02 · huỷ chuyến A rồi đặt B bằng khoá mới', async () => {
    const a = await quote(TD.RIDE_GIAM20);
    const b = await quote(TD.RIDE_GIAM20);
    const tripA = (await book('dat-chuyen-0024', { quoteId: a.quoteId }).expect(201)).body.trip;
    await cancel(tripA.id, 'huy-chuyen-0024');
    const tripB = (await book('dat-chuyen-0025', { quoteId: b.quoteId }).expect(201)).body.trip;
    expect(tripB).toMatchObject({ fare: TD.RIDE_GIAM20_FARE, discount: TD.RIDE_GIAM20_DISCOUNT, voucherCode: 'GIAM20' });
  }),

  'TC-BOOK-022': () => {
    for (const row of [
      { td: 'TD-22a', now: '2026-11-04T23:59:59.999+07:00', status: 422, code: 'VOUCHER_USAGE_LIMIT' },
      { td: 'TD-22b', now: '2026-11-05T00:00:00.000+07:00', status: 201, code: undefined },
    ]) {
      reg('TC-BOOK-022', `${row.td} · đặt B lúc ${row.now.slice(0, 23)} giờ VN ⇒ ${row.status}`, async () => {
        await at('2026-11-04T23:57:00.000+07:00');
        const a = await quote(TD.RIDE_GIAM20);
        await at('2026-11-04T23:58:00.000+07:00');
        const b = await quote(TD.RIDE_GIAM20);
        await at('2026-11-04T23:58:30.000+07:00');
        await book('dat-chuyen-0026', { quoteId: a.quoteId }).expect(201);
        await at(row.now);
        expect(err(await book('dat-chuyen-0027', { quoteId: b.quoteId }))).toEqual([row.status, row.code]);
      });
    }
  },

  'TC-BOOK-023': () => reg('TC-BOOK-023', 'TD-04 · ba báo giá FOOD15, đặt lần lượt', async () => {
    const qs = [await quote(TD.FOOD), await quote(TD.FOOD), await quote(TD.FOOD)];
    expect(qs.map((q) => q.total)).toEqual([TD.FOOD_TOTAL, TD.FOOD_TOTAL, TD.FOOD_TOTAL]);
    const res = [];
    for (const [i, q] of qs.entries()) res.push(err(await book(`dat-do-an-000${i + 1}`, { quoteId: q.quoteId })));
    expect(res).toEqual([[201, undefined], [201, undefined], [422, 'VOUCHER_USAGE_LIMIT']]);
  }),

  'TC-BOOK-030': () => reg('TC-BOOK-030', 'không có header ⇒ 400; báo giá vẫn đặt được', async () => {
    const q = await quote();
    expect(err(await book(null, { quoteId: q.quoteId }))).toEqual([400, 'IDEMPOTENCY_KEY_REQUIRED']);
    expect(await waiting()).toHaveLength(0);
    await book('dat-chuyen-0030', { quoteId: q.quoteId }).expect(201);
  }),

  'TC-BOOK-031': () => {
    for (const [len, status] of [[7, 400], [8, 201], [128, 201], [129, 400]] as const) {
      reg('TC-BOOK-031', `khoá ${len} ký tự ⇒ ${status}`, async () => {
        const q = await quote();
        const res = await book('k'.repeat(len), { quoteId: q.quoteId });
        expect(err(res)).toEqual(status === 201 ? [201, undefined] : [400, 'IDEMPOTENCY_KEY_REQUIRED']);
      });
    }
  },

  'TC-BOOK-032': () => {
    for (const [key, status] of [['Ab9_.:-x', 201], ['dat chuyen1', 400], ['dat/chuyen1', 400], ['dat*chuyen1', 400]] as const) {
      reg('TC-BOOK-032', `khoá «${key}» ⇒ ${status}`, async () => {
        const q = await quote();
        expect((await book(key, { quoteId: q.quoteId })).status).toBe(status);
      });
    }
  },

  'TC-BOOK-033': () => reg('TC-BOOK-033', 'lần hai gửi chuỗi JSON đảo khoá, thêm khoảng trắng', async () => {
    const q = await quote();
    const first = await book('dat-chuyen-0033', { quoteId: q.quoteId, paymentMethod: 'cash' }).expect(201);
    const raw = `{ "paymentMethod" : "cash" ,\n  "quoteId" : "${q.quoteId}" }`;
    const again = await book('dat-chuyen-0033', raw).expect(201);
    expect(again.headers['idempotent-replayed']).toBe('true');
    expect(again.body).toEqual(first.body);
    expect(await waiting()).toHaveLength(1);
    expect(await system.deps.trips.listEvents(first.body.trip.id)).toHaveLength(2);
  }),

  'TC-BOOK-034': () => reg('TC-BOOK-034', 'C001 và C002 cùng khoá dat-chuyen-0034', async () => {
    const q1 = await quote();
    const q2 = await quote(TD.RIDE, C002);
    const t1 = (await book('dat-chuyen-0034', { quoteId: q1.quoteId }).expect(201)).body.trip;
    const r2 = await book('dat-chuyen-0034', { quoteId: q2.quoteId }, C002).expect(201);
    expect(r2.headers['idempotent-replayed']).toBeUndefined();
    expect(r2.body.trip).toMatchObject({ customerId: 'C002' });
    expect(r2.body.trip.id).not.toBe(t1.id);
    expect(await waiting()).toHaveLength(2);
  }),

  'TC-BOOK-035': () => reg('TC-BOOK-035', 'cash rồi vixanh cùng khoá', async () => {
    const q = await quote();
    await book('dat-chuyen-0035', { quoteId: q.quoteId, paymentMethod: 'cash' }).expect(201);
    expect(err(await book('dat-chuyen-0035', { quoteId: q.quoteId, paymentMethod: 'vixanh' }))).toEqual([422, 'IDEMPOTENCY_KEY_REUSED']);
    expect((await waiting()).map((t) => t.paymentMethod)).toEqual(['cash']);
  }),

  'TC-BOOK-036': () => reg('TC-BOOK-036', 'giữ yêu cầu đầu trong createTrip, gửi yêu cầu hai', async () => {
    const hold = gate();
    const entered = gate();
    const real = system.deps.quotes;
    const quotes = {
      save: real.save.bind(real),
      markUsed: real.markUsed.bind(real),
      findById: async (id: string) => {
        entered.open();
        await hold.opened;
        return real.findById(id);
      },
    };
    const slow = appWith({ quotes });
    const q = await quote(TD.RIDE, C001, slow);
    const first = book('dat-chuyen-0036', { quoteId: q.quoteId }, C001, slow).then((r) => r);
    await entered.opened;
    expect(err(await book('dat-chuyen-0036', { quoteId: q.quoteId }, C001, slow))).toEqual([409, 'IDEMPOTENCY_IN_PROGRESS']);
    hold.open();
    expect((await first).status).toBe(201);
  }),

  'TC-BOOK-037': () => reg('TC-BOOK-037', 'B bị 422, huỷ A, gửi lại B cùng khoá', async () => {
    const a = await quote(TD.RIDE_GIAM20);
    const b = await quote(TD.RIDE_GIAM20);
    const tripA = (await book('dat-chuyen-0037', { quoteId: a.quoteId }).expect(201)).body.trip;
    const first = await book('dat-chuyen-0038', { quoteId: b.quoteId });
    expect(err(first)).toEqual([422, 'VOUCHER_USAGE_LIMIT']);
    await cancel(tripA.id, 'huy-chuyen-0037');
    expect((await request(app).post('/api/quotes').set('x-user', C001).send(TD.RIDE_GIAM20)).status).toBe(200); // lượt đã được trả
    const again = await book('dat-chuyen-0038', { quoteId: b.quoteId });
    expect(err(again)).toEqual([422, 'VOUCHER_USAGE_LIMIT']);
    expect(again.headers['idempotent-replayed']).toBe('true');
  }),

  'TC-BOOK-038': () => reg('TC-BOOK-038', 'kho chuyến lỗi một lần ⇒ 500; gửi lại cùng khoá ⇒ 201', async () => {
    const real = system.deps.trips;
    let failed = false;
    const trips = new Proxy(real, {
      get(target, prop, receiver) {
        if (prop === 'create') {
          return async (...args: Parameters<AppDeps['trips']['create']>) => {
            if (!failed) {
              failed = true;
              throw new Error('ECONNRESET trips insert');
            }
            return target.create(...args);
          };
        }
        const value = Reflect.get(target, prop, receiver);
        return typeof value === 'function' ? value.bind(target) : value;
      },
    });
    const flaky = appWith({ trips });
    const q = await quote(TD.RIDE, C001, flaky);
    expect(err(await book('dat-chuyen-0500', { quoteId: q.quoteId }, C001, flaky))).toEqual([500, 'INTERNAL_ERROR']);
    const retry = await book('dat-chuyen-0500', { quoteId: q.quoteId }, C001, flaky);
    expect(retry.headers['idempotent-replayed']).toBeUndefined();
    expect(err(retry)).toEqual([201, undefined]); // thực tế 409 QUOTE_ALREADY_USED
  }),

  'TC-BOOK-039': () => reg('TC-BOOK-039', 'khoá K bị 429 ở 0 giây, gửi lại K ở 60 giây', async () => {
    const q = await quote();
    await exhaustRateLimit();
    expect(err(await book('dat-chuyen-0039', { quoteId: q.quoteId }))).toEqual([429, 'RATE_LIMITED']);
    await at('2026-11-04T09:01:00.000+07:00');
    const later = await book('dat-chuyen-0039', { quoteId: q.quoteId });
    expect(err(later)).toEqual([201, undefined]);
    expect(later.headers['idempotent-replayed']).toBeUndefined();
  }),

  'TC-BOOK-050': () => reg('TC-BOOK-050', 'sáu báo giá, sáu lần đặt cùng thời điểm', async () => {
    const qs = [];
    for (let i = 0; i < 6; i++) qs.push(await quote());
    const res = [];
    for (const [i, q] of qs.entries()) res.push(await book(`dat-chuyen-005${i}`, { quoteId: q.quoteId }));
    expect(res.map((r) => r.status)).toEqual([201, 201, 201, 201, 201, 429]);
    expect(res[5]!.headers['retry-after']).toBe('60');
    expect(await waiting()).toHaveLength(5);
  }),

  'TC-BOOK-051': () => {
    for (const row of [
      { td: 'TD-51a', now: '2026-11-04T09:00:59.999+07:00', status: 429, retryAfter: '1' },
      { td: 'TD-51b', now: '2026-11-04T09:01:00.000+07:00', status: 201, retryAfter: undefined },
    ]) {
      reg('TC-BOOK-051', `${row.td} · yêu cầu thứ 6 lúc ${row.now.slice(11, 23)} ⇒ ${row.status}`, async () => {
        const qs = [];
        for (let i = 0; i < 6; i++) qs.push(await quote());
        for (let i = 0; i < 5; i++) await book(`dat-chuyen-051${i}`, { quoteId: qs[i]!.quoteId }).expect(201);
        await at(row.now);
        const sixth = await book('dat-chuyen-0515', { quoteId: qs[5]!.quoteId });
        expect(sixth.status).toBe(row.status);
        expect(sixth.headers['retry-after']).toBe(row.retryAfter);
      });
    }
  },

  'TC-BOOK-052': () => reg('TC-BOOK-052', 'một lần tạo, bốn lần gửi lại, lần thứ sáu', async () => {
    const q = await quote();
    const statuses = [];
    for (let i = 0; i < 6; i++) {
      const r = await book('dat-chuyen-0052', { quoteId: q.quoteId });
      statuses.push(`${r.status}${r.headers['idempotent-replayed'] ? ' replayed' : ''}`);
    }
    expect(statuses).toEqual(['201', '201 replayed', '201 replayed', '201 replayed', '201 replayed', '429']);
  }),

  'TC-BOOK-053': () => reg('TC-BOOK-053', '5 yêu cầu ở 0 giây, 5 lần bị 429 ở 30 giây, đặt ở 60 giây', async () => {
    const q = await quote();
    await exhaustRateLimit();
    await at('2026-11-04T09:00:30.000+07:00');
    for (let i = 0; i < 5; i++) {
      const r = await book(`bi-chan-${i}-key`, { quoteId: q.quoteId });
      expect([...err(r), r.headers['retry-after']]).toEqual([429, 'RATE_LIMITED', '30']);
    }
    await at('2026-11-04T09:01:00.000+07:00');
    await book('dat-chuyen-0053', { quoteId: q.quoteId }).expect(201);
  }),

  'TC-BOOK-054': () => reg('TC-BOOK-054', 'C001 hết lượt, C002 đặt cùng thời điểm', async () => {
    await exhaustRateLimit();
    expect(err(await book('dat-chuyen-0054', { quoteId: (await quote()).quoteId }))).toEqual([429, 'RATE_LIMITED']);
    await book('dat-chuyen-0055', { quoteId: (await quote(TD.RIDE, C002)).quoteId }, C002).expect(201);
  }),

  'TC-BOOK-060': () => {
    reg('TC-BOOK-060', 'cặp a · x-user không tồn tại + thiếu khoá + thân sai ⇒ 401', async () => {
      expect(err(await book(null, {}, 'customer:C999'))).toEqual([401, 'UNAUTHENTICATED']);
    });
    reg('TC-BOOK-060', 'cặp b · tài xế + thiếu khoá + thân sai ⇒ 403', async () => {
      expect(err(await book(null, {}, 'driver:D101'))).toEqual([403, 'FORBIDDEN']);
    });
    reg('TC-BOOK-060', 'cặp c · đã hết lượt tần suất + thiếu khoá ⇒ 429', async () => {
      await exhaustRateLimit();
      expect(err(await book(null, {}))).toEqual([429, 'RATE_LIMITED']);
    });
    reg('TC-BOOK-060', 'cặp d · thiếu khoá + quoteId sai kiểu ⇒ 400 IDEMPOTENCY_KEY_REQUIRED', async () => {
      expect(err(await book(null, { quoteId: 123 }))).toEqual([400, 'IDEMPOTENCY_KEY_REQUIRED']);
    });
    reg('TC-BOOK-060', 'cặp e · khoá đã dùng + thân sai kiểu ⇒ 422 IDEMPOTENCY_KEY_REUSED', async () => {
      const q = await quote();
      await book('dat-chuyen-0060', { quoteId: q.quoteId }).expect(201);
      expect(err(await book('dat-chuyen-0060', { quoteId: 5 }))).toEqual([422, 'IDEMPOTENCY_KEY_REUSED']);
    });
    reg('TC-BOOK-060', 'cặp f · paymentMethod "card" + báo giá hết hạn ⇒ 400 VALIDATION_ERROR', async () => {
      const q = await quote();
      await at('2026-11-04T09:05:00.000+07:00');
      expect(err(await book('dat-chuyen-0061', { quoteId: q.quoteId, paymentMethod: 'card' }))).toEqual([400, 'VALIDATION_ERROR']);
    });
  },

  'TC-BOOK-061': () => reg('TC-BOOK-061', 'JSON hỏng, không có x-user ⇒ 401', async () => {
    expect(err(await book('dat-chuyen-0062', '{"quoteId":', null))).toEqual([401, 'UNAUTHENTICATED']); // thực tế 400 INVALID_JSON
  }),

  'TC-BOOK-062': () => reg('TC-BOOK-062', 'đã hết lượt tần suất + JSON hỏng ⇒ 429', async () => {
    await exhaustRateLimit();
    expect(err(await book('dat-chuyen-0063', '{"quoteId":'))).toEqual([429, 'RATE_LIMITED']); // thực tế 400 INVALID_JSON
  }),
};

// ------------------------------------------------------------------ test execution schedule
export const SCHEDULE = respectDependencies(riskBased(TEST_CASES, RISKS), TEST_CASES);

SCHEDULE.forEach((id, i) => {
  const tc = TEST_CASES.find((t) => t.id === id)!;
  describe(`#${String(i + 1).padStart(2, '0')} ${id} · ${tc.reqs.join(', ')} · ${tc.title}`, IMPL[id]!);
});

describe('Báo cáo', () => {
  it('RTM và test completion summary (ghi reports/tcd-10/)', () => {
    const summary = buildRtm(results);
    const md = renderMarkdown(summary, SCHEDULE);
    const dir = new URL('../../reports/tcd-10/', import.meta.url);
    mkdirSync(dir, { recursive: true });
    writeFileSync(new URL('rtm.md', dir), md, 'utf8');
    writeFileSync(new URL('rtm.json', dir), JSON.stringify({ summary, results }, null, 2), 'utf8');
    console.log(md);

    expect(Object.keys(IMPL).sort()).toEqual(TEST_CASES.map((t) => t.id).sort());
    expect(summary.executions).toBe(summary.plannedExecutions);
    expect(summary.inScopeCovered).toBe(summary.inScope);
    expect(summary.conditionsCovered).toBe(summary.conditions);
    expect(summary.orphanTestCases).toEqual([]);
    const failedCases = [...new Set(results.filter((r) => r.outcome === 'fail').map((r) => r.tc))];
    const expectedFails = process.env.SHOW_RED ? failedCases : TEST_CASES.filter((t) => t.knownDefect).map((t) => t.id);
    expect(failedCases.sort()).toEqual(expectedFails.sort());
  });
});
