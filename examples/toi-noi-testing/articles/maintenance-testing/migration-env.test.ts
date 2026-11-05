/**
 * TEST-020 · Trigger «upgrade/migration of the operational environment».
 * Cùng một bộ regression chạy trên môi trường cũ (kho trong bộ nhớ) và môi trường mới
 * (PostgreSQL chạy bằng PGlite). Kết quả nghiệp vụ phải giống hệt nhau.
 */
import { beforeAll, beforeEach, describe, expect, it } from 'vitest';
import type { Actor } from '../../src/app/deps';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, cancelByCustomer, createTrip } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, createPgliteSystem, type LabSystem } from '../../src/infra/compose';

const c001: Actor = { role: 'customer', id: 'C001' };
const options = () => ({ clock: new FixedClock(vnTime(2026, 11, 4, 11, 30)), rainingCities: ['SG' as const] }); // cao điểm, SG mưa

const ENVIRONMENTS = [
  { name: 'môi trường cũ — kho bộ nhớ', make: async () => createMemorySystem(options()) },
  { name: 'môi trường mới — PostgreSQL (PGlite)', make: () => createPgliteSystem(options()) },
];

describe.each(ENVIRONMENTS)('TEST-020 · regression sau di trú: $name', ({ make }) => {
  let system: LabSystem;
  beforeAll(async () => {
    system = await make();
  });
  beforeEach(async () => {
    await system.reset();
  });

  const quoteR09 = () => createQuote(system.deps, c001, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4, voucherCode: 'GIAM20' });

  it('MT-E1 R-FARE-09 + R-QUOTE-04: chuyến lưu đúng 52.000 đ, đọc lại vẫn 52.000 đ', async () => {
    const trip = await createTrip(system.deps, c001, { quoteId: (await quoteR09()).id });
    expect((await system.deps.trips.findById(trip.id))?.fare).toBe(52_000);
  });

  it('MT-E2 R-QUOTE-03: một báo giá chỉ đặt được một chuyến', async () => {
    const quote = await quoteR09();
    await createTrip(system.deps, c001, { quoteId: quote.id });
    await expect(createTrip(system.deps, c001, { quoteId: quote.id })).rejects.toMatchObject({ code: 'QUOTE_ALREADY_USED' });
  });

  it('MT-E3 R-VOUCHER-07: hết lượt GIAM20 trong ngày, huỷ chuyến thì trả lại lượt', async () => {
    const trip = await createTrip(system.deps, c001, { quoteId: (await quoteR09()).id });
    await expect(quoteR09()).rejects.toMatchObject({ code: 'VOUCHER_USAGE_LIMIT' });
    await cancelByCustomer(system.deps, c001, trip.id);
    await expect(quoteR09()).resolves.toMatchObject({ details: { total: 52_000 } });
  });

  it('MT-E4 R-ACCEPT-03: tài xế thứ hai nhận cùng chuyến ⇒ TRIP_ALREADY_TAKEN', async () => {
    const trip = await createTrip(system.deps, c001, { quoteId: (await quoteR09()).id });
    await acceptTrip(system.deps, { role: 'driver', id: 'D101' }, trip.id, 5);
    await expect(acceptTrip(system.deps, { role: 'driver', id: 'D102' }, trip.id, 5)).rejects.toMatchObject({ code: 'TRIP_ALREADY_TAKEN' });
  });
});
