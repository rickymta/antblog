/**
 * TEST-049 · Loop testing — vòng thử lại thanh toán (R-PAY-04) và vòng lồng nhau trong canonicalJson (R-IDEM-02).
 * Lệnh: npx vitest run articles/loop-testing
 */
import { describe, expect, it } from 'vitest';
import { fingerprint } from '../../src/api/idempotency';
import { chargeWithRetry } from '../../src/app/payment';
import { FakePaymentGateway, type FakeFailureMode } from '../../src/infra/fake-payment-gateway';

const request = { idempotencyKey: 'trip:id-0001:fare', customerId: 'C001', amount: 52_000, description: 'Cước chuyến' };

async function run(failures: FakeFailureMode[], maxRetries = 2) {
  const gateway = new FakePaymentGateway();
  gateway.failNext(...failures);
  const outcome = await chargeWithRetry(gateway, request, { timeoutMs: 20, maxRetries });
  return { outcome, calls: gateway.calls, charged: gateway.totalCharged() };
}

describe('LR · vòng thử lại chargeWithRetry: số lần lặp do phản hồi của cổng quyết định (max = 1 + maxRetries = 3)', () => {
  it('1 lần: thành công ngay ⇒ paid, 1 lần gọi', async () => {
    const { outcome, calls } = await run([]);
    expect(outcome).toMatchObject({ status: 'paid', attempts: 1 });
    expect(calls).toHaveLength(1);
  });

  it('2 lần: hết giờ rồi thành công ⇒ paid, 2 lần gọi CÙNG khoá', async () => {
    const { outcome, calls, charged } = await run(['timeout']);
    expect(outcome).toMatchObject({ status: 'paid', attempts: 2 });
    expect(new Set(calls.map((c) => c.idempotencyKey))).toEqual(new Set([request.idempotencyKey]));
    expect(charged).toBe(52_000);
  });

  it('max = 3 lần: hết giờ, lỗi tạm thời, rồi thành công ⇒ paid ở lần 3', async () => {
    const { outcome, calls } = await run(['timeout', 'error']);
    expect(outcome).toMatchObject({ status: 'paid', attempts: 3 });
    expect(calls).toHaveLength(3);
  });

  it('max + 1: bốn lần hết giờ xếp hàng ⇒ dừng sau ĐÚNG 3 lần gọi, failed/timeout, không trừ tiền', async () => {
    const { outcome, calls, charged } = await run(['timeout', 'timeout', 'timeout', 'timeout']);
    expect(outcome).toEqual({ status: 'failed', attempts: 3, failure: 'timeout', chargeId: null });
    expect(calls).toHaveLength(3);
    expect(charged).toBe(0);
  });

  it('thoát sớm giữa vòng: lỗi tạm thời rồi bị từ chối ⇒ dừng ở lần 2, không thử lần 3', async () => {
    const { outcome, calls } = await run(['error', 'decline']);
    expect(outcome).toMatchObject({ status: 'failed', attempts: 2, failure: 'declined' });
    expect(calls).toHaveLength(2);
  });

  it('bỏ qua vòng: chỉ xảy ra khi cấu hình sai (maxRetries = −1) ⇒ failed, failure null, 0 lần gọi', async () => {
    const { outcome, calls } = await run([], -1);
    expect(outcome).toEqual({ status: 'failed', attempts: 0, failure: null, chargeId: null });
    expect(calls).toHaveLength(0);
  });
});

describe('LN · vòng lồng nhau: canonicalJson duyệt mảng (vòng ngoài) rồi khoá của từng phần tử (vòng trong)', () => {
  const FIELDS = ['name', 'unitPrice', 'quantity'] as const;
  const values: Record<(typeof FIELDS)[number], string | number> = { name: 'Phở', unitPrice: 45_000, quantity: 2 };

  /** outer phần tử, mỗi phần tử có `inner` khoá đầu của FIELDS; `reversed` đảo thứ tự khoá trong từng phần tử. */
  function body(outer: number, inner: number, reversed: boolean) {
    const keys = FIELDS.slice(0, inner);
    const ordered = reversed ? [...keys].reverse() : keys;
    const items = Array.from({ length: outer }, (_, i) =>
      Object.fromEntries(ordered.map((k) => [k, k === 'quantity' ? i + 1 : values[k]])),
    );
    return reversed ? { paymentMethod: 'cash', items } : { items, paymentMethod: 'cash' };
  }

  // Beizer: vòng trong chạy đủ bộ giá trị khi vòng ngoài ở mức tối thiểu (1),
  // rồi vòng ngoài chạy đủ bộ giá trị khi vòng trong ở mức điển hình (3). 4 + 4 − 1 = 7 ca thay vì 4 × 4 = 16.
  const cases: Array<[outer: number, inner: number]> = [
    [1, 0], [1, 1], [1, 2], [1, 3],
    [0, 3], [2, 3], [20, 3],
  ];

  it.each(cases)('ngoài = %i, trong = %i ⇒ đảo thứ tự khoá không đổi vân tay', (outer, inner) => {
    expect(fingerprint(body(outer, inner, true))).toBe(fingerprint(body(outer, inner, false)));
  });

  it('ngoài = 20, trong = 3 ⇒ đổi một giá trị ở phần tử CUỐI thì vân tay phải đổi', () => {
    const changed = body(20, 3, false);
    (changed.items[19] as Record<string, unknown>).quantity = 99;
    expect(fingerprint(changed)).not.toBe(fingerprint(body(20, 3, false)));
  });

  it('thứ tự PHẦN TỬ mảng có nghĩa: đảo hai món ⇒ vân tay khác (R-IDEM-02 chỉ bỏ qua thứ tự khoá)', () => {
    const original = body(2, 3, false);
    const swapped = { ...original, items: [...original.items].reverse() };
    expect(fingerprint(swapped)).not.toBe(fingerprint(original));
  });
});
