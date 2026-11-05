/**
 * TEST-034 — bản CHÉP bảng chuyển của src/domain/trip-state.ts có MỘT defect cố ý:
 * cho khách huỷ khi đã đón (`picked_up` + `customer_cancel` ⇒ `cancelled`), trái R-STATE-02/03.
 * Mọi chuyển hợp lệ vẫn đúng ⇒ chỉ test chuyển KHÔNG hợp lệ mới bắt được. Không sửa src/.
 */
import { InvalidTransitionError, TRANSITIONS, type TripEvent, type TripStatus } from '../../src/domain/trip-state';

const BUGGY: Record<TripStatus, Partial<Record<TripEvent, TripStatus>>> = {
  ...TRANSITIONS,
  picked_up: { ...TRANSITIONS.picked_up, customer_cancel: 'cancelled' }, // DEFECT
};

export function buggyNextStatus(from: TripStatus, event: TripEvent): TripStatus {
  const to = BUGGY[from][event];
  if (to === undefined) throw new InvalidTransitionError(from, event);
  return to;
}
