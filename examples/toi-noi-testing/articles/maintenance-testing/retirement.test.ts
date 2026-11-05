/**
 * TEST-020 · Trigger «retirement»: ngừng phục vụ Nha Trang (R-GEN-03).
 * Kiểm thử lưu trữ dữ liệu, khôi phục và truy xuất (CTFL v4.0.1 mục 2.3) trên PGlite.
 */
import { beforeAll, describe, expect, it } from 'vitest';
import type { PGlite } from '@electric-sql/pglite';
import type { Actor } from '../../src/app/deps';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, cancelByCustomer, createTrip, driverStep } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createPgliteSystem, type LabSystem } from '../../src/infra/compose';
import { labSeed } from '../../src/infra/seed';
import { exportCityArchive, findArchivedTrip, restoreArchive, type CityArchive } from './retire-city';

const c = (id: string): Actor => ({ role: 'customer', id });
const d601: Actor = { role: 'driver', id: 'D601' };

describe('TEST-020 · ngừng phục vụ NT — lưu trữ, khôi phục, truy xuất', () => {
  let system: LabSystem;
  let archive: CityArchive;
  let store: PGlite;
  const ids: Record<string, string> = {};

  beforeAll(async () => {
    const seed = labSeed();
    seed.drivers.push({ id: 'D601', fullName: 'Tài xế Nha Trang', cityCode: 'NT', vehicle: 'bike', status: 'online' });
    system = await createPgliteSystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)), seed });
    const trip = async (customer: string, cityCode: string, distanceKm: number) => {
      const quote = await createQuote(system.deps, c(customer), { kind: 'ride', cityCode, vehicle: 'bike', distanceKm });
      return createTrip(system.deps, c(customer), { quoteId: quote.id });
    };
    // NT-A: hoàn thành
    const a = await trip('C001', 'NT', 5);
    await acceptTrip(system.deps, d601, a.id, 5);
    for (const step of ['arrive', 'pickup', 'start', 'complete'] as const) await driverStep(system.deps, d601, a.id, step);
    // NT-B: khách huỷ khi đang tìm tài xế
    const b = await trip('C002', 'NT', 3);
    await cancelByCustomer(system.deps, c('C002'), b.id);
    // NT-C: còn đang tìm tài xế lúc chạy script (chuyến «đang bay»)
    const cTrip = await trip('C003', 'NT', 4);
    // SG-D: thành phố khác — không được lẫn vào bản lưu trữ
    const dTrip = await trip('C001', 'SG', 5);
    Object.assign(ids, { a: a.id, b: b.id, c: cTrip.id, d: dTrip.id });

    archive = JSON.parse(JSON.stringify(await exportCityArchive(system.db!, 'NT'))) as CityArchive; // mô phỏng ghi ra tệp rồi đọc lại
    store = await restoreArchive(archive);
  });

  it('MT-R1 chỉ lưu chuyến NT đã kết thúc; chuyến SG không lẫn vào', () => {
    expect(archive.trips.map((t) => [t.id, t.status])).toEqual([
      [ids.a, 'completed'],
      [ids.b, 'cancelled'],
    ]);
    expect(archive.trips.some((t) => t.id === ids.d)).toBe(false);
  });

  it('MT-R2 chuyến NT còn đang chạy bị chặn, không bị lưu trữ âm thầm', () => {
    expect(archive.blocked).toEqual([ids.c]);
  });

  it('MT-R3 tổng kiểm khớp nguồn: số chuyến và tổng cước (35.000 + 26.000)', async () => {
    const { rows } = await system.db!.query<{ n: number; s: number }>(
      `SELECT count(*)::int AS n, sum(fare)::int AS s FROM trips WHERE city_code = 'NT' AND status IN ('completed', 'cancelled')`,
    );
    expect(archive.checksum).toEqual({ count: rows[0]!.n, fareSum: rows[0]!.s });
    expect(archive.checksum).toEqual({ count: 2, fareSum: 61_000 });
  });

  it('MT-R4 khôi phục vào kho lưu trữ: số dòng và tổng cước giữ nguyên', async () => {
    const { rows } = await store.query<{ n: number; s: number }>('SELECT count(*)::int AS n, sum(fare)::int AS s FROM archived_trips');
    expect(rows[0]).toEqual({ n: 2, s: 61_000 });
  });

  it('MT-R5 truy xuất một chuyến sau lưu trữ: đủ giá, phiên bản giá và nhật ký trạng thái', async () => {
    const found = await findArchivedTrip(store, ids.a!);
    expect(found).toMatchObject({ status: 'completed', fare: 35_000, pricingVersion: 'fare-2026.1', customerId: 'C001' });
    expect(found?.events.map((e) => e.status)).toEqual([
      'requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed',
    ]);
  });
});
