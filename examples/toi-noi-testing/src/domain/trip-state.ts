import { DomainError } from './errors';

/** Trạng thái chuyến (SPEC R-STATE-01). */
export const TRIP_STATUSES = [
  'requested',
  'searching',
  'accepted',
  'arriving',
  'picked_up',
  'in_transit',
  'completed',
  'cancelled',
] as const;
export type TripStatus = (typeof TRIP_STATUSES)[number];

export const TRIP_EVENTS = [
  'search',
  'accept',
  'arrive',
  'pickup',
  'start',
  'complete',
  'customer_cancel',
  'driver_cancel',
  'system_cancel',
] as const;
export type TripEvent = (typeof TRIP_EVENTS)[number];

/** Bảng chuyển hợp lệ (SPEC R-STATE-02). Cặp không có trong bảng là không hợp lệ. */
export const TRANSITIONS: Readonly<Record<TripStatus, Partial<Record<TripEvent, TripStatus>>>> = {
  requested: { search: 'searching' },
  searching: { accept: 'accepted', customer_cancel: 'cancelled', system_cancel: 'cancelled' },
  accepted: { arrive: 'arriving', customer_cancel: 'cancelled', driver_cancel: 'searching' },
  arriving: { pickup: 'picked_up', customer_cancel: 'cancelled', driver_cancel: 'searching' },
  picked_up: { start: 'in_transit' },
  in_transit: { complete: 'completed' },
  completed: {},
  cancelled: {},
};

/** Trạng thái tài xế đang bận với chuyến (SPEC R-ACCEPT-05). */
export const ACTIVE_STATUSES: readonly TripStatus[] = ['accepted', 'arriving', 'picked_up', 'in_transit'];

export class InvalidTransitionError extends DomainError {
  readonly from: TripStatus;
  readonly event: TripEvent;

  constructor(from: TripStatus, event: TripEvent) {
    super('INVALID_TRANSITION', `Không thể thực hiện "${event}" khi chuyến đang ở trạng thái "${from}".`);
    this.from = from;
    this.event = event;
  }
}

export function canTransition(from: TripStatus, event: TripEvent): boolean {
  return TRANSITIONS[from][event] !== undefined;
}

/** Hàm chuyển trạng thái thuần: trả trạng thái mới hoặc ném INVALID_TRANSITION (SPEC R-STATE-03). */
export function nextStatus(from: TripStatus, event: TripEvent): TripStatus {
  const to = TRANSITIONS[from][event];
  if (to === undefined) throw new InvalidTransitionError(from, event);
  return to;
}

export function isTerminal(status: TripStatus): boolean {
  return status === 'completed' || status === 'cancelled';
}
