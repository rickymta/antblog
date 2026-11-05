/**
 * TEST-033 — bản CHÉP của src/domain/cancel.ts có MỘT defect cố ý để minh hoạ.
 * Defect: quên điều kiện «chỉ ở accepted» của thời gian miễn phí 2 phút (R-CANCEL-01)
 * ⇒ khách huỷ ở arriving trong 2 phút đầu cũng được miễn phí. Không sửa src/.
 */
import { CANCELLATION_FEE, DRIVER_LATE_GRACE_MINUTES, FREE_CANCEL_MINUTES, type CancellationFee, type CancellationInput } from '../../src/domain/cancel';
import { MINUTE_MS } from '../../src/domain/time';
import { InvalidTransitionError } from '../../src/domain/trip-state';

export function buggyCustomerCancellationFee(input: CancellationInput): CancellationFee {
  if (input.status === 'searching') return { fee: 0, reason: 'searching' };
  if (input.status !== 'accepted' && input.status !== 'arriving') {
    throw new InvalidTransitionError(input.status, 'customer_cancel');
  }
  if (input.acceptedAt === null || input.etaMinutes === null) {
    throw new Error(`Chuyến ở "${input.status}" phải có acceptedAt và etaMinutes`);
  }
  const sinceAcceptedMs = input.now.getTime() - input.acceptedAt.getTime();
  // DEFECT: thiếu `input.status === 'accepted' &&`
  if (sinceAcceptedMs < FREE_CANCEL_MINUTES * MINUTE_MS) return { fee: 0, reason: 'grace_period' };
  if (sinceAcceptedMs > (input.etaMinutes + DRIVER_LATE_GRACE_MINUTES) * MINUTE_MS) return { fee: 0, reason: 'driver_late' };
  return { fee: CANCELLATION_FEE[input.vehicle], reason: 'charged' };
}
