/**
 * TEST-028 · Checklist-based Review — xác nhận phát hiện PAY-09 ở bước communication and analysis.
 * Phát hiện được tìm bằng REVIEW (không chạy code); test này chỉ là bằng chứng để nhóm quyết định
 * anomaly đó có phải defect hay không. Không sửa src/ — chỉ gọi mã của lab.
 */
import { describe, expect, it } from 'vitest';
import { chargeWithRetry } from '../../src/app/payment';
import { FakePaymentGateway } from '../../src/infra/fake-payment-gateway';

const request = {
  idempotencyKey: 'trip:id-0002:fare',
  customerId: 'C001',
  amount: 52_000,
  description: 'Cước chuyến',
};
// SPEC R-PAY-04: hết giờ 3 000 ms, thử lại tối đa 2 lần. Rút hết giờ còn 20 ms cho test nhanh.
const options = { timeoutMs: 20, maxRetries: 2 };

describe('PAY-09 · kết quả «không rõ» bị gộp vào «thất bại»', () => {
  it('lost_response cả 3 lần: status failed nhưng ví đã trừ 52.000 đ (đúng một lần)', async () => {
    const gateway = new FakePaymentGateway();
    gateway.setMode('lost_response'); // R-TEST-03: đã trừ tiền nhưng không trả lời

    const outcome = await chargeWithRetry(gateway, request, options);

    expect(outcome).toMatchObject({ status: 'failed', failure: 'timeout', attempts: 3 });
    expect(gateway.calls).toHaveLength(3);
    expect(gateway.calls.every((c) => c.idempotencyKey === request.idempotencyKey)).toBe(true);
    expect(gateway.totalCharged()).toBe(52_000);
    console.log(`[PAY-09] lost_response: status=${outcome.status}, ${outcome.attempts} lần gọi, ví đã trừ ${gateway.totalCharged()} đ`);
  });

  it('đối chứng — timeout thật (không trừ tiền): cùng kết quả failed, ví trừ 0 đ', async () => {
    const gateway = new FakePaymentGateway();
    gateway.setMode('timeout'); // R-TEST-03: không trả lời và không trừ tiền

    const outcome = await chargeWithRetry(gateway, request, options);

    expect(outcome).toMatchObject({ status: 'failed', failure: 'timeout', attempts: 3 });
    expect(gateway.totalCharged()).toBe(0);
    console.log(`[PAY-09] timeout:       status=${outcome.status}, ${outcome.attempts} lần gọi, ví đã trừ ${gateway.totalCharged()} đ`);
  });
});
