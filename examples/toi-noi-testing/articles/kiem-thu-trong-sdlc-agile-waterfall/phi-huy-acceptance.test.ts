/**
 * Bài TEST-008 «Kiểm thử trong SDLC: Waterfall, Agile và DevOps» — minh hoạ test-first (ATDD/BDD):
 * tiêu chí chấp nhận của câu chuyện người dùng «Khách huỷ chuyến» (SPEC R-CANCEL) được viết thành
 * kịch bản Given/When/Then trong buổi làm rõ yêu cầu, rồi chuyển thành test tự động chạy ở bước
 * «Unit tests» của pipeline CI. Sau sprint, các test này ở lại làm regression test.
 *
 * Chạy: npx vitest run articles/kiem-thu-trong-sdlc-agile-waterfall
 */
import { describe, expect, it } from 'vitest';
import { customerCancellationFee, type CancellationInput } from '../../src/domain/cancel';
import { vnTime } from '../../src/domain/time';

const acceptedAt = vnTime(2026, 11, 4, 9, 0); // tài xế nhận lúc 09:00 giờ VN
const sau = (phut: number, giay = 0) => new Date(acceptedAt.getTime() + (phut * 60 + giay) * 1000);
const chuyen = (patch: Partial<CancellationInput>): CancellationInput => ({
  status: 'accepted',
  vehicle: 'bike',
  acceptedAt,
  etaMinutes: 5,
  now: sau(0),
  ...patch,
});

describe('Tính năng: Phí huỷ chuyến của khách (R-CANCEL)', () => {
  it('AC-1 · R-CANCEL-01 · Given chuyến đang tìm tài xế, When khách huỷ, Then phí 0 đ, lý do searching', () => {
    const input = chuyen({ status: 'searching', acceptedAt: null, etaMinutes: null });
    expect(customerCancellationFee(input)).toEqual({ fee: 0, reason: 'searching' });
  });

  it('AC-2 · R-CANCEL-02 · Given tài xế nhận được 1 phút 59 giây, When khách huỷ, Then miễn phí (grace_period)', () => {
    expect(customerCancellationFee(chuyen({ now: sau(1, 59) }))).toEqual({ fee: 0, reason: 'grace_period' });
  });

  it('AC-3 · R-CANCEL-02/03 · Given tài xế xe máy nhận được đúng 2 phút, When khách huỷ, Then phí 10.000 đ', () => {
    expect(customerCancellationFee(chuyen({ now: sau(2) }))).toEqual({ fee: 10_000, reason: 'charged' });
  });

  it('AC-4 · R-CANCEL-03 · Given chuyến ô tô đang arriving, When khách huỷ, Then phí 20.000 đ', () => {
    const input = chuyen({ status: 'arriving', vehicle: 'car', now: sau(3) });
    expect(customerCancellationFee(input)).toEqual({ fee: 20_000, reason: 'charged' });
  });

  it('AC-5 · R-CANCEL-04 · Given ETA 5 phút, When khách huỷ đúng phút 15, Then vẫn thu phí', () => {
    expect(customerCancellationFee(chuyen({ status: 'arriving', now: sau(15) }))).toEqual({ fee: 10_000, reason: 'charged' });
  });

  it('AC-6 · R-CANCEL-04 · Given ETA 5 phút, When khách huỷ ở phút 15 giây 1, Then miễn phí (driver_late)', () => {
    expect(customerCancellationFee(chuyen({ status: 'arriving', now: sau(15, 1) }))).toEqual({ fee: 0, reason: 'driver_late' });
  });

  it('AC-7 · R-CANCEL-01 · Given chuyến đang in_transit, When khách huỷ, Then bị từ chối INVALID_TRANSITION', () => {
    expect(() => customerCancellationFee(chuyen({ status: 'in_transit', now: sau(20) }))).toThrow(
      expect.objectContaining({ code: 'INVALID_TRANSITION' }),
    );
  });
});
