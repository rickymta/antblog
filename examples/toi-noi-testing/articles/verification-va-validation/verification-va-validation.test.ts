/**
 * TEST-005 · Verification và Validation — /articles/verification-va-validation
 *
 * Phần 1 (verification): kiểm công thức cước có làm ĐÚNG ĐẶC TẢ R-FARE hay không.
 * Phần 2 (câu hỏi validation): mọi khẳng định đều ĐẠT đặc tả, nhưng kết quả cho thấy
 * một câu hỏi mà verification không trả lời được: khách có thật sự được lợi không?
 *
 * Không sửa src/ — chỉ đọc hành vi của lab.
 */
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { calculateFare } from '../../src/domain/fare';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';

const GIAM20 = { code: 'GIAM20', percent: 20, maxDiscount: 30_000, minAmount: 0 };

describe('Verification — cước có làm đúng đặc tả R-FARE không?', () => {
  it('V1 · R-FARE-09: ví dụ kiểm 7,4 km xe máy, cao điểm, mưa, GIAM20 ra đúng từng bước và 52.000 đ', () => {
    const fare = calculateFare({ vehicle: 'bike', distanceKm: 7.4, peak: true, raining: true, voucher: GIAM20 });
    expect(fare).toMatchObject({
      base: 45_300,
      afterPeak: 56_625,
      rainSurcharge: 8_000,
      beforeDiscount: 64_625,
      discount: 12_925,
      afterDiscount: 51_700,
      total: 52_000,
    });
  });

  it('V2 · R-FARE-03/05: phụ phí mưa cộng SAU hệ số cao điểm (không bị nhân 1,25)', () => {
    const peakOnly = calculateFare({ vehicle: 'bike', distanceKm: 7.4, peak: true, raining: false });
    const peakAndRain = calculateFare({ vehicle: 'bike', distanceKm: 7.4, peak: true, raining: true });
    expect(peakAndRain.beforeDiscount - peakOnly.beforeDiscount).toBe(8_000); // không phải 10.000
  });

  it('V3 · R-FARE-08: làm tròn LÊN bội số 1.000 đ; số đã tròn nghìn giữ nguyên', () => {
    expect(calculateFare({ vehicle: 'bike', distanceKm: 7.4, peak: false, raining: false }).total).toBe(46_000); // 45.300
    expect(calculateFare({ vehicle: 'bike', distanceKm: 2, peak: false, raining: false }).total).toBe(21_000); // 21.000
  });

  it('V4 · R-FARE-07: voucher không kéo cước xuống dưới mức tối thiểu 15.000 đ', () => {
    const fare = calculateFare({ vehicle: 'bike', distanceKm: 0.5, peak: false, raining: false, voucher: GIAM20 });
    expect(fare).toMatchObject({ beforeDiscount: 14_250, discount: 2_850, afterDiscount: 11_400, total: 15_000 });
  });
});

describe('Câu hỏi validation — đúng đặc tả, nhưng khách được gì?', () => {
  let clock: FixedClock;
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;
  const shortRide = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 0.5 };
  const longRide = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 };

  beforeEach(() => {
    clock = new FixedClock(vnTime(2026, 11, 4, 9, 0)); // 09:00 giờ VN, ngoài cao điểm, không mưa
    system = createMemorySystem({ clock });
    app = createApp(system.deps, { logErrors: false });
  });

  const quote = (body: object) => request(app).post('/api/quotes').set('x-user', 'customer:C001').send(body);

  it('Q1 · chuyến 0,5 km: có GIAM20 hay không, khách vẫn trả 15.000 đ — nhưng báo giá ghi «giảm 2.850 đ»', async () => {
    const without = (await quote(shortRide).expect(200)).body.quote;
    const withVoucher = (await quote({ ...shortRide, voucherCode: 'GIAM20' }).expect(200)).body.quote;
    expect(without).toMatchObject({ total: 15_000, discount: 0 });
    expect(withVoucher).toMatchObject({ total: 15_000, discount: 2_850, voucherCode: 'GIAM20' });
    console.log(
      `[Q1] không voucher: ${without.total} đ · có GIAM20: ${withVoucher.total} đ, «giảm» ${withVoucher.discount} đ ⇒ khách thực được ${without.total - withVoucher.total} đ`,
    );
  });

  it('Q2 · đặt chuyến 0,5 km bằng GIAM20 tiêu mất lượt trong ngày: chuyến 7,4 km sau đó bị VOUCHER_USAGE_LIMIT', async () => {
    const shortQuote = (await quote({ ...shortRide, voucherCode: 'GIAM20' }).expect(200)).body.quote;
    await request(app)
      .post('/api/trips')
      .set('x-user', 'customer:C001')
      .set('Idempotency-Key', 'tn005-short-0001')
      .send({ quoteId: shortQuote.quoteId })
      .expect(201);

    clock.set(vnTime(2026, 11, 4, 10, 0)); // 10:00 cùng ngày, vẫn ngoài cao điểm
    const longNoVoucher = (await quote(longRide).expect(200)).body.quote;
    const blocked = await quote({ ...longRide, voucherCode: 'GIAM20' });
    expect(blocked.status).toBe(422);
    expect(blocked.body.error.code).toBe('VOUCHER_USAGE_LIMIT');

    // Nếu khách giữ GIAM20 cho chuyến dài (tính bằng chính công thức của lab):
    const ifKept = calculateFare({ vehicle: 'bike', distanceKm: 7.4, peak: false, raining: false, voucher: GIAM20 });
    expect(longNoVoucher.total).toBe(46_000);
    expect(ifKept.total).toBe(37_000);
    console.log(
      `[Q2] chuyến 7,4 km lúc 10:00: không voucher ${longNoVoucher.total} đ · nếu còn GIAM20 ${ifKept.total} đ ⇒ khách mất cơ hội giảm ${longNoVoucher.total - ifKept.total} đ`,
    );
  });
});
