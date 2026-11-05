/**
 * TEST-035 · Use Case Testing và Scenario-based Testing.
 * Mỗi test case đi trọn MỘT kịch bản của use case UC-01 «Khách đặt xe máy» qua API HTTP của lab
 * (supertest, kho trong bộ nhớ, đồng hồ giả FixedClock, ví VíXanh giả). Cuối lần chạy in
 * scenario coverage và simple loop coverage của vòng «tài xế huỷ ⇒ tìm lại».
 *
 * Chạy: npx vitest run articles/use-case-testing-scenario-based-testing --reporter=verbose
 */
import request from 'supertest';
import { afterAll, beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';
import { LOOP_ITEMS, scenarioCoverage, UC01_SCENARIOS, type LoopItem } from './kich-ban';

const T0 = vnTime(2026, 11, 4, 9, 0); // thứ Tư 04/11/2026, 09:00 giờ VN — ngoài cao điểm, không mưa
const KHACH = 'customer:C001';
const RIDE = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 };

let clock: FixedClock;
let system: LabSystem;
let app: ReturnType<typeof createApp>;
let key = 0;

const executed = new Set<string>();
const loops = new Set<LoopItem>();

beforeEach(() => {
  clock = new FixedClock(T0);
  system = createMemorySystem({ clock });
  app = createApp(system.deps, {
    logErrors: false,
    testControl: { reset: () => system.reset(), setNow: (d) => clock.set(d), weather: system.weather, payments: system.payments },
  });
});

/** Dời đồng hồ tới T0 + m phút s giây. */
const at = (m: number, s = 0) => clock.set(new Date(T0.getTime() + (m * 60 + s) * 1000));
const quote = (body: object) => request(app).post('/api/quotes').set('x-user', KHACH).send(body);
const book = (quoteId: string, paymentMethod = 'cash') =>
  request(app).post('/api/trips').set('x-user', KHACH).set('Idempotency-Key', `dat-chuyen-${++key}`).send({ quoteId, paymentMethod });
const view = (tripId: string) => request(app).get(`/api/trips/${tripId}`).set('x-user', KHACH);
const customerCancel = (tripId: string) =>
  request(app).post(`/api/trips/${tripId}/cancel`).set('x-user', KHACH).set('Idempotency-Key', `khach-huy-${++key}`).send({});
const expireSearches = () => request(app).post('/api/test/expire-searches');

function driver(id: string) {
  const user = `driver:${id}`;
  return {
    available: () => request(app).get('/api/driver/available-trips').set('x-user', user),
    accept: (tripId: string, etaMinutes = 5) => request(app).post(`/api/trips/${tripId}/accept`).set('x-user', user).send({ etaMinutes }),
    step: (tripId: string, path: 'arrive' | 'pickup' | 'start' | 'complete') => request(app).post(`/api/trips/${tripId}/${path}`).set('x-user', user),
    cancel: (tripId: string) =>
      request(app).post(`/api/trips/${tripId}/cancel`).set('x-user', user).set('Idempotency-Key', `tai-xe-huy-${++key}`).send({}),
  };
}

/** Bước 1–3: báo giá rồi đặt chuyến; trả id chuyến đang `searching`. */
async function bookTrip(body: object = RIDE, paymentMethod = 'cash'): Promise<string> {
  const q = await quote(body).expect(200);
  const t = await book(q.body.quote.quoteId, paymentMethod).expect(201);
  expect(t.body.trip.status).toBe('searching');
  return t.body.trip.id as string;
}

/** Bước 6–9: tài xế được gán đi hết vòng đời; trả chuyến sau bước «complete». */
async function driveToCompletion(tripId: string, driverId: string) {
  const d = driver(driverId);
  for (const path of ['arrive', 'pickup', 'start'] as const) await d.step(tripId, path).expect(200);
  return (await d.step(tripId, 'complete').expect(200)).body.trip;
}

describe('UC-01 «Khách đặt xe máy» — mỗi test case đi trọn một kịch bản', () => {
  it('TC-01 · KB-01 luồng chính: GIAM20, VíXanh, D101 chở tới nơi ⇒ completed, thu đúng 37.000 đ một lần', async () => {
    const q = await quote({ ...RIDE, voucherCode: 'GIAM20' }).expect(200); // bước 1–2
    expect(q.body.quote).toMatchObject({ total: 37_000, discount: 9_060, peak: false, raining: false });
    expect(new Date(q.body.quote.expiresAt).getTime() - T0.getTime()).toBe(5 * 60_000);

    at(1);
    const t = await book(q.body.quote.quoteId, 'vixanh').expect(201); // bước 3
    const tripId = t.body.trip.id as string;
    expect(t.body.trip).toMatchObject({ status: 'searching', fare: 37_000, paymentStatus: 'pending' });

    at(2);
    const list = await driver('D101').available().expect(200); // bước 4
    expect(list.body.trips.map((x: { id: string }) => x.id)).toContain(tripId);
    expect((await driver('D101').accept(tripId).expect(200)).body.trip.status).toBe('accepted'); // bước 5

    at(20);
    const done = await driveToCompletion(tripId, 'D101'); // bước 6–9
    expect(done).toMatchObject({ status: 'completed', paymentStatus: 'paid' });
    expect((await view(tripId).expect(200)).body.trip).toMatchObject({ status: 'completed', driverId: 'D101' });
    expect(system.payments.totalCharged()).toBe(37_000);
    executed.add('KB-01');
    loops.add('0 lần');
  });

  it('TC-02 · KB-02 voucher hết hạn ⇒ 422 VOUCHER_EXPIRED, bỏ voucher, báo giá lại 46.000 đ rồi đi tiếp tới completed', async () => {
    const bad = await quote({ ...RIDE, voucherCode: 'XMAS50' });
    expect(bad.status).toBe(422);
    expect(bad.body.error.code).toBe('VOUCHER_EXPIRED');

    const q = await quote(RIDE).expect(200);
    expect(q.body.quote).toMatchObject({ total: 46_000, discount: 0, voucherCode: null });
    const tripId = (await book(q.body.quote.quoteId).expect(201)).body.trip.id as string;
    await driver('D101').accept(tripId).expect(200);
    expect((await driveToCompletion(tripId, 'D101')).status).toBe('completed');
    executed.add('KB-02');
  });

  it('TC-03 · KB-03 đặt đúng lúc báo giá hết hạn (5:00) ⇒ 422 QUOTE_EXPIRED, báo giá lại thì đặt được', async () => {
    const old = await quote(RIDE).expect(200);
    at(5); // R-QUOTE-01: now = expiresAt là đã hết hạn
    const late = await book(old.body.quote.quoteId);
    expect(late.status).toBe(422);
    expect(late.body.error.code).toBe('QUOTE_EXPIRED');

    const tripId = await bookTrip();
    await driver('D101').accept(tripId).expect(200);
    expect((await driveToCompletion(tripId, 'D101')).status).toBe('completed');
    executed.add('KB-03');
  });

  it('TC-04 · KB-04 lặp 1 lần: D101 nhận rồi huỷ ⇒ searching, D102 nhận và hoàn thành, D101 nhận được chuyến khác', async () => {
    const tripId = await bookTrip();
    at(1);
    await driver('D101').accept(tripId).expect(200);
    at(2);
    const reopened = (await driver('D101').cancel(tripId).expect(200)).body.trip;
    expect(reopened).toMatchObject({ status: 'searching', driverId: null, acceptedAt: null, etaMinutes: null });

    at(3);
    await driver('D102').accept(tripId).expect(200);
    expect(await driveToCompletion(tripId, 'D102')).toMatchObject({ status: 'completed', driverId: 'D102' });
    // D101 đã về online (R-ACCEPT-05): nhận được chuyến khác, không bị DRIVER_BUSY
    expect((await driver('D101').accept(await bookTrip()).expect(200)).body.trip.status).toBe('accepted');
    executed.add('KB-04');
    loops.add('1 lần');
  });

  it('TC-05 · KB-04 lặp 2 lần: D101 huỷ, D102 huỷ, D101 nhận lại và hoàn thành', async () => {
    const tripId = await bookTrip();
    for (const [minute, id] of [[1, 'D101'], [2, 'D102']] as const) {
      at(minute);
      await driver(id).accept(tripId).expect(200);
      expect((await driver(id).cancel(tripId).expect(200)).body.trip.status).toBe('searching');
    }
    at(3);
    await driver('D101').accept(tripId).expect(200);
    expect(await driveToCompletion(tripId, 'D101')).toMatchObject({ status: 'completed', driverId: 'D101' });
    loops.add('nhiều lần');
  });

  it('TC-06a · KB-05 tại 4:59 kể từ lúc đặt ⇒ bước quét chưa huỷ (BVA trong kịch bản)', async () => {
    const tripId = await bookTrip();
    at(4, 59);
    expect((await expireSearches().expect(200)).body.expired).toBe(0);
    expect((await view(tripId)).body.trip.status).toBe('searching');
  });

  it('TC-06b · KB-05 tại 5:00 ⇒ hệ thống huỷ lý do no_driver, phí 0 đ, không thu gì', async () => {
    const tripId = await bookTrip();
    at(5);
    expect((await expireSearches().expect(200)).body.expired).toBe(1);
    expect((await view(tripId)).body.trip).toMatchObject({
      status: 'cancelled', cancelledBy: 'system', cancelReason: 'no_driver', cancellationFee: 0, paymentStatus: 'none',
    });
    executed.add('KB-05');
  });

  it('TC-07a · KB-06 khách huỷ 1:59 sau khi D101 nhận ⇒ 0 đ, grace_period', async () => {
    const tripId = await bookTrip();
    await driver('D101').accept(tripId).expect(200);
    at(1, 59);
    expect((await customerCancel(tripId).expect(200)).body).toMatchObject({ cancellationFee: 0, feeReason: 'grace_period' });
  });

  it('TC-07b · KB-06 khách huỷ đúng 2:00 sau khi D101 nhận ⇒ 10.000 đ, charged; D101 rảnh lại', async () => {
    const tripId = await bookTrip();
    await driver('D101').accept(tripId).expect(200);
    at(2);
    const res = (await customerCancel(tripId).expect(200)).body;
    expect(res).toMatchObject({ cancellationFee: 10_000, feeReason: 'charged' });
    expect(res.trip).toMatchObject({ status: 'cancelled', cancelledBy: 'customer', paymentStatus: 'cash' });
    expect((await driver('D101').accept(await bookTrip()).expect(200)).body.trip.status).toBe('accepted'); // không kẹt busy
    executed.add('KB-06');
  });

  it('TC-08 · ghép KB-04 → KB-05: tài xế huỷ lúc 4:30 đặt lại mốc tìm ⇒ 9:29 chưa huỷ, 9:30 mới huỷ', async () => {
    const tripId = await bookTrip();
    at(4);
    await driver('D101').accept(tripId).expect(200);
    at(4, 30);
    await driver('D101').cancel(tripId).expect(200); // R-STATE-04: searchingSince = 4:30
    at(9, 29);
    expect((await expireSearches()).body.expired).toBe(0);
    at(9, 30);
    expect((await expireSearches()).body.expired).toBe(1);
    expect((await view(tripId)).body.trip).toMatchObject({ status: 'cancelled', cancelReason: 'no_driver' });
  });

  it('TC-09 · ghép KB-04 → KB-06: phí huỷ tính theo lần nhận MỚI — 4:00 sau D101 nhưng 1:00 sau D102 ⇒ 0 đ', async () => {
    const tripId = await bookTrip();
    await driver('D101').accept(tripId).expect(200);
    at(3);
    await driver('D101').cancel(tripId).expect(200);
    await driver('D102').accept(tripId).expect(200);
    at(4);
    expect((await customerCancel(tripId).expect(200)).body).toMatchObject({ cancellationFee: 0, feeReason: 'grace_period' });
  });
});

afterAll(() => {
  const c = scenarioCoverage(UC01_SCENARIOS, executed);
  const loopMissing = LOOP_ITEMS.filter((item) => !loops.has(item));
  console.log(
    [
      `Scenario coverage UC-01: ${c.executed}/${c.identified} = ${c.percent.toFixed(1)}% · chưa chạy: ${c.missing.join(', ') || '—'}`,
      `Simple loop coverage (tài xế huỷ ⇒ tìm lại): ${loops.size}/${LOOP_ITEMS.length} · chưa phủ: ${loopMissing.join(', ') || '—'}`,
    ].join('\n'),
  );
});
