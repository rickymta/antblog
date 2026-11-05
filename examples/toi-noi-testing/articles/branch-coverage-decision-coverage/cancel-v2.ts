/**
 * TEST-044 · BẢN TÁI CẤU TRÚC CÓ LỖI của customerCancellationFee — chỉ để minh hoạ, KHÔNG phải mã lab.
 * Ai đó gom ba lối ra cuối thành «giá trị mặc định + ghi đè» (if / else if không có else).
 * Mặc định lý do được đặt là 'charged' với phí 0 ⇒ trường hợp tài xế trễ (hai quyết định cùng SAI)
 * trả { fee: 0, reason: 'charged' } thay vì 'driver_late' (SPEC R-CANCEL-06).
 * Đường đi lỗi KHÔNG có câu lệnh riêng nào: 100% statement coverage không ép phải đi qua nó.
 */
import { CANCELLATION_FEE, DRIVER_LATE_GRACE_MINUTES, FREE_CANCEL_MINUTES, type CancellationFee, type CancellationInput, type FeeReason } from '../../src/domain/cancel';
import { MINUTE_MS } from '../../src/domain/time';
import { InvalidTransitionError } from '../../src/domain/trip-state';

export function customerCancellationFeeV2(input: CancellationInput): CancellationFee {
  if (input.status === 'searching') return { fee: 0, reason: 'searching' };
  if (input.status !== 'accepted' && input.status !== 'arriving') {
    throw new InvalidTransitionError(input.status, 'customer_cancel');
  }
  if (input.acceptedAt === null || input.etaMinutes === null) {
    throw new Error(`Chuyến ở "${input.status}" phải có acceptedAt và etaMinutes`);
  }

  const sinceAcceptedMs = input.now.getTime() - input.acceptedAt.getTime();
  const lateAfterMs = (input.etaMinutes + DRIVER_LATE_GRACE_MINUTES) * MINUTE_MS;
  let fee = 0;
  let reason: FeeReason = 'charged'; // DEFECT: mặc định sai — nhánh «tài xế trễ» không ghi đè
  if (input.status === 'accepted' && sinceAcceptedMs < FREE_CANCEL_MINUTES * MINUTE_MS) {
    reason = 'grace_period';
  } else if (sinceAcceptedMs <= lateAfterMs) {
    fee = CANCELLATION_FEE[input.vehicle];
  }
  return { fee, reason };
}
