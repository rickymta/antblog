/**
 * TCD-07 — bản chép customerCancellationFee (src/domain/cancel.ts) có defect CỐ Ý ở hai phép so sánh biên:
 *  - mốc 2 phút: `<=` thay vì `<`  ⇒ đúng 2:00 vẫn miễn phí (sai R-CANCEL-02);
 *  - mốc tài xế trễ: `>=` thay vì `>` ⇒ đúng mốc ETA + 10 phút đã miễn phí (sai R-CANCEL-04).
 * Chỉ dùng để cho thấy ví dụ nằm trên biên trong acceptance criteria bắt được gì.
 */
import { CANCELLATION_FEE, DRIVER_LATE_GRACE_MINUTES, FREE_CANCEL_MINUTES, type CancellationFee, type CancellationInput } from '../../src/domain/cancel';
import { MINUTE_MS } from '../../src/domain/time';
import { InvalidTransitionError } from '../../src/domain/trip-state';

export function buggyCustomerCancellationFee(input: CancellationInput): CancellationFee {
  if (input.status === 'searching') return { fee: 0, reason: 'searching' };
  if (input.status !== 'accepted' && input.status !== 'arriving') {
    throw new InvalidTransitionError(input.status, 'customer_cancel');
  }
  const since = input.now.getTime() - input.acceptedAt!.getTime();
  if (input.status === 'accepted' && since <= FREE_CANCEL_MINUTES * MINUTE_MS) return { fee: 0, reason: 'grace_period' }; // defect
  if (since >= (input.etaMinutes! + DRIVER_LATE_GRACE_MINUTES) * MINUTE_MS) return { fee: 0, reason: 'driver_late' }; // defect
  return { fee: CANCELLATION_FEE[input.vehicle], reason: 'charged' };
}
