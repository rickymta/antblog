/**
 * TEST-004 — tái hiện chuỗi error → defect → failure trên phí huỷ (SPEC R-CANCEL-01…03).
 * Chạy: npx vitest run articles/error-defect-failure-root-cause --reporter=verbose
 */
import { describe, expect, it } from 'vitest';
import { customerCancellationFee, type CancellationInput } from '../../src/domain/cancel';
import { vnTime } from '../../src/domain/time';
import { buggyCustomerCancellationFee } from './buggy-cancel';

const acceptedAt = vnTime(2026, 11, 4, 9, 0, 0); // tài xế nhận lúc 09:00:00 giờ VN
const after = (seconds: number) => new Date(acceptedAt.getTime() + seconds * 1000);
const trip = (seconds: number, status: CancellationInput['status'] = 'accepted'): CancellationInput => ({
  status,
  vehicle: 'bike',
  acceptedAt,
  etaMinutes: 5,
  now: after(seconds),
});

describe('TEST-004 · chuỗi error → defect → failure (phí huỷ R-CANCEL-02)', () => {
  it('ca kiểm thử theo đặc tả: huỷ đúng 2:00 ⇒ thu 10.000 đ (bản đúng của src/ đạt)', () => {
    expect(customerCancellationFee(trip(120))).toEqual({ fee: 10_000, reason: 'charged' });
  });

  it.fails('CÙNG ca kiểm thử chạy trên bản lỗi ⇒ đỏ (it.fails: kỳ vọng ca này thất bại)', () => {
    expect(buggyCustomerCancellationFee(trip(120))).toEqual({ fee: 10_000, reason: 'charged' });
  });

  it('failure quan sát được: bản lỗi trả 0 đ, lý do grace_period ở mốc 2:00', () => {
    expect(buggyCustomerCancellationFee(trip(120))).toEqual({ fee: 0, reason: 'grace_period' });
  });

  it('defect «ngủ»: ở 1:59, 2:01 và 5:00 hai bản cho cùng kết quả', () => {
    for (const s of [119, 121, 300]) {
      expect(buggyCustomerCancellationFee(trip(s))).toEqual(customerCancellationFee(trip(s)));
    }
  });

  it('quét từng giây 0:00 → 10:00: hai bản chỉ lệch nhau tại đúng 1 điểm (giây thứ 120)', () => {
    const lech: number[] = [];
    for (let s = 0; s <= 600; s++) {
      const dung = customerCancellationFee(trip(s));
      const loi = buggyCustomerCancellationFee(trip(s));
      if (dung.fee !== loi.fee || dung.reason !== loi.reason) lech.push(s);
    }
    expect(lech).toEqual([120]);
  });

  it('defect không được thực thi: chuyến đã ở arriving thì nhánh lỗi bị bỏ qua, không failure', () => {
    expect(buggyCustomerCancellationFee(trip(120, 'arriving'))).toEqual(customerCancellationFee(trip(120, 'arriving')));
    expect(buggyCustomerCancellationFee(trip(120, 'arriving'))).toEqual({ fee: 10_000, reason: 'charged' });
  });

  it('failure KHÔNG do defect: mã đúng, nhưng đồng hồ máy tính phí chạy nhanh 5 giây (giả định môi trường)', () => {
    const lechDongHo = 5_000; // máy B nhanh hơn máy A (máy ghi acceptedAt) 5 giây
    const thucTe = after(117); // khách bấm huỷ ở 1:57 thực tế — còn trong 2 phút miễn phí
    const nowMayB = new Date(thucTe.getTime() + lechDongHo); // máy B tưởng là 2:02
    const ketQua = customerCancellationFee({ ...trip(0), now: nowMayB });
    expect(ketQua).toEqual({ fee: 10_000, reason: 'charged' }); // khách bị thu phí oan
    expect(customerCancellationFee({ ...trip(0), now: thucTe })).toEqual({ fee: 0, reason: 'grace_period' });
  });
});
