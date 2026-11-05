/**
 * TEST-034 — mô hình trạng thái chuyến chép TỪ ĐẶC TẢ (SPEC R-STATE-01/02), không import bảng của src/:
 * test basis phải độc lập với thứ được kiểm. Kèm hàm chạy một chuỗi sự kiện và đo ba loại coverage của CTFL 4.2.4.
 */
import type { TripEvent, TripStatus } from '../../src/domain/trip-state';

export const STATES: readonly TripStatus[] = ['requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed', 'cancelled'];
export const EVENTS: readonly TripEvent[] = ['search', 'accept', 'arrive', 'pickup', 'start', 'complete', 'customer_cancel', 'driver_cancel', 'system_cancel'];

export interface Transition {
  id: string;
  from: TripStatus;
  event: TripEvent;
  to: TripStatus;
}

/** 12 chuyển hợp lệ — R-STATE-02 tách mỗi trạng thái nguồn thành một chuyển. */
export const VALID: readonly Transition[] = [
  { id: 'T01', from: 'requested', event: 'search', to: 'searching' },
  { id: 'T02', from: 'searching', event: 'accept', to: 'accepted' },
  { id: 'T03', from: 'accepted', event: 'arrive', to: 'arriving' },
  { id: 'T04', from: 'arriving', event: 'pickup', to: 'picked_up' },
  { id: 'T05', from: 'picked_up', event: 'start', to: 'in_transit' },
  { id: 'T06', from: 'in_transit', event: 'complete', to: 'completed' },
  { id: 'T07', from: 'searching', event: 'customer_cancel', to: 'cancelled' },
  { id: 'T08', from: 'accepted', event: 'customer_cancel', to: 'cancelled' },
  { id: 'T09', from: 'arriving', event: 'customer_cancel', to: 'cancelled' },
  { id: 'T10', from: 'accepted', event: 'driver_cancel', to: 'searching' },
  { id: 'T11', from: 'arriving', event: 'driver_cancel', to: 'searching' },
  { id: 'T12', from: 'searching', event: 'system_cancel', to: 'cancelled' },
];

export function validTarget(from: TripStatus, event: TripEvent): TripStatus | null {
  return VALID.find((t) => t.from === from && t.event === event)?.to ?? null;
}

/** Hàm chuyển của một bản cài đặt: trả trạng thái mới hoặc ném lỗi (như nextStatus). */
export type Machine = (from: TripStatus, event: TripEvent) => TripStatus;

/** Năm test case — mỗi ca là một chuỗi sự kiện bắt đầu từ `requested`. */
export const TEST_CASES: readonly { id: string; events: TripEvent[]; ketThuc: TripStatus }[] = [
  {
    id: 'TC-ST-01', // luồng chính, chen hai lần tài xế huỷ
    events: ['search', 'accept', 'driver_cancel', 'accept', 'arrive', 'driver_cancel', 'accept', 'arrive', 'pickup', 'start', 'complete'],
    ketThuc: 'completed',
  },
  { id: 'TC-ST-02', events: ['search', 'customer_cancel'], ketThuc: 'cancelled' },
  { id: 'TC-ST-03', events: ['search', 'accept', 'customer_cancel'], ketThuc: 'cancelled' },
  { id: 'TC-ST-04', events: ['search', 'accept', 'arrive', 'customer_cancel'], ketThuc: 'cancelled' },
  { id: 'TC-ST-05', events: ['search', 'system_cancel'], ketThuc: 'cancelled' },
];

/** Chạy một chuỗi trên bản cài đặt; ghi lại các chuyển hợp lệ đã đi qua (theo mô hình). */
export function chay(machine: Machine, events: readonly TripEvent[]): { cuoi: TripStatus; daDi: Transition[]; trangThai: TripStatus[] } {
  let cur: TripStatus = 'requested';
  const daDi: Transition[] = [];
  const trangThai: TripStatus[] = [cur];
  for (const e of events) {
    const next = machine(cur, e);
    const t = VALID.find((v) => v.from === cur && v.event === e && v.to === next);
    if (!t) throw new Error(`(${cur}, ${e}) ⇒ ${next} không có trong mô hình`);
    daDi.push(t);
    cur = next;
    trangThai.push(cur);
  }
  return { cuoi: cur, daDi, trangThai };
}

export function coverage(runs: { daDi: Transition[]; trangThai: TripStatus[] }[]) {
  const states = new Set(runs.flatMap((r) => r.trangThai));
  const trans = new Set(runs.flatMap((r) => r.daDi.map((t) => t.id)));
  return {
    allStates: `${states.size}/${STATES.length}`,
    validTransitions: `${trans.size}/${VALID.length}`,
    chuaPhu: VALID.filter((t) => !trans.has(t.id)).map((t) => t.id),
  };
}
