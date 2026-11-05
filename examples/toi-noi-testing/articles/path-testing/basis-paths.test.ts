/**
 * TEST-048 · Path testing: basis path cho nextStatus (trip-state.ts) và acceptTrip (trips.ts).
 * Chạy: npx vitest run articles/path-testing --reporter=verbose
 */
import { describe, expect, it, vi } from 'vitest';
import type { AppDeps } from '../../src/app/deps';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, createTrip } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import { nextStatus, TRANSITIONS, TRIP_EVENTS, TRIP_STATUSES } from '../../src/domain/trip-state';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { countPaths, cyclomatic, EDGES, edgeVector, NODES, rank, walk } from './cfg';

const AT = vnTime(2026, 11, 4, 9, 0);
const C001 = { role: 'customer' as const, id: 'C001' };
const drv = (id: string) => ({ role: 'driver' as const, id });
const err = (code: string) => expect.objectContaining({ code });

/** Hệ thống mới + một chuyến SG xe máy đang `searching`. */
async function setup() {
  const system = createMemorySystem({ clock: new FixedClock(AT) });
  const deps = system.deps;
  const newTrip = async () => {
    const quote = await createQuote(deps, C001, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 });
    return createTrip(deps, C001, { quoteId: quote.id });
  };
  return { deps, trip: await newTrip(), newTrip };
}

// ------------------------------------------------------------ nextStatus: V(G) = 2

describe('nextStatus — V(G) = 2 ⇒ 2 basis path', () => {
  it('P1 · cặp có trong bảng: searching + accept ⇒ accepted', () => {
    expect(nextStatus('searching', 'accept')).toBe('accepted');
  });
  it('P2 · cặp không có trong bảng: completed + accept ⇒ INVALID_TRANSITION', () => {
    expect(() => nextStatus('completed', 'accept')).toThrow(err('INVALID_TRANSITION'));
  });
  it('100% path của hàm chỉ chạm 2 trong 72 cặp (trạng thái, sự kiện) của bảng', () => {
    const pairs = TRIP_STATUSES.flatMap((s) => TRIP_EVENTS.map((ev) => TRANSITIONS[s][ev]));
    expect({ pairs: pairs.length, valid: pairs.filter((t) => t !== undefined).length }).toEqual({ pairs: 72, valid: 12 });
  });
});

// ------------------------------------------------------------ acceptTrip: CFG và tập basis

/** Lựa chọn tại các nút quyết định của từng basis path (B0 = đường cơ sở, mỗi đường sau lật một quyết định). */
const BASIS: Record<string, Record<string, string>> = {
  B0: { D1: 'F', D2a: 'F', D3: 'F', D4: 'F', D5: 'F', D7: 'F' },
  B1: { D1: 'T' },
  B2: { D1: 'F', D2a: 'T', D2b: 'T' },
  B3: { D1: 'F', D2a: 'T', D2b: 'F', D3: 'F', D4: 'F', D5: 'F', D7: 'F' },
  B4: { D1: 'F', D2a: 'F', D3: 'T' },
  B5: { D1: 'F', D2a: 'F', D3: 'F', D4: 'T' },
  B6: { D1: 'F', D2a: 'F', D3: 'F', D4: 'F', D5: 'T', D6: 'co' },
  B7: { D1: 'F', D2a: 'F', D3: 'F', D4: 'F', D5: 'T', D6: 'null' },
  B8: { D1: 'F', D2a: 'F', D3: 'F', D4: 'F', D5: 'F', D7: 'T', D8: 'T' },
  B9: { D1: 'F', D2a: 'F', D3: 'F', D4: 'F', D5: 'F', D7: 'T', D8: 'F' },
};

describe('acceptTrip — đồ thị luồng điều khiển', () => {
  it('N = 23, E = 31 ⇒ V(G) = 10 (khớp ESLint complexity 10)', () => {
    expect({ N: NODES.length, E: EDGES.length, V: cyclomatic(NODES, EDGES) }).toEqual({ N: 23, E: 31, V: 10 });
  });
  it('10 đường B0–B9 độc lập tuyến tính (hạng 10) ⇒ là một tập basis', () => {
    const vectors = Object.values(BASIS).map((c) => edgeVector(walk(c)));
    expect(rank(vectors)).toBe(10);
  });
  it('thêm một đường bất kỳ (B2 lật D2b rồi đi tiếp như B8) không tăng hạng — nó là tổ hợp tuyến tính', () => {
    const extra = edgeVector(walk({ D1: 'F', D2a: 'T', D2b: 'F', D3: 'F', D4: 'F', D5: 'F', D7: 'T', D8: 'T' }));
    const vectors = Object.values(BASIS).map((c) => edgeVector(walk(c)));
    expect(rank([...vectors, extra])).toBe(10);
  });
  it('tổng số đường đi hoàn chỉnh = 16 (> 10 basis path)', () => {
    expect(countPaths()).toBe(16);
  });
});

describe('acceptTrip — mỗi basis path một test', () => {
  it('B0 · đường cơ sở: D101 nhận chuyến SG xe máy ⇒ accepted, tài xế busy', async () => {
    const { deps, trip } = await setup();
    const got = await acceptTrip(deps, drv('D101'), trip.id, 5);
    expect({ status: got.status, driverId: got.driverId, driver: (await deps.drivers.findById('D101'))?.status }).toEqual({
      status: 'accepted',
      driverId: 'D101',
      driver: 'busy',
    });
  });

  it('B1 · D1 = T: chuyến không tồn tại ⇒ TRIP_NOT_FOUND', async () => {
    const { deps } = await setup();
    await expect(acceptTrip(deps, drv('D101'), 'khong-co', 5)).rejects.toThrow(err('TRIP_NOT_FOUND'));
  });

  it('B2 · D2a = T, D2b = T: nhận lại chuyến mình đang giữ ⇒ trả chuyến cũ, giữ etaMinutes cũ (R-ACCEPT-07)', async () => {
    const { deps, trip } = await setup();
    await acceptTrip(deps, drv('D101'), trip.id, 5);
    const again = await acceptTrip(deps, drv('D101'), trip.id, 30);
    expect({ status: again.status, eta: again.etaMinutes }).toEqual({ status: 'accepted', eta: 5 });
  });

  it('B3 · D2a = T, D2b = F: chuyến searching mà driverId đã là D101 (dữ liệu vi phạm bất biến, dựng tay) ⇒ vẫn nhận bình thường', async () => {
    const { deps, trip } = await setup();
    await deps.trips.updateIfStatus({ ...trip, driverId: 'D101' }, 'searching');
    const got = await acceptTrip(deps, drv('D101'), trip.id, 7);
    expect({ status: got.status, eta: got.etaMinutes }).toEqual({ status: 'accepted', eta: 7 });
  });

  it('B4 · D3 = T: tài xế HN nhận chuyến SG ⇒ CITY_MISMATCH', async () => {
    const { deps, trip } = await setup();
    await expect(acceptTrip(deps, drv('D201'), trip.id, 5)).rejects.toThrow(err('CITY_MISMATCH'));
  });

  it('B5 · D4 = T: tài xế ô tô nhận chuyến xe máy ⇒ VEHICLE_MISMATCH', async () => {
    const { deps, trip } = await setup();
    await expect(acceptTrip(deps, drv('D103'), trip.id, 5)).rejects.toThrow(err('VEHICLE_MISMATCH'));
  });

  it('B6 · D5 = T, D6 = có: D101 nhận HAI chuyến cùng lúc ⇒ đúng một thành công, lệnh kia DRIVER_BUSY (R-ACCEPT-03)', async () => {
    const { deps, trip, newTrip } = await setup();
    const trip2 = await newTrip();
    const results = await Promise.allSettled([acceptTrip(deps, drv('D101'), trip.id, 5), acceptTrip(deps, drv('D101'), trip2.id, 5)]);
    expect(results.map((r) => (r.status === 'fulfilled' ? 'accepted' : (r.reason as { code: string }).code))).toEqual(['accepted', 'DRIVER_BUSY']);
  });

  it('B7 · D5 = T, D6 = null: tài xế "biến mất" sau khi giữ chỗ thất bại (stub) ⇒ DRIVER_BUSY ở dòng 174', async () => {
    const { deps, trip } = await setup();
    const real = deps.drivers;
    const findById = vi.fn(real.findById.bind(real)).mockImplementationOnce(real.findById.bind(real)).mockResolvedValueOnce(null);
    const stubbed: AppDeps = { ...deps, drivers: { ...real, findById, tryMarkBusy: async () => false, markOnline: real.markOnline.bind(real) } };
    await expect(acceptTrip(stubbed, drv('D101'), trip.id, 5)).rejects.toThrow(err('DRIVER_BUSY'));
    expect(findById).toHaveBeenCalledTimes(2);
  });

  it('B8 · D7 = T, D8 = T: D101 và D102 cùng nhận một chuyến ⇒ một người thắng, người thua TRIP_ALREADY_TAKEN và về online', async () => {
    const { deps, trip } = await setup();
    const results = await Promise.allSettled([acceptTrip(deps, drv('D101'), trip.id, 5), acceptTrip(deps, drv('D102'), trip.id, 5)]);
    expect({
      outcomes: results.map((r) => (r.status === 'fulfilled' ? 'accepted' : (r.reason as { code: string }).code)),
      loser: (await deps.drivers.findById('D102'))?.status,
    }).toEqual({ outcomes: ['accepted', 'TRIP_ALREADY_TAKEN'], loser: 'online' });
  });

  it('B9 · D7 = T, D8 = F: chuyến "biến mất" sau khi giành thất bại (stub) ⇒ TRIP_ALREADY_TAKEN, tài xế về online', async () => {
    const { deps, trip } = await setup();
    const real = deps.trips;
    const findById = vi.fn(real.findById.bind(real)).mockImplementationOnce(real.findById.bind(real)).mockResolvedValueOnce(null);
    const stubbed: AppDeps = {
      ...deps,
      trips: Object.assign(Object.create(Object.getPrototypeOf(real)), real, { findById, acceptIfSearching: async () => null }),
    };
    await expect(acceptTrip(stubbed, drv('D101'), trip.id, 5)).rejects.toThrow(err('TRIP_ALREADY_TAKEN'));
    expect((await deps.drivers.findById('D101'))?.status).toBe('online');
  });
});
