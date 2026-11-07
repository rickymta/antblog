import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Actor } from '../../src/app/deps';
import { chargeWithRetry } from '../../src/app/payment';
import { PaymentUnavailableError, type PaymentGateway } from '../../src/app/ports';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, createTrip, driverStep } from '../../src/app/trips';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { FakePaymentGateway } from '../../src/infra/fake-payment-gateway';

const customer: Actor = { role: 'customer', id: 'C001' };
const driver: Actor = { role: 'driver', id: 'D101' };
const request = { idempotencyKey: 'trip:example:fare', customerId: 'C001', amount: 21_000, description: 'Cước chuyến' };
const options = { timeoutMs: 3_000, maxRetries: 2 };
const unusedRefund: PaymentGateway['refund'] = async () => { throw new Error('Không được gọi refund trong ca này'); };

async function ongoingTrip(gateway: PaymentGateway, paymentMethod: 'cash' | 'vixanh' = 'vixanh') {
  const system = createMemorySystem({ clock: new FixedClock(new Date('2026-11-07T02:00:00Z')) });
  system.deps.payments = gateway;
  const quote = await createQuote(system.deps, customer, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 2 });
  const trip = await createTrip(system.deps, customer, { quoteId: quote.id, paymentMethod });
  await acceptTrip(system.deps, driver, trip.id, 3);
  for (const event of ['arrive', 'pickup', 'start'] as const) await driverStep(system.deps, driver, trip.id, event);
  return { system, trip };
}

afterEach(() => vi.restoreAllMocks());

describe('TEST-057 — vai trò của test double tại cổng thanh toán', () => {
  it('dummy: chuyến cash hoàn thành mà không dùng cổng ví', async () => {
    const dummy: PaymentGateway = {
      charge: async () => { throw new Error('Chuyến cash không được gọi ví'); },
      refund: unusedRefund,
    };
    const { system, trip } = await ongoingTrip(dummy, 'cash');
    const completed = await driverStep(system.deps, driver, trip.id, 'complete');
    expect(completed).toMatchObject({ status: 'completed', paymentStatus: 'cash' });
    expect(await system.deps.drivers.findById(driver.id)).toMatchObject({ status: 'online' });
  });

  it('stub: ví từ chối nhưng chuyến vẫn completed, paymentStatus failed', async () => {
    const stub: PaymentGateway = { charge: async () => ({ status: 'declined', chargeId: null }), refund: unusedRefund };
    const { system, trip } = await ongoingTrip(stub);
    expect(await driverStep(system.deps, driver, trip.id, 'complete'))
      .toMatchObject({ status: 'completed', paymentStatus: 'failed' });
    expect(await system.deps.trips.findById(trip.id)).toMatchObject({ status: 'completed', paymentStatus: 'failed' });
  });

  it('fake: gửi lại cùng khoá không làm tăng tiền đã thu', async () => {
    const fake = new FakePaymentGateway();
    const first = await chargeWithRetry(fake, request, options);
    const replay = await chargeWithRetry(fake, { ...request }, options);
    expect(first.status).toBe('paid');
    expect(replay.chargeId).toBe(first.chargeId);
    expect(fake.totalCharged()).toBe(21_000);
    expect(fake.calls).toHaveLength(2);
  });

  it('mock-style: hoàn thành gửi đúng khách, số tiền và khoá R-PAY-03', async () => {
    const charge = vi.fn<PaymentGateway['charge']>().mockResolvedValue({ status: 'succeeded', chargeId: 'charge-demo' });
    const { system, trip } = await ongoingTrip({ charge, refund: unusedRefund });
    expect(await driverStep(system.deps, driver, trip.id, 'complete')).toMatchObject({ paymentStatus: 'paid' });
    expect(charge).toHaveBeenCalledTimes(1);
    expect(charge).toHaveBeenCalledWith(expect.objectContaining({
      customerId: 'C001', amount: 21_000, idempotencyKey: `trip:${trip.id}:fare`,
    }));
  });

  it('spy: quan sát fake đang chạy thật mà không thay implementation', async () => {
    const fake = new FakePaymentGateway();
    const charge = vi.spyOn(fake, 'charge');
    const { system, trip } = await ongoingTrip(fake);
    await driverStep(system.deps, driver, trip.id, 'complete');
    expect(charge).toHaveBeenCalledTimes(1);
    expect(fake.totalCharged()).toBe(21_000);
  });

  it('R-PAY-04: lỗi tạm thời được retry với cùng request', async () => {
    const charge = vi.fn<PaymentGateway['charge']>()
      .mockRejectedValueOnce(new PaymentUnavailableError())
      .mockResolvedValue({ status: 'succeeded', chargeId: 'retry-ok' });
    expect(await chargeWithRetry({ charge, refund: unusedRefund }, request, options))
      .toMatchObject({ status: 'paid', attempts: 2 });
    expect(charge.mock.calls.map(([arg]) => arg.idempotencyKey)).toEqual([request.idempotencyKey, request.idempotencyKey]);
    expect(charge.mock.calls.map(([arg]) => arg.amount)).toEqual([21_000, 21_000]);
  });

  it('R-PAY-04: declined không retry', async () => {
    const charge = vi.fn<PaymentGateway['charge']>().mockResolvedValue({ status: 'declined', chargeId: null });
    expect(await chargeWithRetry({ charge, refund: unusedRefund }, request, options))
      .toMatchObject({ status: 'failed', attempts: 1, failure: 'declined' });
    expect(charge).toHaveBeenCalledTimes(1);
  });
});
