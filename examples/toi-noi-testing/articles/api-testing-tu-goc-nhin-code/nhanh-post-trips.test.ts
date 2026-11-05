/**
 * TEST-050 · API testing từ góc nhìn code — phủ từng nhánh của handler POST /api/trips.
 * Nhánh đọc từ src/api/app.ts (chuỗi middleware), src/api/idempotency.ts, src/app/trips.ts (createTrip).
 * Lệnh: npx vitest run articles/api-testing-tu-goc-nhin-code
 */
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import type { AppDeps } from '../../src/app/deps';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';

const C001 = 'customer:C001';
const ride = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 };
const itRed = process.env.SHOW_RED ? it : it.fails; // ca ghi nhận defect: xanh khi defect còn, SHOW_RED=1 để thấy failure thật

/** Cổng chờ dùng để giữ một yêu cầu ở giữa handler (tạo nhánh khó gây ra một cách tất định). */
function gate() {
  let open!: () => void;
  const opened = new Promise<void>((resolve) => (open = resolve));
  return { open, opened };
}

let clock: FixedClock;
let system: LabSystem;

function appWith(overrides: Partial<AppDeps> = {}) {
  return createApp({ ...system.deps, ...overrides }, { logErrors: false });
}

async function quoteId(app: ReturnType<typeof createApp>, user = C001, body: object = ride): Promise<string> {
  const res = await request(app).post('/api/quotes').set('x-user', user).send(body).expect(200);
  return res.body.quote.quoteId;
}

function book(app: ReturnType<typeof createApp>, key: string | null, body: object, user: string | null = C001) {
  let req = request(app).post('/api/trips');
  if (user) req = req.set('x-user', user);
  if (key) req = req.set('Idempotency-Key', key);
  return req.send(body);
}

const errorOf = (res: request.Response) => ({ status: res.status, code: res.body?.error?.code });

beforeEach(() => {
  clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
  system = createMemorySystem({ clock });
});

describe('H0 · body parser (express.json) — chạy TRƯỚC mọi middleware khác', () => {
  it('N01 · thân > 10 KB ⇒ 413 PAYLOAD_TOO_LARGE', async () => {
    const res = await book(appWith(), 'key-n01-0000', { quoteId: 'x'.repeat(11_000) });
    expect(errorOf(res)).toEqual({ status: 413, code: 'PAYLOAD_TOO_LARGE' });
  });

  it('N02 · JSON hỏng, CÓ x-user ⇒ 400 INVALID_JSON', async () => {
    const res = await request(appWith()).post('/api/trips').set('x-user', C001).set('content-type', 'application/json').send('{"quoteId":');
    expect(errorOf(res)).toEqual({ status: 400, code: 'INVALID_JSON' });
  });

  itRed('N02b · DEFECT? JSON hỏng, KHÔNG có x-user ⇒ R-VALID-02 đòi 401 trước (thực tế 400 INVALID_JSON)', async () => {
    const res = await request(appWith()).post('/api/trips').set('content-type', 'application/json').send('{"quoteId":');
    expect(errorOf(res)).toEqual({ status: 401, code: 'UNAUTHENTICATED' });
  });
});

describe('H1–H3 · authenticate → only(customer) → rateLimitTrips', () => {
  it.each([
    ['N03 thiếu header', null],
    ['N04 sai cú pháp', 'customer C001'],
    ['N05 id không tồn tại', 'customer:C999'],
  ])('%s ⇒ 401 UNAUTHENTICATED', async (_name, user) => {
    expect(errorOf(await book(appWith(), 'key-n03-0000', { quoteId: 'q' }, user))).toEqual({ status: 401, code: 'UNAUTHENTICATED' });
  });

  it('N06 · tài xế gọi endpoint của khách ⇒ 403 FORBIDDEN', async () => {
    expect(errorOf(await book(appWith(), 'key-n06-0000', { quoteId: 'q' }, 'driver:D101'))).toEqual({ status: 403, code: 'FORBIDDEN' });
  });

  it('N07 · yêu cầu thứ 6 trong 60 giây ⇒ 429 + Retry-After; 5 yêu cầu trước bị 400 vẫn được đếm', async () => {
    const app = appWith();
    for (let i = 0; i < 5; i++) expect((await book(app, null, { quoteId: 'q' })).status).toBe(400);
    const res = await book(app, null, { quoteId: 'q' });
    expect(errorOf(res)).toEqual({ status: 429, code: 'RATE_LIMITED' });
    expect(res.headers['retry-after']).toBe('60');
  });

  it('N08 · lỗi kho khi tra khách lúc xác thực ⇒ 500 INTERNAL_ERROR, không phải 401', async () => {
    const customers = { findById: async () => { throw new Error('ECONNRESET customers'); } };
    const res = await book(appWith({ customers }), 'key-n08-0000', { quoteId: 'q' });
    expect(errorOf(res)).toEqual({ status: 500, code: 'INTERNAL_ERROR' });
    expect(JSON.stringify(res.body)).not.toContain('ECONNRESET');
  });
});

describe('H4 · runIdempotent', () => {
  it.each([
    ['N09 thiếu khoá', null],
    ['N10 khoá 7 ký tự', 'abc-123'],
    ['N11 khoá có ký tự lạ', 'key/0001-x'],
  ])('%s ⇒ 400 IDEMPOTENCY_KEY_REQUIRED', async (_name, key) => {
    expect(errorOf(await book(appWith(), key, { quoteId: 'q' }))).toEqual({ status: 400, code: 'IDEMPOTENCY_KEY_REQUIRED' });
  });

  it('N12 · 400 VALIDATION_ERROR được LƯU: gửi lại ⇒ 400 replayed; sửa thân cùng khoá ⇒ 422 IDEMPOTENCY_KEY_REUSED', async () => {
    const app = appWith();
    const first = await book(app, 'key-n12-0000', {});
    expect(errorOf(first)).toEqual({ status: 400, code: 'VALIDATION_ERROR' });
    const again = await book(app, 'key-n12-0000', {});
    expect(again.status).toBe(400);
    expect(again.headers['idempotent-replayed']).toBe('true');
    const fixed = await book(app, 'key-n12-0000', { quoteId: await quoteId(app) });
    expect(errorOf(fixed)).toEqual({ status: 422, code: 'IDEMPOTENCY_KEY_REUSED' });
  });

  it('N13 · yêu cầu thứ hai cùng khoá tới khi yêu cầu đầu còn trong handler ⇒ 409 IDEMPOTENCY_IN_PROGRESS', async () => {
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
    const app = appWith({ quotes });
    const id = await quoteId(app);
    const first = book(app, 'key-n13-0000', { quoteId: id }).then((r) => r);
    await entered.opened; // yêu cầu 1 đã giữ chỗ khoá và đang chờ trong createTrip
    const second = await book(app, 'key-n13-0000', { quoteId: id });
    expect(errorOf(second)).toEqual({ status: 409, code: 'IDEMPOTENCY_IN_PROGRESS' });
    hold.open();
    expect((await first).status).toBe(201);
  });

  it('N14 · thành công 201; gửi lại cùng khoá ⇒ 201, cùng chuyến, Idempotent-Replayed: true', async () => {
    const app = appWith();
    const id = await quoteId(app);
    const first = await book(app, 'key-n14-0000', { quoteId: id });
    const again = await book(app, 'key-n14-0000', { quoteId: id });
    expect([first.status, again.status]).toEqual([201, 201]);
    expect(again.body.trip.id).toBe(first.body.trip.id);
    expect(again.headers['idempotent-replayed']).toBe('true');
  });
});

describe('H5 · createTrip (R-QUOTE-03, R-VOUCHER-07)', () => {
  it('N15 · báo giá của khách khác ⇒ 422 QUOTE_NOT_FOUND', async () => {
    const app = appWith();
    const other = await quoteId(app, 'customer:C002');
    expect(errorOf(await book(app, 'key-n15-0000', { quoteId: other }))).toEqual({ status: 422, code: 'QUOTE_NOT_FOUND' });
  });

  it('N16 · đúng bằng expiresAt (5 phút) ⇒ 422 QUOTE_EXPIRED', async () => {
    const app = appWith();
    const id = await quoteId(app);
    clock.advanceMinutes(5);
    expect(errorOf(await book(app, 'key-n16-0000', { quoteId: id }))).toEqual({ status: 422, code: 'QUOTE_EXPIRED' });
  });

  it('N17 · báo giá đã dùng, khoá khác ⇒ 409 QUOTE_ALREADY_USED (nhánh usedByTripId)', async () => {
    const app = appWith();
    const id = await quoteId(app);
    await book(app, 'key-n17-0001', { quoteId: id }).expect(201);
    expect(errorOf(await book(app, 'key-n17-0002', { quoteId: id }))).toEqual({ status: 409, code: 'QUOTE_ALREADY_USED' });
  });

  it('N18 · hai khoá khác nhau cùng đọc báo giá trước khi ai đánh dấu ⇒ một 201, một 409 (nhánh markUsed = false)', async () => {
    const real = system.deps.quotes;
    const bothRead = gate();
    let reads = 0;
    let markCalls = 0;
    const quotes = {
      save: real.save.bind(real),
      findById: async (id: string) => {
        const quote = await real.findById(id);
        if (++reads === 2) bothRead.open();
        await bothRead.opened;
        return quote;
      },
      markUsed: async (q: string, t: string) => {
        markCalls++;
        return real.markUsed(q, t);
      },
    };
    const app = appWith({ quotes });
    const id = await quoteId(app);
    const results = await Promise.all([
      book(app, 'key-n18-0001', { quoteId: id }).then((r) => r),
      book(app, 'key-n18-0002', { quoteId: id }).then((r) => r),
    ]);
    expect(results.map(errorOf).sort((a, b) => a.status - b.status)).toEqual([
      { status: 201, code: undefined },
      { status: 409, code: 'QUOTE_ALREADY_USED' },
    ]);
    expect(markCalls).toBe(2); // cả hai đều qua kiểm usedByTripId ⇒ người thua bị chặn ở markUsed
  });

  it('N19 · GIAM20 (1 lần/ngày): hai báo giá cùng mã, đặt chuyến thứ hai ⇒ 422 VOUCHER_USAGE_LIMIT', async () => {
    const app = appWith();
    const body = { ...ride, voucherCode: 'GIAM20' };
    const a = await quoteId(app, C001, body);
    const b = await quoteId(app, C001, body);
    await book(app, 'key-n19-0001', { quoteId: a }).expect(201);
    expect(errorOf(await book(app, 'key-n19-0002', { quoteId: b }))).toEqual({ status: 422, code: 'VOUCHER_USAGE_LIMIT' });
  });
});

describe('H6 · lỗi hạ tầng trong handler: 5xx không được lưu, khoá được nhả (R-IDEM-05)', () => {
  function failingTripsOnce() {
    const real = system.deps.trips;
    let failed = false;
    return new Proxy(real, {
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
  }

  it('N20 · kho chuyến ném lỗi ⇒ 500 INTERNAL_ERROR, thân không lộ chi tiết; lần gửi lại KHÔNG phải replay', async () => {
    const app = appWith({ trips: failingTripsOnce() });
    const id = await quoteId(app);
    const first = await book(app, 'key-n20-0000', { quoteId: id });
    expect(first.status).toBe(500);
    expect(first.body).toEqual({ error: { code: 'INTERNAL_ERROR', message: 'Lỗi hệ thống, vui lòng thử lại sau.' } });
    const retry = await book(app, 'key-n20-0000', { quoteId: id });
    expect(retry.headers['idempotent-replayed']).toBeUndefined();
  });

  itRed('N21 · DEFECT: sau 500, gửi lại cùng khoá phải tạo được chuyến (thực tế 409 QUOTE_ALREADY_USED — báo giá đã bị đánh dấu)', async () => {
    const app = appWith({ trips: failingTripsOnce() });
    const id = await quoteId(app);
    expect((await book(app, 'key-n21-0000', { quoteId: id })).status).toBe(500);
    const retry = await book(app, 'key-n21-0000', { quoteId: id });
    expect(errorOf(retry)).toEqual({ status: 201, code: undefined });
  });
});
