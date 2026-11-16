/**
 * TCD-04 · phần 2 — kết quả mong đợi của lệnh huỷ là NHIỀU đầu ra quan sát được, không chỉ mã HTTP:
 * thân phản hồi (R-API-11, R-CANCEL-06), trạng thái lưu (R-CANCEL-07, R-STATE-07), nhật ký trip_events (R-STATE-06),
 * trạng thái tài xế (R-ACCEPT-05), lời gọi cổng thanh toán và khoá idempotency (R-PAY-04/05/06), gửi lại (R-IDEM-02).
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { oracleCuoc } from './oracle-cuoc';

const KHACH = 'customer:C001';
const T0 = vnTime(2026, 11, 4, 9, 0); // ngoài cao điểm, không mưa

/** Dựng chuyến đặt xe 5 km đã được nhận (ETA 5 phút), rồi dời đồng hồ `phut` phút sau lúc nhận. */
async function dungChuyen(opts: { xe?: 'bike' | 'car'; thanhToan: 'cash' | 'vixanh'; phut: number }) {
  const xe = opts.xe ?? 'bike';
  const clock = new FixedClock(T0);
  // hết giờ 50 ms thay cho 3 giây của R-PAY-04 để test nhanh; số lần thử lại giữ đúng SPEC (2)
  const system = createMemorySystem({ clock, config: { payment: { timeoutMs: 50, maxRetries: 2 } } });
  const app = createApp(system.deps, { logErrors: false });
  const q = await request(app).post('/api/quotes').set('x-user', KHACH).send({ kind: 'ride', cityCode: 'SG', vehicle: xe, distanceKm: 5 });
  const t = await request(app).post('/api/trips').set('x-user', KHACH).set('Idempotency-Key', 'dat-chuyen-0001')
    .send({ quoteId: q.body.quote.quoteId, paymentMethod: opts.thanhToan });
  const id: string = t.body.trip.id;
  const taiXe = xe === 'bike' ? 'D101' : 'D103';
  await request(app).post(`/api/trips/${id}/accept`).set('x-user', `driver:${taiXe}`).send({ etaMinutes: 5 }).expect(200);
  clock.advanceMinutes(opts.phut);
  const huy = () => request(app).post(`/api/trips/${id}/cancel`).set('x-user', KHACH).set('Idempotency-Key', `huy-${id}`).send({});
  return { clock, system, app, id, taiXe, huy, khoa: `trip:${id}:cancel-fee` };
}

const trangThaiNhatKy = async (s: ReturnType<typeof createMemorySystem>, id: string) =>
  (await s.deps.trips.listEvents(id)).map((e) => [e.status, e.occurredAt.toISOString()]);

describe('TCD-04 · 6. khách huỷ: kiểm mọi đầu ra quan sát được', () => {
  it('HUY-01 · vixanh, xe máy, 5 phút sau khi nhận: phản hồi + lưu trữ + nhật ký + tài xế + lời gọi ví', async () => {
    const c = await dungChuyen({ thanhToan: 'vixanh', phut: 5 });
    const now = c.clock.now().toISOString(); // 09:05 giờ VN = 02:05Z
    const res = await c.huy();

    // (1) phản hồi HTTP — R-API-11, R-CANCEL-01/03/06
    expect(res.status).toBe(200);
    expect(res.body).toMatchObject({ cancellationFee: 10_000, feeReason: 'charged' });
    expect(res.body.trip).toMatchObject({
      status: 'cancelled', cancelledBy: 'customer', cancelledAt: now, cancellationFee: 10_000,
      paymentStatus: 'paid', fare: Number(oracleCuoc({ xe: 'bike', km: 5, caoDiem: false, mua: false }).total.tu), // giá chuyến không đổi
    });
    // (2) trạng thái đã lưu — đọc lại qua API, không tin phản hồi của chính lệnh huỷ
    const doc = await request(c.app).get(`/api/trips/${c.id}`).set('x-user', KHACH).expect(200);
    expect(doc.body.trip).toEqual(res.body.trip);
    // (3) nhật ký trip_events — R-STATE-06: đúng một dòng mới, đúng thời điểm
    expect(await trangThaiNhatKy(c.system, c.id)).toEqual([
      ['requested', T0.toISOString()], ['searching', T0.toISOString()], ['accepted', T0.toISOString()], ['cancelled', now],
    ]);
    // (4) tài xế — R-ACCEPT-05
    expect((await c.system.deps.drivers.findById('D101'))?.status).toBe('online');
    // (5) lời gọi ra ngoài — R-PAY-05: đúng một lần, đúng số tiền, đúng khoá, đúng khách
    expect(c.system.payments.calls).toHaveLength(1);
    expect(c.system.payments.calls[0]).toMatchObject({ idempotencyKey: c.khoa, amount: 10_000, customerId: 'C001' });
    expect(c.system.payments.totalCharged()).toBe(10_000);
  });

  it('HUY-02 · gửi lại cùng Idempotency-Key: cùng mã và thân, Idempotent-Replayed, không gọi ví thêm, không thêm nhật ký', async () => {
    const c = await dungChuyen({ thanhToan: 'vixanh', phut: 5 });
    const lan1 = await c.huy();
    c.clock.advanceMinutes(1);
    const lan2 = await c.huy();
    expect(lan2.status).toBe(lan1.status);
    expect(lan2.body).toEqual(lan1.body);
    expect(lan2.headers['idempotent-replayed']).toBe('true');
    expect(c.system.payments.calls).toHaveLength(1);
    expect(await trangThaiNhatKy(c.system, c.id)).toHaveLength(4);
  });

  it('HUY-03 · ví hết giờ 2 lần rồi trả lời: 3 lời gọi CÙNG khoá, paid, chỉ trừ 10.000 đ một lần (R-PAY-04)', async () => {
    const c = await dungChuyen({ thanhToan: 'vixanh', phut: 5 });
    c.system.payments.failNext('timeout', 'timeout');
    const res = await c.huy();
    expect(res.body.trip.paymentStatus).toBe('paid');
    expect(c.system.payments.calls.map((x) => x.idempotencyKey)).toEqual([c.khoa, c.khoa, c.khoa]);
    expect(c.system.payments.totalCharged()).toBe(10_000);
  });

  it('HUY-04 · ví trừ tiền nhưng mất phản hồi cả 3 lần: failed nhưng tiền đã bị trừ đúng MỘT lần (oracle bộ phận)', async () => {
    const c = await dungChuyen({ thanhToan: 'vixanh', phut: 5 });
    c.system.payments.setMode('lost_response');
    const res = await c.huy();
    expect(res.status).toBe(200); // R-PAY-06: thanh toán không chặn huỷ
    expect(res.body.trip).toMatchObject({ status: 'cancelled', paymentStatus: 'failed' });
    expect(c.system.payments.calls).toHaveLength(3);
    expect(c.system.payments.totalCharged()).toBe(10_000); // bất biến «không trừ hai lần» (R-NFR-05)
  });

  it('HUY-05 · ví từ chối: 1 lời gọi, không thử lại, failed, chuyến vẫn cancelled (R-PAY-04, R-PAY-06)', async () => {
    const c = await dungChuyen({ thanhToan: 'vixanh', phut: 5 });
    c.system.payments.setMode('decline');
    const res = await c.huy();
    expect(res.status).toBe(200);
    expect(res.body.trip).toMatchObject({ status: 'cancelled', paymentStatus: 'failed', cancellationFee: 10_000 });
    expect(c.system.payments.calls).toHaveLength(1);
    expect(c.system.payments.totalCharged()).toBe(0);
  });

  it('HUY-06 · tiền mặt có phí: ghi nhận 10.000 đ, paymentStatus cash, KHÔNG gọi ví (R-PAY-05)', async () => {
    const c = await dungChuyen({ thanhToan: 'cash', phut: 5 });
    const res = await c.huy();
    expect(res.body).toMatchObject({ cancellationFee: 10_000, feeReason: 'charged', trip: { paymentStatus: 'cash' } });
    expect(c.system.payments.calls).toHaveLength(0);
  });

  it('HUY-07 · vixanh, 1 phút sau khi nhận: phí 0, grace_period, paymentStatus none, không gọi ví', async () => {
    const c = await dungChuyen({ thanhToan: 'vixanh', phut: 1 });
    const res = await c.huy();
    expect(res.body).toMatchObject({ cancellationFee: 0, feeReason: 'grace_period', trip: { paymentStatus: 'none', status: 'cancelled' } });
    expect(c.system.payments.calls).toHaveLength(0);
    expect((await c.system.deps.drivers.findById('D101'))?.status).toBe('online');
  });

  it('HUY-08 · ô tô vixanh: lời gọi ví 20.000 đ (R-CANCEL-03), tài xế D103 về online', async () => {
    const c = await dungChuyen({ xe: 'car', thanhToan: 'vixanh', phut: 5 });
    await c.huy();
    expect(c.system.payments.calls).toEqual([expect.objectContaining({ idempotencyKey: c.khoa, amount: 20_000 })]);
    expect((await c.system.deps.drivers.findById('D103'))?.status).toBe('online');
  });
});

describe('TCD-04 · 7. hoàn thành chuyến vixanh: số tiền thu lấy từ oracle, khoá theo R-PAY-03', () => {
  it('HT-01 · thu đúng fare 35.000 đ với khoá trip:<id>:fare', async () => {
    const c = await dungChuyen({ thanhToan: 'vixanh', phut: 3 });
    for (const buoc of ['arrive', 'pickup', 'start', 'complete']) {
      await request(c.app).post(`/api/trips/${c.id}/${buoc}`).set('x-user', 'driver:D101').send({}).expect(200);
    }
    const mongDoi = Number(oracleCuoc({ xe: 'bike', km: 5, caoDiem: false, mua: false }).total.tu);
    expect(mongDoi).toBe(35_000);
    expect(c.system.payments.calls).toEqual([expect.objectContaining({ idempotencyKey: `trip:${c.id}:fare`, amount: mongDoi })]);
    const doc = await request(c.app).get(`/api/trips/${c.id}`).set('x-user', KHACH).expect(200);
    expect(doc.body.trip).toMatchObject({ status: 'completed', paymentStatus: 'paid', completedAt: c.clock.now().toISOString() });
  });
});
