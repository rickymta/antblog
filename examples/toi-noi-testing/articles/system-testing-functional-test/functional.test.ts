import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { vnTime } from '../../src/domain/time';

describe('SYS-03 · functional: yêu cầu → dữ liệu → oracle', () => {
  let clock: FixedClock;
  let system: ReturnType<typeof createMemorySystem>;
  let app: ReturnType<typeof createApp>;
  const ride = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 };
  beforeEach(() => {
    clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
    system = createMemorySystem({ clock });
    app = createApp(system.deps, { logErrors: false });
  });
  const quote = (extra: object = {}) => request(app).post('/api/quotes')
    .set('x-user', 'customer:C001').send({ ...ride, ...extra });
  const book = (quoteId: string) => request(app).post('/api/trips')
    .set('x-user', 'customer:C001').set('Idempotency-Key', 'sys03-book-001').send({ quoteId });

  it.each([
    { raw: 0.44, normalized: null }, { raw: 0.45, normalized: 0.5 },
    { raw: 0.5, normalized: 0.5 }, { raw: 5, normalized: 5 },
    { raw: 7.35, normalized: 7.4 }, { raw: 100, normalized: 100 },
    { raw: 100.04, normalized: 100 }, { raw: 100.05, normalized: null },
  ])('FN-01 · R-FARE-02: $raw km → $normalized', async ({ raw, normalized }) => {
    const res = await quote({ distanceKm: raw });
    if (normalized === null) {
      expect(res.status).toBe(422);
      expect(res.body.error.code).toBe('DISTANCE_OUT_OF_RANGE');
    } else {
      expect(res.status).toBe(200);
      expect(res.body.quote.distanceKm).toBe(normalized);
    }
  });

  // Oracle tính trước từ SPEC, không gọi calculator để sinh expected.
  it.each([
    { peak: false, rain: false, voucher: false, before: 34_500, discount: 0, total: 35_000 },
    { peak: false, rain: false, voucher: true, before: 34_500, discount: 6_900, total: 28_000 },
    { peak: false, rain: true, voucher: false, before: 42_500, discount: 0, total: 43_000 },
    { peak: false, rain: true, voucher: true, before: 42_500, discount: 8_500, total: 34_000 },
    { peak: true, rain: false, voucher: false, before: 43_125, discount: 0, total: 44_000 },
    { peak: true, rain: false, voucher: true, before: 43_125, discount: 8_625, total: 35_000 },
    { peak: true, rain: true, voucher: false, before: 51_125, discount: 0, total: 52_000 },
    { peak: true, rain: true, voucher: true, before: 51_125, discount: 10_225, total: 41_000 },
  ])('FN-02 · R-FARE-03/06/08: peak=$peak rain=$rain voucher=$voucher', async (row) => {
    if (row.peak) clock.set(vnTime(2026, 11, 4, 11, 30));
    system.weather.setRaining(row.rain ? ['SG'] : []);
    const res = await quote(row.voucher ? { voucherCode: 'GIAM20' } : {}).expect(200);
    expect(res.body.quote.fare).toMatchObject({
      beforeDiscount: row.before, discount: row.discount, total: row.total,
    });
    expect(res.body.quote.total).toBe(row.total);
  });

  it.each([
    { input: { distanceKm: '5' }, status: 400, code: 'VALIDATION_ERROR' },
    { input: { cityCode: 'XX' }, status: 422, code: 'CITY_NOT_SUPPORTED' },
    { input: { cityCode: 'XX', distanceKm: 0.1 }, status: 422, code: 'CITY_NOT_SUPPORTED' },
    { input: { distanceKm: 0.1, voucherCode: 'KHONGCO' }, status: 422, code: 'DISTANCE_OUT_OF_RANGE' },
  ])('FN-03 · R-VALID-01: $code với $input', async ({ input, status, code }) => {
    const res = await quote(input).expect(status);
    expect(res.body.error.code).toBe(code);
  });

  it('FN-04 · R-FARE-07: voucher không kéo giá xuống dưới cước tối thiểu', async () => {
    const res = await quote({ distanceKm: 0.5, voucherCode: 'GIAM20' }).expect(200);
    expect(res.body.quote.fare).toMatchObject({ beforeDiscount: 14_250, discount: 2_850, afterDiscount: 11_400, total: 15_000 });
  });

  it('FN-05 · R-VOUCHER-06: giảm phần trăm bị chặn ở trần 30.000', async () => {
    const res = await quote({ distanceKm: 100, voucherCode: 'GIAM20' }).expect(200);
    expect(res.body.quote).toMatchObject({ discount: 30_000, total: 432_000 });
  });

  it('FN-06 · R-QUOTE-04: điều kiện mưa đổi sau báo giá không đổi giá chuyến', async () => {
    const res = await quote().expect(200);
    system.weather.setRaining(['SG']);
    const trip = await book(res.body.quote.quoteId).expect(201);
    expect(trip.body.trip).toMatchObject({ fare: 35_000, raining: false });
    const next = await quote().expect(200);
    expect(next.body.quote).toMatchObject({ total: 43_000, raining: true });
  });

  it('FN-07 · R-STATE-02/03: từ accepted không được complete; trạng thái giữ nguyên', async () => {
    const q = await quote().expect(200);
    const created = await book(q.body.quote.quoteId).expect(201);
    const id = created.body.trip.id;
    await request(app).post(`/api/trips/${id}/accept`).set('x-user', 'driver:D101').send({ etaMinutes: 5 }).expect(200);
    const invalid = await request(app).post(`/api/trips/${id}/complete`).set('x-user', 'driver:D101').send({}).expect(409);
    expect(invalid.body.error.code).toBe('INVALID_TRANSITION');
    const read = await request(app).get(`/api/trips/${id}`).set('x-user', 'customer:C001').expect(200);
    expect(read.body.trip.status).toBe('accepted');
    expect(system.payments.calls).toHaveLength(0);
  });
});
