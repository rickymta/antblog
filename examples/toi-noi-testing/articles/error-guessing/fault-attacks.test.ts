/**
 * TEST-051 · Error Guessing — 12 fault attack lên thanh toán VíXanh và huỷ chuyến của lab Tới Nơi.
 * Mỗi ca = một «đoán lỗi» (giả thuyết về error/defect/failure) + cách tấn công + kết quả mong đợi theo SPEC.
 * Test basis: SPEC R-PAY-03…06, R-CANCEL-01…07, R-STATE-02/03, R-IDEM-01…05, R-API-11.
 * Không sửa src/ — chỉ gọi API (supertest) hoặc use case của lab.
 * Chạy: npx vitest run articles/error-guessing
 * Đo thật hết giờ 3 giây (≈ 9 giây/ca): FA_REAL_TIMEOUT=1 npx vitest run articles/error-guessing -t "FA-03R"
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import type { TripRepository } from '../../src/app/ports';
import { cancelByCustomer, driverStep } from '../../src/app/trips';
import { DEFAULT_CONFIG } from '../../src/app/deps';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { vnTime } from '../../src/domain/time';

const C = 'customer:C001';
const D = 'driver:D101';
const FAST_PAYMENT = { timeoutMs: 50, maxRetries: 2 }; // R-PAY-04 rút 3 000 ms còn 50 ms cho test nhanh

function setup(payment = FAST_PAYMENT) {
  const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0)); // 09:00 giờ VN — ngoài cao điểm, không mưa
  const system = createMemorySystem({ clock, config: { payment, tripRateLimit: { max: 0, windowMs: 60_000 } } });
  const app = createApp(system.deps, { logErrors: false });
  let keyNo = 0;
  const key = () => `fa-key-${String(++keyNo).padStart(4, '0')}`;

  /** Khách C001 đặt xe máy 7,4 km ở SG: 12.000 + 4.500 × 7,4 = 45.300 ⇒ 46.000 đ. */
  async function book(paymentMethod: 'cash' | 'vixanh' = 'vixanh'): Promise<string> {
    const q = await request(app).post('/api/quotes').set('x-user', C)
      .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 });
    expect(q.status).toBe(200);
    const t = await request(app).post('/api/trips').set('x-user', C).set('Idempotency-Key', key())
      .send({ quoteId: q.body.quote.quoteId, paymentMethod });
    expect(t.status).toBe(201);
    return t.body.trip.id as string;
  }
  const accept = (id: string, etaMinutes = 5) =>
    request(app).post(`/api/trips/${id}/accept`).set('x-user', D).send({ etaMinutes });
  const step = (id: string, action: 'arrive' | 'pickup' | 'start' | 'complete') =>
    request(app).post(`/api/trips/${id}/${action}`).set('x-user', D).send({});
  const cancel = (id: string, idemKey = key(), body: object = {}) =>
    request(app).post(`/api/trips/${id}/cancel`).set('x-user', C).set('Idempotency-Key', idemKey).send(body);
  async function toInTransit(id: string) {
    expect((await accept(id)).status).toBe(200);
    for (const a of ['arrive', 'pickup', 'start'] as const) expect((await step(id, a)).status).toBe(200);
  }
  return { clock, system, app, book, accept, step, cancel, toInTransit, key };
}

describe('TEST-051 · fault attack — thanh toán khi hoàn thành chuyến (R-PAY-03…06)', () => {
  it('FA-01 · bấm «Hoàn thành» hai lần liên tiếp ⇒ lần 2 bị 409, ví chỉ trừ một lần', async () => {
    const s = setup();
    const id = await s.book();
    await s.toInTransit(id);
    const first = await s.step(id, 'complete');
    const second = await s.step(id, 'complete');
    expect(first.status).toBe(200);
    expect(first.body.trip.paymentStatus).toBe('paid');
    expect(second.status).toBe(409);
    expect(second.body.error.code).toBe('INVALID_TRANSITION');
    expect(s.system.payments.calls).toHaveLength(1);
    expect(s.system.payments.totalCharged()).toBe(46_000);
    console.log(`[FA-01] lần 1: ${first.status} ${first.body.trip.paymentStatus} · lần 2: ${second.status} ${second.body.error.code} · gọi ví ${s.system.payments.calls.length} lần, trừ ${s.system.payments.totalCharged()} đ`);
  });

  it('FA-02 · bấm «Hoàn thành» lần 2 TRONG LÚC ví còn đang trả lời chậm ⇒ 409 ngay, không gọi ví lần nữa', async () => {
    const s = setup({ timeoutMs: 1_000, maxRetries: 2 });
    s.system.payments.latencyMs = 300; // ví chậm 300 ms nhưng vẫn kịp hạn
    const id = await s.book();
    await s.toInTransit(id);
    const [a, b] = await Promise.all([s.step(id, 'complete'), s.step(id, 'complete')]);
    const statuses = [a.status, b.status].sort();
    expect(statuses).toEqual([200, 409]);
    expect(s.system.payments.calls).toHaveLength(1);
    expect(s.system.payments.totalCharged()).toBe(46_000);
    console.log(`[FA-02] hai lệnh đồng thời: ${a.status} / ${b.status} · gọi ví ${s.system.payments.calls.length} lần, trừ ${s.system.payments.totalCharged()} đ`);
  });

  it('FA-03 · cổng hết giờ cả 3 lần ⇒ chuyến vẫn completed, paymentStatus failed, 3 lần gọi CÙNG khoá', async () => {
    const s = setup();
    s.system.payments.setMode('timeout');
    const id = await s.book();
    await s.toInTransit(id);
    const t0 = Date.now();
    const res = await s.step(id, 'complete');
    const ms = Date.now() - t0;
    expect(res.status).toBe(200);
    expect(res.body.trip).toMatchObject({ status: 'completed', paymentStatus: 'failed' });
    expect(s.system.payments.calls.map((c) => c.idempotencyKey)).toEqual([`trip:${id}:fare`, `trip:${id}:fare`, `trip:${id}:fare`]);
    expect(ms).toBeGreaterThanOrEqual(3 * FAST_PAYMENT.timeoutMs);
    console.log(`[FA-03] hết giờ 50 ms × 3: HTTP ${res.status}, ${res.body.trip.status}/${res.body.trip.paymentStatus}, ${s.system.payments.calls.length} lần gọi, phản hồi sau ${ms} ms`);
  });

  it.runIf(process.env.FA_REAL_TIMEOUT === '1')('FA-03R · như FA-03 nhưng hết giờ THẬT 3 giây (cấu hình mặc định)', async () => {
    const s = setup(DEFAULT_CONFIG.payment);
    s.system.payments.setMode('timeout');
    const id = await s.book();
    await s.toInTransit(id);
    const t0 = Date.now();
    const res = await s.step(id, 'complete');
    const ms = Date.now() - t0;
    expect(res.body.trip.paymentStatus).toBe('failed');
    console.log(`[FA-03R] hết giờ 3 000 ms × 3: tài xế chờ phản hồi «Hoàn thành» ${ms} ms`);
  }, 15_000);

  it('FA-04 · hết giờ 2 lần rồi thành công ⇒ paid sau 3 lần gọi, ví trừ đúng một lần', async () => {
    const s = setup();
    s.system.payments.failNext('timeout', 'timeout');
    const id = await s.book();
    await s.toInTransit(id);
    const res = await s.step(id, 'complete');
    expect(res.body.trip.paymentStatus).toBe('paid');
    expect(s.system.payments.calls).toHaveLength(3);
    expect(s.system.payments.chargedKeys()).toEqual([`trip:${id}:fare`]);
    expect(s.system.payments.totalCharged()).toBe(46_000);
    console.log(`[FA-04] timeout, timeout, ok: ${res.body.trip.paymentStatus}, ${s.system.payments.calls.length} lần gọi, trừ ${s.system.payments.totalCharged()} đ`);
  });

  it('FA-05 · ví từ chối ⇒ KHÔNG thử lại, chuyến vẫn completed, paymentStatus failed', async () => {
    const s = setup();
    s.system.payments.setMode('decline');
    const id = await s.book();
    await s.toInTransit(id);
    const res = await s.step(id, 'complete');
    expect(res.status).toBe(200);
    expect(res.body.trip).toMatchObject({ status: 'completed', paymentStatus: 'failed' });
    expect(s.system.payments.calls).toHaveLength(1);
    console.log(`[FA-05] decline: ${res.body.trip.status}/${res.body.trip.paymentStatus}, ${s.system.payments.calls.length} lần gọi`);
  });
});

describe('TEST-051 · fault attack — khách huỷ chuyến (R-CANCEL, R-PAY-05, R-IDEM)', () => {
  it('FA-06 · lỗi tạm thời liên tục khi thu phí huỷ ⇒ chuyến vẫn cancelled, failed, khoá cancel-fee, tài xế về online', async () => {
    const s = setup();
    s.system.payments.setMode('error');
    const id = await s.book();
    expect((await s.accept(id)).status).toBe(200);
    expect((await s.step(id, 'arrive')).status).toBe(200); // arriving ⇒ có phí (R-CANCEL-01)
    const res = await s.cancel(id);
    expect(res.status).toBe(200);
    expect(res.body).toMatchObject({ cancellationFee: 10_000, feeReason: 'charged' });
    expect(res.body.trip).toMatchObject({ status: 'cancelled', paymentStatus: 'failed' });
    expect(s.system.payments.calls.map((c) => c.idempotencyKey)).toEqual(Array(3).fill(`trip:${id}:cancel-fee`));
    const driver = await s.system.deps.drivers.findById('D101');
    expect(driver?.status).toBe('online');
    console.log(`[FA-06] error × 3 khi thu phí huỷ: ${res.body.trip.status}/${res.body.trip.paymentStatus}, ${s.system.payments.calls.length} lần gọi, tài xế ${driver?.status}`);
  });

  it('FA-07 · huỷ ĐÚNG giây thứ 120 sau khi nhận ⇒ có phí 10.000 đ; 119,999 giây ⇒ miễn phí', async () => {
    const rows: string[] = [];
    for (const [label, ms, fee, reason] of [
      ['119,999 s', 119_999, 0, 'grace_period'],
      ['120,000 s', 120_000, 10_000, 'charged'],
    ] as const) {
      const s = setup();
      const id = await s.book();
      expect((await s.accept(id)).status).toBe(200);
      s.clock.advanceMs(ms);
      const res = await s.cancel(id);
      expect(res.body).toMatchObject({ cancellationFee: fee, feeReason: reason });
      expect(s.system.payments.totalCharged()).toBe(fee);
      rows.push(`${label} ⇒ ${res.body.cancellationFee} đ (${res.body.feeReason})`);
    }
    console.log(`[FA-07] ${rows.join(' · ')}`);
  });

  it('FA-08 · huỷ đúng mốc acceptedAt + ETA + 10 phút ⇒ vẫn thu phí; trễ thêm 1 ms ⇒ miễn (driver_late)', async () => {
    const rows: string[] = [];
    for (const [label, extraMs, fee, reason] of [
      ['đúng mốc', 0, 10_000, 'charged'],
      ['mốc + 1 ms', 1, 0, 'driver_late'],
    ] as const) {
      const s = setup();
      const id = await s.book();
      expect((await s.accept(id, 5)).status).toBe(200);
      expect((await s.step(id, 'arrive')).status).toBe(200);
      s.clock.advanceMs((5 + 10) * 60_000 + extraMs);
      const res = await s.cancel(id);
      expect(res.body).toMatchObject({ cancellationFee: fee, feeReason: reason });
      rows.push(`${label} ⇒ ${res.body.cancellationFee} đ (${res.body.feeReason})`);
    }
    console.log(`[FA-08] ETA 5 phút: ${rows.join(' · ')}`);
  });

  it('FA-09 · lý do huỷ 200 / 201 ký tự / 201 ký tự có khoảng trắng cuối / chỉ toàn khoảng trắng', async () => {
    const rows: string[] = [];
    for (const [label, reason, status] of [
      ['200 ký tự', 'a'.repeat(200), 200],
      ['201 ký tự', 'a'.repeat(201), 400],
      ['200 + 1 khoảng trắng', `${'a'.repeat(200)} `, 200],
      ['3 khoảng trắng', '   ', 200],
    ] as const) {
      const s = setup();
      const id = await s.book('cash');
      const res = await s.cancel(id, s.key(), { reason });
      expect(res.status).toBe(status);
      const stored = res.status === 200 ? JSON.stringify(res.body.trip.cancelReason).slice(0, 12) : res.body.error.code;
      rows.push(`${label} ⇒ ${res.status} ${stored}`);
    }
    console.log(`[FA-09] ${rows.join(' · ')}`);
  });

  it('FA-10 · lý do chưa tới 200 ký tự NHÌN THẤY: chữ tổ hợp (NFD), emoji ⇒ kết quả tuỳ cách đếm «ký tự»', async () => {
    const segmenter = new Intl.Segmenter('vi', { granularity: 'grapheme' });
    const visible = (s: string) => [...segmenter.segment(s)].length;
    const codePoints = (s: string) => [...s].length;
    const E_NFC = 'ế'; // «ế» — một code point
    const E_NFD = 'ế'; // «ế» = e + dấu mũ + dấu sắc — ba code point
    const CAR = '\u{1F697}'; // 🚗 — một code point, hai đơn vị UTF-16
    const FAMILY = '\u{1F468}‍\u{1F469}‍\u{1F467}'; // 👨‍👩‍👧 — một hình, năm code point
    const rows: string[] = [];
    for (const [label, reason, status] of [
      ['70 × «ế» NFC', E_NFC.repeat(70), 200],
      ['70 × «ế» NFD', E_NFD.repeat(70), 400],
      ['101 × 🚗', CAR.repeat(101), 200],
      ['41 × 👨‍👩‍👧', FAMILY.repeat(41), 400],
    ] as const) {
      const s = setup();
      const id = await s.book('cash');
      const res = await s.cancel(id, s.key(), { reason });
      expect(res.status).toBe(status);
      rows.push(`${label}: nhìn thấy ${visible(reason)}, code point ${codePoints(reason)}, UTF-16 ${reason.length} ⇒ ${res.status}`);
    }
    console.log(`[FA-10] ${rows.join(' · ')}`);
  });

  it('FA-11 · bấm «Huỷ» hai lần: khác khoá ⇒ 409 và phí thu một lần; cùng khoá ⇒ trả lại kết quả cũ', async () => {
    const s = setup();
    const id = await s.book();
    expect((await s.accept(id)).status).toBe(200);
    expect((await s.step(id, 'arrive')).status).toBe(200);
    const first = await s.cancel(id, 'huy-thiet-bi-1');
    const otherKey = await s.cancel(id, 'huy-thiet-bi-2');
    const sameKey = await s.cancel(id, 'huy-thiet-bi-1');
    expect(first.status).toBe(200);
    expect(otherKey.status).toBe(409);
    expect(otherKey.body.error.code).toBe('INVALID_TRANSITION');
    expect(sameKey.status).toBe(200);
    expect(sameKey.headers['idempotent-replayed']).toBe('true');
    expect(s.system.payments.calls).toHaveLength(1);
    expect(s.system.payments.totalCharged()).toBe(10_000);
    console.log(`[FA-11] lần 1: ${first.status} phí ${first.body.cancellationFee} · khoá khác: ${otherKey.status} ${otherKey.body.error.code} · cùng khoá: ${sameKey.status} replayed=${sameKey.headers['idempotent-replayed']} · ví ${s.system.payments.calls.length} lần, ${s.system.payments.totalCharged()} đ`);
  });

  it('FA-12 · khách bấm «Huỷ» (còn trong 2 phút miễn phí) đúng lúc tài xế bấm «Đang tới đón»', async () => {
    const s = setup();
    const id = await s.book();
    expect((await s.accept(id)).status).toBe(200);
    s.clock.advanceMs(30_000); // 30 giây sau khi nhận — khách tin là được huỷ miễn phí

    // Chen lệnh của tài xế vào GIỮA lúc use case huỷ đọc chuyến và lúc ghi (khoá lạc quan R-STATE-03).
    let interleave: (() => Promise<unknown>) | null = () =>
      driverStep(s.system.deps, { role: 'driver', id: 'D101' }, id, 'arrive');
    const base = s.system.deps.trips;
    const trips = new Proxy(base, {
      get(target, prop, receiver) {
        if (prop === 'findById') {
          return async (tripId: string) => {
            const found = await target.findById(tripId);
            const hook = interleave;
            interleave = null;
            if (hook) await hook();
            return found;
          };
        }
        const value = Reflect.get(target, prop, receiver);
        return typeof value === 'function' ? value.bind(target) : value;
      },
    }) as TripRepository;
    const racingDeps = { ...s.system.deps, trips };

    const error = await cancelByCustomer(racingDeps, { role: 'customer', id: 'C001' }, id).catch((e: unknown) => e as Error & { code?: string });
    expect(error).toMatchObject({ code: 'INVALID_TRANSITION' });
    expect((error as Error).message).toContain('customer_cancel');
    expect((error as Error).message).toContain('arriving');

    // Khách thử lại ngay ⇒ huỷ được, nhưng giờ đã ở arriving nên mất phí.
    const retry = await s.cancel(id);
    expect(retry.body).toMatchObject({ cancellationFee: 10_000, feeReason: 'charged' });
    console.log(`[FA-12] lần 1: ${(error as Error & { code?: string }).code} — «${(error as Error).message}» · thử lại: ${retry.status}, phí ${retry.body.cancellationFee} đ (${retry.body.feeReason})`);
  });
});
