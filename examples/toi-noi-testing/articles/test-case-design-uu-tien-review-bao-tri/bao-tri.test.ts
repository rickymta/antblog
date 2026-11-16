/**
 * TCD-09 · Bảo trì bộ test khi SPEC đổi: giả định bài tập R-PAY-04 v1.2 «thử lại tối đa 3 lần» (v1.1: 2 lần).
 * Kết quả mong đợi viết theo tham số của SPEC ở MỘT chỗ (SPEC_RETRIES) — đổi SPEC là đổi một hằng,
 * rồi để bộ test chỉ ra ca nào đổi kết quả mong đợi và ca biên nào còn thiếu.
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import type { FakeFailureMode } from '../../src/infra/fake-payment-gateway';

type Outcome = { paymentStatus: 'paid' | 'failed'; calls: number };

/** Oracle viết từ câu chữ R-PAY-04 (không gọi mã sản phẩm): gọi tới khi thành công, bị từ chối, hoặc hết lượt. */
function oracle(retries: number, failures: FakeFailureMode[], thenMode: FakeFailureMode): Outcome {
  const maxCalls = 1 + retries;
  for (let call = 1; call <= maxCalls; call++) {
    const mode = failures[call - 1] ?? thenMode;
    if (mode === 'none') return { paymentStatus: 'paid', calls: call };
    if (mode === 'decline') return { paymentStatus: 'failed', calls: call };
    // timeout, error, lost_response ⇒ thử lại cùng khoá (lost_response đã trừ tiền nhưng client không biết)
  }
  return { paymentStatus: 'failed', calls: maxCalls };
}

/** Các ca R-PAY-04 của bộ v1.1, cộng ca biên mới cần thêm cho v1.2. */
const CASES: { id: string; failures: FakeFailureMode[]; thenMode: FakeFailureMode; since: 'v1.1' | 'v1.2' }[] = [
  { id: 'TC-PAY-002', failures: ['timeout', 'timeout'], thenMode: 'none', since: 'v1.1' },
  { id: 'TC-PAY-003', failures: [], thenMode: 'error', since: 'v1.1' },
  { id: 'TC-PAY-004', failures: [], thenMode: 'decline', since: 'v1.1' },
  { id: 'TC-PAY-005', failures: ['lost_response'], thenMode: 'none', since: 'v1.1' },
  { id: 'TC-PAY-010', failures: ['timeout', 'timeout', 'timeout'], thenMode: 'none', since: 'v1.2' },
];

/** Chạy một ca qua API trên hệ thống cấu hình `systemRetries` lần thử lại. */
async function run(systemRetries: number, failures: FakeFailureMode[], thenMode: FakeFailureMode): Promise<Outcome> {
  const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
  const system = createMemorySystem({ clock, config: { payment: { timeoutMs: 20, maxRetries: systemRetries } } });
  const app = createApp(system.deps, { logErrors: false });
  const q = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
    .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 }).expect(200);
  const trip = await request(app).post('/api/trips').set('x-user', 'customer:C001').set('Idempotency-Key', 'dat-chuyen-0001')
    .send({ quoteId: q.body.quote.quoteId, paymentMethod: 'vixanh' }).expect(201);
  const id = trip.body.trip.id;
  await request(app).post(`/api/trips/${id}/accept`).set('x-user', 'driver:D101').send({ etaMinutes: 5 }).expect(200);
  for (const step of ['arrive', 'pickup', 'start']) await request(app).post(`/api/trips/${id}/${step}`).set('x-user', 'driver:D101').expect(200);
  system.payments.setMode(thenMode);
  system.payments.failNext(...failures);
  const res = await request(app).post(`/api/trips/${id}/complete`).set('x-user', 'driver:D101').expect(200);
  return { paymentStatus: res.body.trip.paymentStatus, calls: system.payments.calls.length };
}

describe('Bảo trì khi R-PAY-04 đổi 2 → 3 lần thử lại', () => {
  it('phân tích ảnh hưởng: chỉ TC-PAY-003 đổi kết quả mong đợi (3 → 4 lần gọi); TC-PAY-010 là ca biên mới', () => {
    const changed = CASES.filter((c) => JSON.stringify(oracle(2, c.failures, c.thenMode)) !== JSON.stringify(oracle(3, c.failures, c.thenMode)));
    expect(changed.map((c) => c.id)).toEqual(['TC-PAY-003', 'TC-PAY-010']);
    expect(oracle(2, [], 'error')).toEqual({ paymentStatus: 'failed', calls: 3 });
    expect(oracle(3, [], 'error')).toEqual({ paymentStatus: 'failed', calls: 4 });
    expect(oracle(2, ['timeout', 'timeout', 'timeout'], 'none')).toEqual({ paymentStatus: 'failed', calls: 3 });
    expect(oracle(3, ['timeout', 'timeout', 'timeout'], 'none')).toEqual({ paymentStatus: 'paid', calls: 4 });
  });

  it.each(CASES.filter((c) => c.since === 'v1.1'))('hệ thống hiện tại (2 lần) khớp oracle v1.1 · $id', async (c) => {
    expect(await run(2, c.failures, c.thenMode)).toEqual(oracle(2, c.failures, c.thenMode));
  });

  it('bộ v1.1 chạy trên hệ thống đã đổi sang 3 lần: đỏ đúng một ca — TC-PAY-003; ba ca còn lại xanh nhưng không kiểm lượt thứ 4', async () => {
    const red: string[] = [];
    for (const c of CASES.filter((x) => x.since === 'v1.1')) {
      const actual = await run(3, c.failures, c.thenMode);
      if (JSON.stringify(actual) !== JSON.stringify(oracle(2, c.failures, c.thenMode))) red.push(c.id);
    }
    expect(red).toEqual(['TC-PAY-003']);
  });

  it.each(CASES)('bộ v1.2 (đã cập nhật + ca biên mới) khớp hệ thống 3 lần · $id', async (c) => {
    expect(await run(3, c.failures, c.thenMode)).toEqual(oracle(3, c.failures, c.thenMode));
  });
});
