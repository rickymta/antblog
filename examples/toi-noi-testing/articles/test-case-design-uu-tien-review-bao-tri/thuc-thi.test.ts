/**
 * TCD-09 · Chạy THẬT 15 test case R-PAY + R-IDEM qua API (supertest, kho bộ nhớ, đồng hồ giả, ví VíXanh giả),
 * theo đúng test execution schedule risk-based đã sửa phụ thuộc. Thứ tự đăng ký `it` = thứ tự chạy.
 *
 * Bản tự động hoá cho mỗi ca tự dựng dữ liệu của mình (độc lập), kể cả TC-IDEM-007: phụ thuộc vào TC-PAY-006
 * chỉ tồn tại ở test procedure chạy tay trên staging (bài viết mục 6).
 * Hết giờ cổng thanh toán rút còn 20 ms để test nhanh; SPEC R-PAY-04 là 3 giây.
 */
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import type { AppDeps } from '../../src/app/deps';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';
import { RISKS, TEST_CASES } from './bo-test-case';
import { respectDependencies, riskBased } from './uu-tien';

const T0 = vnTime(2026, 11, 4, 9, 0); // thứ Tư 04/11/2026 09:00 giờ VN: ngoài cao điểm
const KHACH = 'customer:C001';
const RIDE = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 }; // 12.000 + 4.500 × 7,4 = 45.300 ⇒ 46.000 đ
const FARE = 46_000;
const CANCEL_FEE_BIKE = 10_000; // R-CANCEL-03

let clock: FixedClock;
let system: LabSystem;
let app: ReturnType<typeof createApp>;

beforeEach(() => {
  clock = new FixedClock(T0);
  system = createMemorySystem({ clock, config: { payment: { timeoutMs: 20, maxRetries: 2 } } });
  app = appWith();
});

const appWith = (overrides: Partial<AppDeps> = {}) => createApp({ ...system.deps, ...overrides }, { logErrors: false });

async function quoteId(a = app): Promise<string> {
  return (await request(a).post('/api/quotes').set('x-user', KHACH).send(RIDE).expect(200)).body.quote.quoteId;
}
function book(key: string | null, body: object, a = app) {
  const req = request(a).post('/api/trips').set('x-user', KHACH);
  return (key ? req.set('Idempotency-Key', key) : req).send(body);
}
async function bookedTrip(paymentMethod: 'cash' | 'vixanh'): Promise<string> {
  return (await book(`dat-${paymentMethod}-0001`, { quoteId: await quoteId(), paymentMethod }).expect(201)).body.trip.id;
}
async function acceptedTrip(paymentMethod: 'cash' | 'vixanh'): Promise<string> {
  const id = await bookedTrip(paymentMethod);
  await request(app).post(`/api/trips/${id}/accept`).set('x-user', 'driver:D101').send({ etaMinutes: 5 }).expect(200);
  return id;
}
/** Đưa chuyến vixanh tới `in_transit`; bước `complete` để test tự gọi. */
async function inTransitTrip(): Promise<string> {
  const id = await acceptedTrip('vixanh');
  for (const step of ['arrive', 'pickup', 'start']) {
    await request(app).post(`/api/trips/${id}/${step}`).set('x-user', 'driver:D101').expect(200);
  }
  return id;
}
const complete = (id: string) => request(app).post(`/api/trips/${id}/complete`).set('x-user', 'driver:D101');
const cancel = (id: string, key: string) =>
  request(app).post(`/api/trips/${id}/cancel`).set('x-user', KHACH).set('Idempotency-Key', key).send({});
const waitingTrips = async () =>
  (await request(app).get('/api/driver/available-trips').set('x-user', 'driver:D101').expect(200)).body.trips;

/** Cổng chờ: giữ một yêu cầu ở giữa handler để tạo tình huống «đang xử lý» một cách tất định. */
function gate() {
  let open!: () => void;
  const opened = new Promise<void>((resolve) => (open = resolve));
  return { open, opened };
}

const IMPL: Record<string, () => Promise<void>> = {
  async 'TC-PAY-001'() {
    const id = await inTransitTrip();
    const res = await complete(id).expect(200);
    expect(res.body.trip).toMatchObject({ status: 'completed', paymentStatus: 'paid', fare: FARE });
    expect(system.payments.calls.map((c) => [c.idempotencyKey, c.amount])).toEqual([[`trip:${id}:fare`, FARE]]);
    expect(system.payments.totalCharged()).toBe(FARE);
  },
  async 'TC-PAY-002'() {
    const id = await inTransitTrip();
    system.payments.failNext('timeout', 'timeout');
    const res = await complete(id).expect(200);
    expect(res.body.trip.paymentStatus).toBe('paid');
    expect(system.payments.calls.map((c) => c.idempotencyKey)).toEqual(Array(3).fill(`trip:${id}:fare`));
    expect(system.payments.totalCharged()).toBe(FARE);
  },
  async 'TC-PAY-003'() {
    const id = await inTransitTrip();
    system.payments.setMode('error');
    const res = await complete(id).expect(200);
    expect(res.body.trip).toMatchObject({ status: 'completed', paymentStatus: 'failed' });
    expect(system.payments.calls).toHaveLength(3); // 1 lần gọi + tối đa 2 lần thử lại
    expect(system.payments.totalCharged()).toBe(0);
  },
  async 'TC-PAY-004'() {
    const id = await inTransitTrip();
    system.payments.setMode('decline');
    const res = await complete(id).expect(200);
    expect(res.body.trip).toMatchObject({ status: 'completed', paymentStatus: 'failed' });
    expect(system.payments.calls).toHaveLength(1);
  },
  async 'TC-PAY-005'() {
    const id = await inTransitTrip();
    system.payments.failNext('lost_response');
    const res = await complete(id).expect(200);
    expect(res.body.trip.paymentStatus).toBe('paid');
    expect(system.payments.calls.map((c) => c.idempotencyKey)).toEqual([`trip:${id}:fare`, `trip:${id}:fare`]);
    expect(system.payments.totalCharged()).toBe(FARE); // đã trừ ở lần 1, lần 2 không trừ thêm
  },
  async 'TC-PAY-006'() {
    const id = await acceptedTrip('vixanh');
    clock.advanceMinutes(5);
    const res = await cancel(id, 'huy-0001').expect(200);
    expect(res.body).toMatchObject({ cancellationFee: CANCEL_FEE_BIKE, feeReason: 'charged' });
    expect(res.body.trip).toMatchObject({ status: 'cancelled', paymentStatus: 'paid' });
    expect(system.payments.calls.map((c) => [c.idempotencyKey, c.amount])).toEqual([[`trip:${id}:cancel-fee`, CANCEL_FEE_BIKE]]);
  },
  async 'TC-PAY-007'() {
    const id = await acceptedTrip('cash');
    clock.advanceMinutes(5);
    const res = await cancel(id, 'huy-0001').expect(200);
    expect(res.body).toMatchObject({ cancellationFee: CANCEL_FEE_BIKE, trip: { paymentStatus: 'cash' } });
    expect(system.payments.calls).toHaveLength(0);
  },
  async 'TC-PAY-008'() {
    const id = await bookedTrip('vixanh');
    const res = await cancel(id, 'huy-0001').expect(200);
    expect(res.body).toMatchObject({ cancellationFee: 0, feeReason: 'searching', trip: { paymentStatus: 'none' } });
    expect(system.payments.calls).toHaveLength(0);
  },
  async 'TC-IDEM-001'() {
    const id = await quoteId();
    for (const key of [null, 'abc-123', 'dat/chuyen-01']) {
      const res = await book(key, { quoteId: id });
      expect([res.status, res.body.error.code]).toEqual([400, 'IDEMPOTENCY_KEY_REQUIRED']);
    }
    expect(await waitingTrips()).toHaveLength(0);
  },
  async 'TC-IDEM-002'() {
    const id = await quoteId();
    const first = await book('dat-chuyen-0001', { quoteId: id, paymentMethod: 'cash' }).expect(201);
    const again = await book('dat-chuyen-0001', { paymentMethod: 'cash', quoteId: id }).expect(201);
    expect(again.headers['idempotent-replayed']).toBe('true');
    expect(again.body).toEqual(first.body);
    expect(await waitingTrips()).toHaveLength(1);
  },
  async 'TC-IDEM-003'() {
    const id = await quoteId();
    await book('dat-chuyen-0001', { quoteId: id, paymentMethod: 'cash' }).expect(201);
    const res = await book('dat-chuyen-0001', { quoteId: id, paymentMethod: 'vixanh' });
    expect([res.status, res.body.error.code]).toEqual([422, 'IDEMPOTENCY_KEY_REUSED']);
    const trips = await waitingTrips();
    expect(trips.map((t: { paymentMethod: string }) => t.paymentMethod)).toEqual(['cash']);
  },
  async 'TC-IDEM-004'() {
    const hold = gate();
    const entered = gate();
    const real = system.deps.quotes;
    const quotes = {
      save: real.save.bind(real),
      markUsed: real.markUsed.bind(real),
      findById: async (qid: string) => {
        entered.open();
        await hold.opened;
        return real.findById(qid);
      },
    };
    const slow = appWith({ quotes });
    const id = await quoteId(slow);
    const first = book('dat-chuyen-0001', { quoteId: id }, slow).then((r) => r);
    await entered.opened;
    const second = await book('dat-chuyen-0001', { quoteId: id }, slow);
    expect([second.status, second.body.error.code]).toEqual([409, 'IDEMPOTENCY_IN_PROGRESS']);
    hold.open();
    expect((await first).status).toBe(201);
  },
  async 'TC-IDEM-005'() {
    const id = await quoteId();
    clock.advanceMinutes(5); // đúng bằng expiresAt ⇒ đã hết hạn (R-QUOTE-01)
    const first = await book('dat-chuyen-0001', { quoteId: id });
    const again = await book('dat-chuyen-0001', { quoteId: id });
    expect([first.status, first.body.error.code]).toEqual([422, 'QUOTE_EXPIRED']);
    expect(again.status).toBe(422);
    expect(again.headers['idempotent-replayed']).toBe('true');
    expect(again.body).toEqual(first.body);
  },
  async 'TC-IDEM-006'() {
    // Kho chuyến lỗi đúng một lần ⇒ 500; R-IDEM-05: 5xx không lưu, client được thử lại cùng khoá.
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
    const id = await quoteId(flaky);
    expect((await book('dat-chuyen-0500', { quoteId: id }, flaky)).status).toBe(500);
    const retry = await book('dat-chuyen-0500', { quoteId: id }, flaky);
    expect(retry.headers['idempotent-replayed']).toBeUndefined();
    expect(retry.status).toBe(201); // thực tế 409 QUOTE_ALREADY_USED — defect DR-050-01 (TEST-050)
  },
  async 'TC-IDEM-007'() {
    const id = await acceptedTrip('vixanh'); // tự dựng lại tiền điều kiện của TC-PAY-006
    clock.advanceMinutes(5);
    const first = await cancel(id, 'huy-0001').expect(200);
    const again = await cancel(id, 'huy-0001').expect(200);
    expect(again.headers['idempotent-replayed']).toBe('true');
    expect(again.body).toEqual(first.body);
    expect(system.payments.calls).toHaveLength(1);
    expect(system.payments.totalCharged()).toBe(CANCEL_FEE_BIKE);
  },
};

/** Ca đang ghi nhận defect đã biết: chạy như it.fails (xanh khi defect còn). SHOW_RED=1 để thấy đỏ thật. */
const KNOWN_DEFECT: Record<string, string> = { 'TC-IDEM-006': 'DR-050-01' };

export const SCHEDULE = respectDependencies(riskBased(TEST_CASES, RISKS), TEST_CASES);

describe('Thực thi theo lịch risk-based (đã sửa phụ thuộc)', () => {
  SCHEDULE.forEach((id, i) => {
    const tc = TEST_CASES.find((t) => t.id === id)!;
    const name = `#${String(i + 1).padStart(2, '0')} ${id} · ${tc.reqs.join(', ')} · ${tc.title}`;
    const defect = KNOWN_DEFECT[id];
    if (defect && !process.env.SHOW_RED) it.fails(`${name} [defect ${defect}]`, IMPL[id]!);
    else it(name, IMPL[id]!);
  });

  it('mọi test case trong dữ liệu đều có bản cài đặt', () => {
    expect(Object.keys(IMPL).sort()).toEqual(TEST_CASES.map((t) => t.id).sort());
  });
});
