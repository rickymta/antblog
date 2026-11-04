import { InvalidTransitionError, type TripStatus } from './trip-state';
import { MINUTE_MS } from './time';
import type { Vehicle } from './types';

export const CANCELLATION_FEE: Readonly<Record<Vehicle, number>> = { bike: 10_000, car: 20_000 }; // SPEC R-CANCEL-03
export const FREE_CANCEL_MINUTES = 2; // SPEC R-CANCEL-02
export const DRIVER_LATE_GRACE_MINUTES = 10; // SPEC R-CANCEL-04

export type FeeReason = 'searching' | 'grace_period' | 'driver_late' | 'charged';

export interface CancellationInput {
  status: TripStatus;
  vehicle: Vehicle;
  acceptedAt: Date | null;
  etaMinutes: number | null;
  now: Date;
}

export interface CancellationFee {
  fee: number;
  reason: FeeReason;
}

/**
 * Phí khi KHÁCH huỷ (SPEC R-CANCEL-01…06). Tài xế/hệ thống huỷ không gọi hàm này.
 * Trạng thái không cho khách huỷ ⇒ INVALID_TRANSITION.
 */
export function customerCancellationFee(input: CancellationInput): CancellationFee {
  if (input.status === 'searching') return { fee: 0, reason: 'searching' };
  if (input.status !== 'accepted' && input.status !== 'arriving') {
    throw new InvalidTransitionError(input.status, 'customer_cancel');
  }
  if (input.acceptedAt === null || input.etaMinutes === null) {
    throw new Error(`Chuyến ở "${input.status}" phải có acceptedAt và etaMinutes`);
  }

  const sinceAcceptedMs = input.now.getTime() - input.acceptedAt.getTime();
  if (input.status === 'accepted' && sinceAcceptedMs < FREE_CANCEL_MINUTES * MINUTE_MS) {
    return { fee: 0, reason: 'grace_period' };
  }
  const lateAfterMs = (input.etaMinutes + DRIVER_LATE_GRACE_MINUTES) * MINUTE_MS;
  if (sinceAcceptedMs > lateAfterMs) {
    return { fee: 0, reason: 'driver_late' };
  }
  return { fee: CANCELLATION_FEE[input.vehicle], reason: 'charged' };
}
