/**
 * TEST-039 · Consistency testing (kiểm ĐỘNG) cho thực thể chuyến, qua API thật + PostgreSQL (PGlite).
 * - Vòng đời theo thực thể: R trước C, C, rồi sau MỖI lần U đọc lại bằng MỌI chức năng R.
 * - Ma trận vai trò × thao tác (kiểm soát truy cập).
 * - CRUD coverage: ô của ma trận SPEC đã được thực thi / tổng số ô.
 */
import request from 'supertest';
import { beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import type { AppDeps } from '../../src/app/deps';
import { expireStaleSearches } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createPgliteSystem, type LabSystem } from '../../src/infra/compose';
import { matrixCells, SPEC_MATRIX } from './crud-matrix';
import { CrudTracer } from './crud-tracer';

const START = vnTime(2026, 11, 4, 9, 0); // 09:00 giờ VN — ngoài cao điểm
const clock = new FixedClock(START);
const tracer = new CrudTracer();
let system: LabSystem;
let deps: AppDeps;
let app: ReturnType<typeof createApp>;
let seq = 0;
const key = () => `crud-key-${String(++seq).padStart(5, '0')}`;
const RIDE = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 };

beforeAll(async () => {
  system = await createPgliteSystem({ clock, config: { tripRateLimit: { max: 0, windowMs: 60_000 } } });
  deps = tracer.deps(system.deps);
  app = createApp(deps, { logErrors: false });
});
beforeEach(async () => {
  await system.reset();
  clock.set(START);
});

// ---- mỗi chức năng của ma trận = một hàm gọi API, gắn nhãn F01…F10
const quote = (c: string, body: object = RIDE) =>
  tracer.as('F01', () => request(app).post('/api/quotes').set('x-user', `customer:${c}`).send(body));
const book = (c: string, quoteId: string, k = key()) =>
  tracer.as('F02', () => request(app).post('/api/trips').set('x-user', `customer:${c}`).set('Idempotency-Key', k).send({ quoteId }));
const view = (user: string, id: string) => tracer.as('F03', () => request(app).get(`/api/trips/${id}`).set('x-user', user));
const list = (d: string) => tracer.as('F04', () => request(app).get('/api/driver/available-trips').set('x-user', `driver:${d}`));
const accept = (d: string, id: string) =>
  tracer.as('F05', () => request(app).post(`/api/trips/${id}/accept`).set('x-user', `driver:${d}`).send({ etaMinutes: 5 }));
const step = (d: string, id: string, s: 'arrive' | 'pickup' | 'start' | 'complete') =>
  tracer.as(s === 'complete' ? 'F07' : 'F06', () => request(app).post(`/api/trips/${id}/${s}`).set('x-user', `driver:${d}`));
const cancel = (user: string, id: string, k = key()) =>
  tracer.as(user.startsWith('customer') ? 'F08' : 'F09', () =>
    request(app).post(`/api/trips/${id}/cancel`).set('x-user', user).set('Idempotency-Key', k).send({}));

async function newTrip(c = 'C001', body: object = RIDE): Promise<string> {
  const q = await quote(c, body);
  const t = await book(c, q.body.quote.quoteId);
  expect(t.status).toBe(201);
  return t.body.trip.id as string;
}

const outcome = (r: request.Response) => (r.status < 300 ? String(r.status) : `${r.status} ${r.body.error.code}`);

/** «Sau mỗi U, mọi R»: đọc chuyến bằng mọi chức năng đọc, dưới mọi vai trò. */
async function readAll(id: string) {
  const seen = async (user: string) => {
    const r = await view(user, id);
    return r.status === 200 ? (r.body.trip.status as string) : r.status;
  };
  const inList = async (d: string) => ((await list(d)).body.trips as { id: string }[]).some((t) => t.id === id);
  return {
    owner: await seen('customer:C001'),
    otherCustomer: await seen('customer:C002'),
    d101: await seen('driver:D101'),
    d102: await seen('driver:D102'),
    d103car: await seen('driver:D103'),
    d201hn: await seen('driver:D201'),
    listD101: await inList('D101'),
    listD102: await inList('D102'),
  };
}

/** Toàn vẹn dữ liệu đọc thẳng từ database: trạng thái = dòng nhật ký cuối; tài xế busy ⇔ đang giữ chuyến. */
async function integrity(id: string) {
  const db = system.db!;
  const trip = (await db.query<{ status: string; driver_id: string | null }>('SELECT status, driver_id FROM trips WHERE id = $1', [id])).rows[0]!;
  const events = (await db.query<{ status: string }>('SELECT status FROM trip_events WHERE trip_id = $1 ORDER BY id', [id])).rows.map((e) => e.status);
  const busy = (await db.query<{ id: string }>("SELECT id FROM drivers WHERE status = 'busy' ORDER BY id")).rows.map((d) => d.id);
  const active = ['accepted', 'arriving', 'picked_up', 'in_transit'].includes(trip.status);
  expect(events.at(-1)).toBe(trip.status);
  expect(busy).toEqual(active ? [trip.driver_id] : []);
  return events;
}

const owned = { owner: 'searching', otherCustomer: 404, d101: 'searching', d102: 'searching', d103car: 404, d201hn: 404, listD101: true, listD102: true };
const heldBy101 = (s: string) => ({ owner: s, otherCustomer: 404, d101: s, d102: 404, d103car: 404, d201hn: 404, listD101: false, listD102: false });

describe('TEST-039 · vòng đời thực thể chuyến (consistency)', () => {
  it('L1 · R trước C → C → U×5 (đọc lại sau mỗi U) → U sau kết thúc bị chặn → không có D', async () => {
    expect(outcome(await view('customer:C001', 'id-9999'))).toBe('404 TRIP_NOT_FOUND'); // đọc thực thể chưa tạo
    const id = await newTrip('C001', { ...RIDE, voucherCode: 'GIAM20' });
    expect(await readAll(id)).toEqual(owned);
    expect(await integrity(id)).toEqual(['requested', 'searching']);

    expect((await accept('D101', id)).status).toBe(200);
    expect(await readAll(id)).toEqual(heldBy101('accepted'));
    for (const [s, status] of [['arrive', 'arriving'], ['pickup', 'picked_up'], ['start', 'in_transit'], ['complete', 'completed']] as const) {
      expect((await step('D101', id, s)).status).toBe(200);
      expect(await readAll(id)).toEqual(heldBy101(status));
      await integrity(id);
    }
    expect(await integrity(id)).toEqual(['requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed']);

    expect(outcome(await cancel('customer:C001', id))).toBe('409 INVALID_TRANSITION'); // U sau trạng thái kết thúc
    expect(outcome(await cancel('driver:D101', id))).toBe('409 INVALID_TRANSITION');
    expect(outcome(await request(app).delete(`/api/trips/${id}`).set('x-user', 'customer:C001'))).toBe('404 NOT_FOUND');
    expect((await view('customer:C001', id)).body.trip.status).toBe('completed'); // vẫn còn: không có xoá cứng
    // thực thể khác đọc lại chuyến: lượt voucher của chuyến đã hoàn thành vẫn bị tính (R-VOUCHER-07)
    expect(outcome(await quote('C001', { ...RIDE, voucherCode: 'GIAM20' }))).toBe('422 VOUCHER_USAGE_LIMIT');
  });

  it('L2 · khách huỷ chuyến đã có tài xế: tài xế rảnh lại, lượt voucher được trả', async () => {
    const id = await newTrip('C001', { ...RIDE, voucherCode: 'GIAM20' });
    await accept('D101', id);
    const r = await cancel('customer:C001', id);
    expect([r.status, r.body.cancellationFee, r.body.feeReason]).toEqual([200, 0, 'grace_period']);
    expect(await readAll(id)).toEqual(heldBy101('cancelled')); // D101 vẫn xem được chuyến đã huỷ — xem mục 6
    expect(await integrity(id)).toEqual(['requested', 'searching', 'accepted', 'cancelled']);
    expect(outcome(await quote('C001', { ...RIDE, voucherCode: 'GIAM20' }))).toBe('200');
  });

  it('L3 · tài xế huỷ: chuyến về searching, bỏ gán, hiện lại cho mọi tài xế đủ điều kiện', async () => {
    const id = await newTrip();
    await accept('D102', id);
    expect((await cancel('driver:D102', id)).body.trip).toMatchObject({ status: 'searching', driverId: null });
    expect(await readAll(id)).toEqual(owned);
    expect(await integrity(id)).toEqual(['requested', 'searching', 'accepted', 'searching']);
    expect((await accept('D101', id)).body.trip.driverId).toBe('D101');
  });

  it('L4 · hệ thống huỷ sau 5 phút không có tài xế: chuyến biến khỏi mọi danh sách chờ', async () => {
    const id = await newTrip();
    clock.advanceMinutes(5);
    expect(await tracer.as('F10', () => expireStaleSearches(deps))).toBe(1);
    expect(await readAll(id)).toEqual({ ...owned, owner: 'cancelled', d101: 404, d102: 404, listD101: false, listD102: false });
    expect(await integrity(id)).toEqual(['requested', 'searching', 'cancelled']);
  });

  it('L5 · gửi lại cùng Idempotency-Key: đọc lại kết quả cũ, không tạo chuyến thứ hai', async () => {
    const q = await quote('C001');
    const k = key();
    const first = await book('C001', q.body.quote.quoteId, k);
    const again = await book('C001', q.body.quote.quoteId, k);
    expect([again.status, again.get('Idempotent-Replayed'), again.body.trip.id]).toEqual([201, 'true', first.body.trip.id]);
    const n = await system.db!.query<{ n: number }>('SELECT count(*)::int AS n FROM trips WHERE quote_id = $1', [q.body.quote.quoteId]);
    expect(n.rows[0]?.n).toBe(1);
  });

  it('L6 · database lỗi lúc ghi chuyến: khoá idempotency được thả (D) nhưng báo giá đã bị đánh dấu dùng', async () => {
    const q = await quote('C001');
    const k = key();
    tracer.failOnce('trips.create');
    expect(outcome(await book('C001', q.body.quote.quoteId, k))).toBe('500 INTERNAL_ERROR');
    expect(outcome(await book('C001', q.body.quote.quoteId, k))).toBe('409 QUOTE_ALREADY_USED'); // thực tế
    const orphan = await system.db!.query<{ used_by_trip_id: string; trip: string | null }>(
      'SELECT q.used_by_trip_id, t.id AS trip FROM quotes q LEFT JOIN trips t ON t.id = q.used_by_trip_id WHERE q.id = $1',
      [q.body.quote.quoteId],
    );
    expect(orphan.rows[0]?.used_by_trip_id).toBeTruthy();
    expect(orphan.rows[0]?.trip).toBeNull(); // báo giá trỏ tới một chuyến không tồn tại
  });

  it.fails('L6-kỳ vọng · R-IDEM-05: sau 5xx, thử lại cùng khoá phải đặt được chuyến (đang ĐỎ — xem DR-039-01)', async () => {
    const q = await quote('C001');
    const k = key();
    tracer.failOnce('trips.create');
    await book('C001', q.body.quote.quoteId, k);
    expect(outcome(await book('C001', q.body.quote.quoteId, k))).toBe('201');
  });
});

// ---- ma trận vai trò × thao tác trên thực thể chuyến (R-API-04, R-API-05, R-API-08…11, R-ACCEPT-02)
type Col = 'tao' | 'xemSearching' | 'nhanSearching' | 'xemAccepted' | 'arriveAccepted' | 'huyAccepted' | 'xoa';
const ROLES = {
  'khách chủ chuyến': 'customer:C001',
  'khách khác': 'customer:C002',
  'tài xế D101 (được gán)': 'driver:D101',
  'tài xế D102 (đủ điều kiện)': 'driver:D102',
  'tài xế D201 (khác thành phố)': 'driver:D201',
  'tài xế D103 (khác loại xe)': 'driver:D103',
} as const;
const F = '403 FORBIDDEN';
const N = '404 TRIP_NOT_FOUND';
const X = '404 NOT_FOUND';
const EXPECTED: Record<keyof typeof ROLES, Record<Col, string>> = {
  'khách chủ chuyến': { tao: '201', xemSearching: '200', nhanSearching: F, xemAccepted: '200', arriveAccepted: F, huyAccepted: '200', xoa: X },
  'khách khác': { tao: '422 QUOTE_NOT_FOUND', xemSearching: N, nhanSearching: F, xemAccepted: N, arriveAccepted: F, huyAccepted: N, xoa: X },
  'tài xế D101 (được gán)': { tao: F, xemSearching: '200', nhanSearching: '200', xemAccepted: '200', arriveAccepted: '200', huyAccepted: '200', xoa: X },
  'tài xế D102 (đủ điều kiện)': { tao: F, xemSearching: '200', nhanSearching: '200', xemAccepted: N, arriveAccepted: N, huyAccepted: N, xoa: X },
  'tài xế D201 (khác thành phố)': { tao: F, xemSearching: N, nhanSearching: '422 CITY_MISMATCH', xemAccepted: N, arriveAccepted: N, huyAccepted: N, xoa: X },
  'tài xế D103 (khác loại xe)': { tao: F, xemSearching: N, nhanSearching: '422 VEHICLE_MISMATCH', xemAccepted: N, arriveAccepted: N, huyAccepted: N, xoa: X },
};

async function perform(col: Col, user: string): Promise<string> {
  if (col === 'tao') {
    const q = await quote('C001'); // báo giá của khách chủ chuyến
    const fn = () => request(app).post('/api/trips').set('x-user', user).set('Idempotency-Key', key()).send({ quoteId: q.body.quote.quoteId });
    return outcome(await tracer.as('F02', fn));
  }
  const id = await newTrip();
  if (col === 'xoa') {
    const r = outcome(await request(app).delete(`/api/trips/${id}`).set('x-user', user));
    expect((await view('customer:C001', id)).status).toBe(200); // chuyến vẫn còn
    return r;
  }
  if (col === 'xemSearching') return outcome(await view(user, id));
  if (col === 'nhanSearching') {
    return outcome(await tracer.as('F05', () => request(app).post(`/api/trips/${id}/accept`).set('x-user', user).send({ etaMinutes: 5 })));
  }
  await accept('D101', id); // các cột còn lại: chuyến đã được D101 nhận
  if (col === 'xemAccepted') return outcome(await view(user, id));
  if (col === 'arriveAccepted') return outcome(await tracer.as('F06', () => request(app).post(`/api/trips/${id}/arrive`).set('x-user', user)));
  return outcome(await cancel(user, id));
}

describe('TEST-039 · ma trận vai trò × thao tác trên chuyến', () => {
  const cells = Object.entries(EXPECTED).flatMap(([role, row]) =>
    Object.entries(row).map(([col, expected]) => ({ role, col: col as Col, expected })),
  );
  it.each(cells)('$role · $col ⇒ $expected', async ({ role, col, expected }) => {
    expect(await perform(col, ROLES[role as keyof typeof ROLES])).toBe(expected);
  });
});

describe('TEST-039 · CRUD coverage', () => {
  it('đo số ô của ma trận SPEC đã được thực thi, và thao tác ngầm ngoài ma trận', () => {
    const spec = matrixCells(SPEC_MATRIX);
    const covered = spec.filter((c) => tracer.hits.has(c));
    const uncovered = spec.filter((c) => !tracer.hits.has(c));
    const implicit = [...tracer.hits].filter((c) => !spec.includes(c)).sort();
    console.log(`CRUD coverage = ${covered.length}/${spec.length}\nchưa phủ: ${uncovered.join(', ')}\nngoài ma trận: ${implicit.join(', ')}`);
    expect([covered.length, spec.length]).toEqual([44, 48]);
    expect(uncovered).toEqual(['F08|idempotency_keys|R', 'F08|idempotency_keys|D', 'F09|idempotency_keys|R', 'F09|idempotency_keys|D']);
    expect(implicit).toEqual(['F02|drivers|R', 'F06|drivers|R', 'F07|drivers|R', 'F09|drivers|R']); // xác thực R-API-03
  });
});
