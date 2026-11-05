/**
 * Bài TEST-015 «System Integration Testing» — kiểm PHÍA TỚI NƠI của hai điểm tích hợp với hệ ngoài
 * mà lab có bản giả: ví VíXanh (IP-1) và dịch vụ thời tiết (IP-4).
 *
 * Bản giả chỉ chứng minh được: Tới Nơi GỬI đúng dữ liệu và XỬ LÝ đúng các phản hồi mà ta đã mô phỏng.
 * Nó KHÔNG chứng minh VíXanh thật cư xử như bản giả — đó là việc của system integration testing
 * trên sandbox/staging (ngoài lab). Ca SIT-05 chạy cùng một bộ kiểm «giả định về đối tác» với bản giả;
 * khi có sandbox, chỉ cần truyền một client sandbox vào cùng hàm `kiemGiaDinhViXanh`.
 *
 * Chạy: npx vitest run articles/system-integration-testing
 */
import { beforeEach, describe, expect, it } from 'vitest';
import type { Actor } from '../../src/app/deps';
import type { PaymentGateway } from '../../src/app/ports';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, createTrip, driverStep } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';
import { FakePaymentGateway } from '../../src/infra/fake-payment-gateway';

const C001: Actor = { role: 'customer', id: 'C001' };
const D101: Actor = { role: 'driver', id: 'D101' };

describe('TEST-015 · Điểm tích hợp Tới Nơi ↔ hệ ngoài (phía Tới Nơi, dùng bản giả)', () => {
  const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0)); // 09:00 giờ VN — ngoài cao điểm
  // Hết giờ 50 ms thay cho 3 giây của R-PAY-04 để test chạy nhanh; số lần thử lại giữ nguyên (2).
  const system: LabSystem = createMemorySystem({ clock, config: { payment: { timeoutMs: 50, maxRetries: 2 } } });

  beforeEach(async () => {
    await system.reset();
    clock.set(vnTime(2026, 11, 4, 9, 0));
  });

  /** Chuyến xe máy 5 km ở SG trả bằng VíXanh, tài xế D101 chạy tới hoàn thành. Cước 35.000 đ. */
  async function chuyenViXanhHoanThanh() {
    const quote = await createQuote(system.deps, C001, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 });
    const trip = await createTrip(system.deps, C001, { quoteId: quote.id, paymentMethod: 'vixanh' });
    await acceptTrip(system.deps, D101, trip.id, 5);
    for (const step of ['arrive', 'pickup', 'start'] as const) await driverStep(system.deps, D101, trip.id, step);
    return driverStep(system.deps, D101, trip.id, 'complete');
  }

  it('SIT-01 · IP-1 · R-PAY-03 · gửi VíXanh đúng số tiền (nguyên đồng), đúng khách, đúng khoá idempotency', async () => {
    const done = await chuyenViXanhHoanThanh();
    expect(system.payments.calls).toEqual([
      { idempotencyKey: `trip:${done.id}:fare`, customerId: 'C001', amount: 35_000, description: 'Cước chuyến' },
    ]);
    expect(system.payments.calls.every((c) => Number.isInteger(c.amount))).toBe(true);
    expect(done.paymentStatus).toBe('paid');
  });

  it('SIT-02 · IP-1 · R-PAY-04, R-NFR-05 · mất phản hồi ⇒ thử lại CÙNG khoá, ví chỉ trừ một lần', async () => {
    system.payments.failNext('lost_response'); // lần 1: VíXanh đã trừ tiền nhưng không trả lời
    const done = await chuyenViXanhHoanThanh();
    const keys = system.payments.calls.map((c) => c.idempotencyKey);
    expect(keys).toEqual([`trip:${done.id}:fare`, `trip:${done.id}:fare`]);
    expect(system.payments.totalCharged()).toBe(35_000);
    expect(done.paymentStatus).toBe('paid');
  });

  it('SIT-03 · IP-1 · R-PAY-04, R-PAY-06 · VíXanh từ chối ⇒ không thử lại, chuyến vẫn hoàn thành, failed', async () => {
    system.payments.setMode('decline');
    const done = await chuyenViXanhHoanThanh();
    expect(system.payments.calls).toHaveLength(1);
    expect(done.status).toBe('completed');
    expect(done.paymentStatus).toBe('failed');
  });

  it('SIT-04 · IP-4 · R-GEN-05, R-FARE-05 · dữ liệu «đang mưa» từ dịch vụ thời tiết đi vào giá đúng chỗ', async () => {
    system.weather.setRaining(['SG']);
    const sg = await createQuote(system.deps, C001, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 });
    const hn = await createQuote(system.deps, C001, { kind: 'ride', cityCode: 'HN', vehicle: 'bike', distanceKm: 5 });
    expect([sg.details.raining, sg.details.fare.rainSurcharge, sg.details.total]).toEqual([true, 8_000, 43_000]);
    expect([hn.details.raining, hn.details.fare.rainSurcharge, hn.details.total]).toEqual([false, 0, 35_000]);
  });

  it('SIT-05 · giả định A-1 về VíXanh · cùng khoá ⇒ cùng giao dịch; khác khoá ⇒ giao dịch mới (chạy với bản GIẢ)', async () => {
    await kiemGiaDinhViXanh(new FakePaymentGateway());
  });
});

/**
 * Bộ kiểm giả định về đối tác: Tới Nơi DỰA vào điều này để thử lại an toàn (R-PAY-04).
 * Với bản giả, ca này chỉ cho biết bản giả mô phỏng đúng giả định. Giá trị thật của nó là khi
 * truyền vào client sandbox của VíXanh — khi đó nó là một test system integration.
 */
async function kiemGiaDinhViXanh(gateway: PaymentGateway): Promise<void> {
  const request = { idempotencyKey: 'sit-a1-0001', customerId: 'C001', amount: 35_000, description: 'SIT A-1' };
  const first = await gateway.charge(request);
  const again = await gateway.charge(request);
  const other = await gateway.charge({ ...request, idempotencyKey: 'sit-a1-0002' });
  expect(first.status).toBe('succeeded');
  expect(again.chargeId).toBe(first.chargeId);
  expect(other.chargeId).not.toBe(first.chargeId);
}
