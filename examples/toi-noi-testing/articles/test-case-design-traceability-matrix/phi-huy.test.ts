/**
 * TCD-08 — test case cho R-CANCEL, có tag truy vết trong tiêu đề (xem trang-thai.test.ts).
 * Mốc: nhận 09:01:00, ETA 5 phút ⇒ mốc tài xế trễ = 09:16:00.
 */
import { describe, expect, it } from 'vitest';
import { cancelByCustomer } from '../../src/app/trips';
import { KHACH, NHAN_LUC, PHUT, dungChuyen } from './dung-chuyen';

const sauNhan = (ms: number) => new Date(NHAN_LUC.getTime() + ms);

async function huyLuc(opts: Parameters<typeof dungChuyen>[0], ms: number, ly_do?: string) {
  const c = await dungChuyen(opts);
  c.clock.set(sauNhan(ms));
  const kq = await cancelByCustomer(c.deps, KHACH, c.tripId, ly_do);
  return { ...c, kq, trip: (await c.deps.trips.findById(c.tripId))! };
}

describe('TCD-08 · R-CANCEL', () => {
  it('TC-CANCEL-001 @R-CANCEL-01 @R-CANCEL-06 @TCOND-CANCEL-01 · huỷ khi đang tìm tài xế ⇒ 0 đ, searching', async () => {
    const { kq } = await huyLuc({ toi: 'searching' }, 0);
    expect([kq.cancellationFee, kq.feeReason]).toEqual([0, 'searching']);
  });

  it('TC-CANCEL-002 @R-CANCEL-02 @TCOND-CANCEL-02 · accepted 1 phút ⇒ 0 đ, grace_period', async () => {
    const { kq } = await huyLuc({ toi: 'accepted' }, 1 * PHUT);
    expect([kq.cancellationFee, kq.feeReason]).toEqual([0, 'grace_period']);
  });

  it('TC-CANCEL-003 @R-CANCEL-02 @TCOND-CANCEL-03 · accepted 1:59,999 ⇒ vẫn 0 đ', async () => {
    const { kq } = await huyLuc({ toi: 'accepted' }, 2 * PHUT - 1);
    expect([kq.cancellationFee, kq.feeReason]).toEqual([0, 'grace_period']);
  });

  it('TC-CANCEL-004 @R-CANCEL-02 @R-CANCEL-03 @TCOND-CANCEL-03 @TCOND-CANCEL-06 · accepted đúng 2:00, xe máy ⇒ 10.000 đ', async () => {
    const { kq } = await huyLuc({ toi: 'accepted', xe: 'bike' }, 2 * PHUT);
    expect([kq.cancellationFee, kq.feeReason]).toEqual([10_000, 'charged']);
  });

  it('TC-CANCEL-005 @R-CANCEL-03 @TCOND-CANCEL-04 @TCOND-CANCEL-06 · accepted 5 phút, ô tô ⇒ 20.000 đ', async () => {
    const { kq } = await huyLuc({ toi: 'accepted', xe: 'car' }, 5 * PHUT);
    expect([kq.cancellationFee, kq.feeReason]).toEqual([20_000, 'charged']);
  });

  it('TC-CANCEL-006 @R-CANCEL-01 @R-CANCEL-02 @TCOND-CANCEL-05 · arriving 1 phút sau khi nhận ⇒ 10.000 đ, không có 2 phút miễn phí', async () => {
    const { kq } = await huyLuc({ toi: 'arriving', xe: 'bike' }, 1 * PHUT);
    expect([kq.cancellationFee, kq.feeReason]).toEqual([10_000, 'charged']);
  });

  it('TC-CANCEL-007 @R-CANCEL-03 @TCOND-CANCEL-06 · đơn đồ ăn tính phí như xe máy ⇒ 10.000 đ', async () => {
    const { kq, trip } = await huyLuc({ toi: 'accepted', dichVu: 'food' }, 4 * PHUT);
    expect(trip.vehicle).toBe('bike');
    expect(kq.cancellationFee).toBe(10_000);
  });

  it('TC-CANCEL-008 @R-CANCEL-04 @TCOND-CANCEL-07 · 20 phút sau khi nhận với ETA 5 ⇒ 0 đ, driver_late', async () => {
    const { kq } = await huyLuc({ toi: 'accepted' }, 20 * PHUT);
    expect([kq.cancellationFee, kq.feeReason]).toEqual([0, 'driver_late']);
  });

  it('TC-CANCEL-009 @R-CANCEL-07 @TCOND-CANCEL-10 · VíXanh: thu 20.000 đ đúng một lần với khoá cancel-fee, lưu vào chuyến', async () => {
    const { kq, trip, system, tripId } = await huyLuc({ toi: 'accepted', xe: 'car', thanhToan: 'vixanh' }, 5 * PHUT);
    expect(kq.cancellationFee).toBe(20_000);
    expect(trip).toMatchObject({ cancellationFee: 20_000, paymentStatus: 'paid' });
    expect(system.payments.calls).toEqual([expect.objectContaining({ idempotencyKey: `trip:${tripId}:cancel-fee`, amount: 20_000 })]);
  });

  // Viết theo một bản SPEC cũ: R-CANCEL-08 không còn tồn tại (lý do huỷ nay thuộc R-API-11) ⇒ mồ côi kiểu «trỏ vào mã không có».
  it('TC-CANCEL-090 @R-CANCEL-08 · lý do khách nhập được lưu vào cancelReason', async () => {
    const { trip } = await huyLuc({ toi: 'searching' }, 0, 'Đổi kế hoạch');
    expect(trip.cancelReason).toBe('Đổi kế hoạch');
  });
});
