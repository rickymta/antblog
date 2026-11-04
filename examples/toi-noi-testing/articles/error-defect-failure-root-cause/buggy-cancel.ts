/**
 * BẢN CÓ LỖI CỐ Ý — chỉ dùng cho bài TEST-004 «Error, Defect, Failure và Root Cause».
 * Chép từ src/domain/cancel.ts, sửa ĐÚNG MỘT ký tự ở dòng đánh dấu DEFECT.
 *
 * Câu chuyện (giả định): lập trình viên đọc yêu cầu cũ «khách huỷ trong vòng 2 phút sau khi
 * tài xế nhận thì miễn phí» và hiểu «trong vòng 2 phút» là «tới hết phút thứ 2, tính cả mốc 2:00».
 *   - error  : hiểu sai mốc biên (hành động của con người)
 *   - defect : `<=` thay vì `<` ở dòng DEFECT (khiếm khuyết nằm trong mã)
 *   - failure: khách huỷ đúng 2 phút 0 giây không bị thu phí — trái SPEC R-CANCEL-02
 * KHÔNG import file này từ src/.
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
  // DEFECT: đặc tả R-CANCEL-02 là «now − acceptedAt < 2 phút»; ở đây viết `<=`.
  if (input.status === 'accepted' && sinceAcceptedMs <= FREE_CANCEL_MINUTES * MINUTE_MS) {
    return { fee: 0, reason: 'grace_period' };
  }
  const lateAfterMs = (input.etaMinutes + DRIVER_LATE_GRACE_MINUTES) * MINUTE_MS;
  if (sinceAcceptedMs > lateAfterMs) {
    return { fee: 0, reason: 'driver_late' };
  }
  return { fee: CANCELLATION_FEE[input.vehicle], reason: 'charged' };
}
