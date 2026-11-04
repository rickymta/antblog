import { beforeAll, beforeEach, describe, expect, it } from 'vitest';
import type { Actor } from '../../src/app/deps';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, createTrip } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createPgliteSystem, type LabSystem } from '../../src/infra/compose';

const customer: Actor = { role: 'customer', id: 'C001' };
const d101: Actor = { role: 'driver', id: 'D101' };
const d102: Actor = { role: 'driver', id: 'D102' };

describe('smoke · PGlite (PostgreSQL thật, một kết nối)', () => {
  const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
  let system: LabSystem;

  beforeAll(async () => {
    system = await createPgliteSystem({ clock });
  });
  beforeEach(async () => {
    await system.reset(); // mỗi test bắt đầu từ dữ liệu mẫu
  });

  async function newTrip() {
    const quote = await createQuote(system.deps, customer, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 });
    return createTrip(system.deps, customer, { quoteId: quote.id });
  }

  it('lưu chuyến và nhật ký requested → searching', async () => {
    const trip = await newTrip();
    expect(await system.deps.trips.findById(trip.id)).toMatchObject({ status: 'searching', fare: 35_000, distanceKm: 5 });
    expect((await system.deps.trips.listEvents(trip.id)).map((e) => e.status)).toEqual(['requested', 'searching']);
  });

  it('R-ACCEPT-03: lần nhận thứ hai bị từ chối TRIP_ALREADY_TAKEN, tài xế thua vẫn online', async () => {
    const trip = await newTrip();
    await acceptTrip(system.deps, d101, trip.id, 5);
    await expect(acceptTrip(system.deps, d102, trip.id, 5)).rejects.toMatchObject({ code: 'TRIP_ALREADY_TAKEN' });
    expect((await system.deps.drivers.findById('D101'))?.status).toBe('busy');
    expect((await system.deps.drivers.findById('D102'))?.status).toBe('online');
  });

  it('R-ACCEPT-03: hai lệnh nhận chạy xen kẽ ⇒ UPDATE có điều kiện chỉ cho một người thắng', async () => {
    const trip = await newTrip();
    const results = await Promise.allSettled([acceptTrip(system.deps, d101, trip.id, 5), acceptTrip(system.deps, d102, trip.id, 5)]);
    expect(results.filter((r) => r.status === 'fulfilled')).toHaveLength(1);
    const { rows } = await system.db!.query<{ n: number }>(`SELECT count(*)::int AS n FROM trip_events WHERE trip_id = $1 AND status = 'accepted'`, [trip.id]);
    expect(rows[0]?.n).toBe(1);
  });

  it('R-QUOTE-03: một báo giá chỉ đặt được một chuyến', async () => {
    const quote = await createQuote(system.deps, customer, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 });
    await createTrip(system.deps, customer, { quoteId: quote.id });
    await expect(createTrip(system.deps, customer, { quoteId: quote.id })).rejects.toMatchObject({ code: 'QUOTE_ALREADY_USED' });
  });
});
