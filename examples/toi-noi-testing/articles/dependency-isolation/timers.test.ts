import { afterEach, describe, expect, it, vi } from 'vitest';
import { chargeWithRetry, withTimeout } from '../../src/app/payment';
import { FakePaymentGateway } from '../../src/infra/fake-payment-gateway';

afterEach(() => { vi.clearAllTimers(); vi.useRealTimers(); });

describe('TEST-058 — scheduler khác clock nghiệp vụ', () => {
  it('withTimeout: 2.999 ms chưa hết hạn, đúng 3.000 ms thì lỗi', async () => {
    vi.useFakeTimers();
    let settled = false;
    // Gắn rejection handler trước khi đẩy timer, tránh unhandled rejection.
    const outcome = withTimeout(new Promise<never>(() => {}), 3_000).then(
      () => { settled = true; return 'unexpected'; },
      error => { settled = true; return error; },
    );
    await vi.advanceTimersByTimeAsync(2_999);
    expect(settled).toBe(false);
    await vi.advanceTimersByTimeAsync(1);
    expect(await outcome).toMatchObject({ name: 'PaymentTimeoutError' });
    expect(vi.getTimerCount()).toBe(0);
  });

  it('R-PAY-04: mất phản hồi sau thu tiền, retry vẫn chỉ thu một lần', async () => {
    vi.useFakeTimers();
    const fake = new FakePaymentGateway();
    fake.failNext('lost_response');
    const request = { idempotencyKey: 'trip:timer:fare', customerId: 'C001', amount: 21_000, description: 'Cước chuyến' };
    const result = chargeWithRetry(fake, request, { timeoutMs: 3_000, maxRetries: 2 });
    const checked = expect(result).resolves.toMatchObject({ status: 'paid', attempts: 2, failure: null });
    await vi.advanceTimersByTimeAsync(3_000);
    await checked;
    expect(fake.calls.map(call => call.idempotencyKey)).toEqual([request.idempotencyKey, request.idempotencyKey]);
    expect(fake.totalCharged()).toBe(21_000);
    expect(vi.getTimerCount()).toBe(0);
  });
});
