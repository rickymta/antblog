/**
 * Bài TEST-024 «Walkthrough» — sau buổi walkthrough luồng giao đồ ăn, mỗi câu hỏi đã có câu trả lời
 * trong SPEC được ghi thành một test (mã WT-Qn → mã yêu cầu). Walkthrough là static testing: test này
 * KHÔNG thay buổi review, nó chỉ khoá lại câu trả lời đã thống nhất để không bị hiểu lại khác đi.
 * Câu WT-Q8 (khách huỷ khi món đã làm — quán có được bù không) SPEC chưa trả lời ⇒ không có test,
 * chỉ có test cho phần SPEC đã nói (phí huỷ 10.000 đ).
 *
 * Chạy: npx vitest run articles/walkthrough --reporter=verbose
 */
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';

const C001 = 'customer:C001';
const D101 = 'driver:D101';

/** Đơn mẫu BA dùng để dẫn buổi walkthrough: SG, quán cách khách 3,2 km. */
const don = (items: { name: string; unitPrice: number; quantity: number }[], extra: object = {}) => ({
  kind: 'food',
  cityCode: 'SG',
  distanceKm: 3.2,
  items,
  ...extra,
});
const comTam = { name: 'Cơm tấm sườn', unitPrice: 45_000, quantity: 1 };
const traDa = { name: 'Trà đá', unitPrice: 5_000, quantity: 1 };

describe('TEST-024 · Câu hỏi walkthrough luồng giao đồ ăn (R-FOOD)', () => {
  let clock: FixedClock;
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;

  beforeEach(() => {
    clock = new FixedClock(vnTime(2026, 11, 4, 9, 0)); // 09:00 giờ VN — ngoài cao điểm, không mưa
    system = createMemorySystem({ clock });
    app = createApp(system.deps, { logErrors: false });
  });

  const baoGia = (body: object) => request(app).post('/api/quotes').set('x-user', C001).send(body);

  it('WT-Q1 · R-FOOD-03 · tiền món đúng 30.000 đ hợp lệ, 29.000 đ bị SUBTOTAL_TOO_LOW', async () => {
    await baoGia(don([{ name: 'Bánh mì', unitPrice: 15_000, quantity: 2 }])).expect(200);
    const res = await baoGia(don([{ name: 'Bánh mì', unitPrice: 29_000, quantity: 1 }])).expect(422);
    expect(res.body.error.code).toBe('SUBTOTAL_TOO_LOW');
  });

  it('WT-Q2 · R-FOOD-03 · ngưỡng 30.000 đ so với tiền món TRƯỚC voucher', async () => {
    const res = await baoGia(don([{ name: 'Bánh mì', unitPrice: 15_000, quantity: 2 }], { voucherCode: 'GIAM20' })).expect(200);
    expect(res.body.quote).toMatchObject({ itemsSubtotal: 30_000, itemsDiscount: 6_000 }); // còn 24.000 đ tiền món, vẫn hợp lệ
  });

  it('WT-Q3 · R-FOOD-04 · voucher không trừ vào phí giao', async () => {
    const khong = await baoGia(don([comTam, traDa])).expect(200);
    const co = await baoGia(don([comTam, traDa], { voucherCode: 'FOOD15' })).expect(200);
    expect(co.body.quote.deliveryFee).toBe(khong.body.quote.deliveryFee);
    expect(co.body.quote.fare.discount).toBe(0); // bảng chi tiết của phí giao không có tiền giảm
  });

  it('WT-Q4 · R-FOOD-05 · tối thiểu của voucher so với tiền món, không so với tổng', async () => {
    const res = await baoGia(don([{ name: 'Cơm tấm sườn', unitPrice: 49_000, quantity: 1 }], { voucherCode: 'FOOD15' })).expect(422);
    expect(res.body.error.code).toBe('VOUCHER_MIN_AMOUNT'); // tiền món 49.000 < 50.000 dù tổng có phí giao là 76.000
  });

  it('WT-Q5 · R-FOOD-04 · phí giao = công thức xe máy trên quãng đường quán → khách, làm tròn lên 1.000 đ', async () => {
    const res = await baoGia(don([comTam, traDa])).expect(200);
    expect(res.body.quote.fare).toMatchObject({ base: 26_400, total: 27_000 }); // 12.000 + 4.500 × 3,2
    expect(res.body.quote.deliveryFee).toBe(27_000);
  });

  it('WT-Q6 · R-FOOD-06 · đơn đồ ăn chọn ô tô bị VALIDATION_ERROR', async () => {
    const res = await baoGia(don([comTam, traDa], { vehicle: 'car' })).expect(400);
    expect(res.body.error.code).toBe('VALIDATION_ERROR');
  });

  it('WT-Q7 · R-FOOD-01 · tổng không làm tròn: 50.000 − 7.500 + 27.000 = 69.500 đ', async () => {
    const res = await baoGia(don([comTam, traDa], { voucherCode: 'FOOD15' })).expect(200);
    expect(res.body.quote).toMatchObject({ itemsSubtotal: 50_000, itemsDiscount: 7_500, deliveryFee: 27_000, total: 69_500 });
  });

  it('WT-Q8 (phần SPEC đã nói) · R-CANCEL-01/03 · khách huỷ khi tài xế đang tới quán ⇒ phí 10.000 đ', async () => {
    const q = await baoGia(don([comTam, traDa])).expect(200);
    const t = await request(app).post('/api/trips').set('x-user', C001).set('Idempotency-Key', 'wt-q8-dat-0001')
      .send({ quoteId: q.body.quote.quoteId }).expect(201);
    const id = t.body.trip.id;
    await request(app).post(`/api/trips/${id}/accept`).set('x-user', D101).send({ etaMinutes: 5 }).expect(200);
    await request(app).post(`/api/trips/${id}/arrive`).set('x-user', D101).expect(200);
    const huy = await request(app).post(`/api/trips/${id}/cancel`).set('x-user', C001).set('Idempotency-Key', 'wt-q8-huy-0001')
      .send({}).expect(200);
    expect(huy.body).toMatchObject({ cancellationFee: 10_000, feeReason: 'charged' });
  });
});
