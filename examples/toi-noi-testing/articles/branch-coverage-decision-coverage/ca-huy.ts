/**
 * TEST-044 · Bộ ca phí huỷ của khách (SPEC R-CANCEL-01…06), viết thành bảng dữ liệu để chạy
 * trên mã thật `src/domain/cancel.ts` và trên bản tái cấu trúc có lỗi `cancel-v2.ts`.
 * Kết quả mong đợi lấy từ SPEC, không lấy từ code.
 */
import { expect, it } from 'vitest';
import type { CancellationFee, CancellationInput } from '../../src/domain/cancel';
import { vnTime } from '../../src/domain/time';

export type FeeFn = (input: CancellationInput) => CancellationFee;

const ACCEPTED_AT = vnTime(2026, 11, 4, 9, 0); // tài xế nhận lúc 09:00 giờ VN, ETA 5 phút
const sau = (phut: number, giay = 0) => new Date(ACCEPTED_AT.getTime() + (phut * 60 + giay) * 1000);
const nhan = { acceptedAt: ACCEPTED_AT, etaMinutes: 5 };

export interface CaHuy {
  id: string;
  ten: string;
  input: CancellationInput;
  mongDoi: CancellationFee | { loi: string };
}

export const CA_HUY: CaHuy[] = [
  { id: 'BC-01', ten: 'searching ⇒ 0 đ (R-CANCEL-01)', input: { status: 'searching', vehicle: 'bike', acceptedAt: null, etaMinutes: null, now: sau(0) }, mongDoi: { fee: 0, reason: 'searching' } },
  { id: 'BC-02', ten: 'accepted, 1 phút 59 giây ⇒ grace_period (R-CANCEL-02)', input: { status: 'accepted', vehicle: 'bike', ...nhan, now: sau(1, 59) }, mongDoi: { fee: 0, reason: 'grace_period' } },
  { id: 'BC-03', ten: 'accepted, đúng 2 phút ⇒ 10.000 đ (R-CANCEL-02/03)', input: { status: 'accepted', vehicle: 'bike', ...nhan, now: sau(2) }, mongDoi: { fee: 10_000, reason: 'charged' } },
  { id: 'BC-04', ten: 'arriving xe máy, 3 phút ⇒ 10.000 đ (R-CANCEL-01/03)', input: { status: 'arriving', vehicle: 'bike', ...nhan, now: sau(3) }, mongDoi: { fee: 10_000, reason: 'charged' } },
  { id: 'BC-05', ten: 'completed ⇒ INVALID_TRANSITION (R-CANCEL-01)', input: { status: 'completed', vehicle: 'bike', ...nhan, now: sau(30) }, mongDoi: { loi: 'INVALID_TRANSITION' } },
  { id: 'BC-06', ten: 'accepted thiếu acceptedAt ⇒ lỗi bất biến dữ liệu', input: { status: 'accepted', vehicle: 'bike', acceptedAt: null, etaMinutes: 5, now: sau(3) }, mongDoi: { loi: 'Error' } },
  { id: 'BC-07', ten: 'accepted, 15 phút 1 giây (ETA 5 + 10) ⇒ driver_late (R-CANCEL-04/06)', input: { status: 'accepted', vehicle: 'bike', ...nhan, now: sau(15, 1) }, mongDoi: { fee: 0, reason: 'driver_late' } },
  { id: 'BC-08', ten: 'arriving ô tô, 3 phút ⇒ 20.000 đ (R-CANCEL-03)', input: { status: 'arriving', vehicle: 'car', ...nhan, now: sau(3) }, mongDoi: { fee: 20_000, reason: 'charged' } },
  { id: 'BC-09', ten: 'arriving, đúng 15 phút ⇒ vẫn thu 10.000 đ (R-CANCEL-04)', input: { status: 'arriving', vehicle: 'bike', ...nhan, now: sau(15) }, mongDoi: { fee: 10_000, reason: 'charged' } },
];

export function kiemCa(fn: FeeFn, ca: CaHuy): void {
  if ('loi' in ca.mongDoi) {
    const loi = ca.mongDoi.loi;
    expect(() => fn(ca.input)).toThrow(loi === 'Error' ? Error : expect.objectContaining({ code: loi }));
  } else {
    expect(fn(ca.input)).toEqual(ca.mongDoi);
  }
}

/** Đăng ký các ca có id trong `ids` thành test vitest; `itFor` cho phép đánh dấu ca ghi nhận defect. */
type ItLike = (name: string, body: () => void) => void;

export function dangKy(fn: FeeFn, ids: readonly string[], itFor: (id: string) => ItLike = () => it): void {
  for (const ca of CA_HUY.filter((c) => ids.includes(c.id))) {
    itFor(ca.id)(`${ca.id} · ${ca.ten}`, () => kiemCa(fn, ca));
  }
}
