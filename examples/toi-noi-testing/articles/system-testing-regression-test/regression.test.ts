import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { calculateFare, type FareInput } from '../../src/domain/fare';
import { createApp } from '../../src/api/app';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { vnTime } from '../../src/domain/time';

// Candidate lỗi GIẢ LẬP: refactor nhầm phí mưa thành phụ phí trong base.
// Không sửa implementation dùng chung.
function buggyRainRefactor(input: FareInput) {
  return calculateFare({ ...input, raining: false,
    parcelSurcharge: (input.parcelSurcharge ?? 0) + (input.raining ? 8_000 : 0) });
}

describe('SYS-06 · regression cho refactor công thức cước dùng chung', () => {
  it.each([
    { id: 'RG-01', peak: false, raining: false, total: 35_000 },
    { id: 'RG-02', peak: false, raining: true, total: 43_000 },
    { id: 'RG-03', peak: true, raining: false, total: 44_000 },
    { id: 'RG-04', peak: true, raining: true, total: 52_000 },
  ])('$id · R-FARE-03/05: peak=$peak rain=$raining', ({ peak, raining, total }) => {
    expect(calculateFare({ vehicle: 'bike', distanceKm: 5, peak, raining }).total).toBe(total);
  });

  it('RG-05 · fixture: một ca thường không phát hiện lỗi, ca tương tác phát hiện được', () => {
    const input: FareInput = { vehicle: 'bike', distanceKm: 5, peak: false, raining: true };
    expect(buggyRainRefactor(input).total).toBe(43_000);
    expect(buggyRainRefactor({ ...input, peak: true }).total).toBe(54_000);
    expect(buggyRainRefactor({ ...input, peak: true }).total).not.toBe(52_000);
    expect(calculateFare({ ...input, peak: true }).total).toBe(52_000);
  });

  let app: ReturnType<typeof createApp>;
  beforeEach(() => {
    const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 11, 30)), rainingCities: ['SG'] });
    app = createApp(system.deps, { logErrors: false });
  });

  it('RG-06 · R-FARE-09: oracle chuẩn 7,4 km + cao điểm + mưa + GIAM20', async () => {
    const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send({
      kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4, voucherCode: 'GIAM20',
    }).expect(200);
    expect(res.body.quote.fare).toMatchObject({ beforeDiscount: 64_625, discount: 12_925, total: 52_000 });
  });

  it('RG-07 · R-PARCEL-04: phí khối lượng nhân cao điểm nhưng phí mưa không nhân', async () => {
    const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send({
      kind: 'parcel', cityCode: 'SG', vehicle: 'bike', distanceKm: 5, parcelWeightKg: 6,
    }).expect(200);
    expect(res.body.quote.fare).toMatchObject({ parcelSurcharge: 2_000, base: 36_500, afterPeak: 45_625, rainSurcharge: 8_000, total: 54_000 });
  });

  it('RG-08 · R-FOOD-04/05: voucher chỉ giảm tiền món, giữ phí giao không giảm', async () => {
    const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send({
      kind: 'food', cityCode: 'SG', distanceKm: 5,
      items: [{ name: 'Cơm', unitPrice: 60_000, quantity: 1 }], voucherCode: 'FOOD15',
    }).expect(200);
    expect(res.body.quote).toMatchObject({ itemsSubtotal: 60_000, itemsDiscount: 9_000, deliveryFee: 52_000, total: 103_000 });
    expect(res.body.quote.fare.discount).toBe(0);
  });
});
