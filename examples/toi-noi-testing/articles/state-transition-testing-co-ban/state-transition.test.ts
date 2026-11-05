/**
 * TEST-034 — State Transition Testing cơ bản trên vòng đời chuyến (SPEC R-STATE-01…06, R-API-10/11, R-ACCEPT-07).
 * Chạy: npx vitest run articles/state-transition-testing-co-ban --reporter=verbose
 */
import { describe, expect, it } from 'vitest';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, cancelByCustomer, cancelByDriver, createTrip, driverStep, expireStaleSearches } from '../../src/app/trips';
import { DomainError } from '../../src/domain/errors';
import { vnTime } from '../../src/domain/time';
import { nextStatus, type TripEvent, type TripStatus } from '../../src/domain/trip-state';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { chay, coverage, EVENTS, STATES, TEST_CASES, VALID, validTarget, type Machine } from './bang-trang-thai';
import { buggyNextStatus } from './buggy-trip-state';

// ---------------------------------------------------------------- tầng domain: hàm chuyển thuần

const caBang = STATES.flatMap((from) => EVENTS.map((event) => ({ from, event, to: validTarget(from, event) })));
const ketQua = (m: Machine, from: TripStatus, e: TripEvent): string => {
  try {
    return m(from, e);
  } catch (err) {
    if (err instanceof DomainError) return err.code;
    throw err;
  }
};

describe('TEST-034 · mô hình (state table) lấy từ R-STATE-02', () => {
  it('8 trạng thái × 9 sự kiện = 72 ô: 12 chuyển hợp lệ + 60 ô trống (không hợp lệ)', () => {
    expect(caBang).toHaveLength(72);
    expect(caBang.filter((c) => c.to !== null)).toHaveLength(12);
    expect(caBang.filter((c) => c.to === null)).toHaveLength(60);
  });
});

describe('TEST-034 · 5 test case đạt 100% all states + 100% valid transitions (domain)', () => {
  it.each(TEST_CASES.map((tc) => [tc.id, tc] as const))('%s', (_id, tc) => {
    expect(chay(nextStatus, tc.events).cuoi).toBe(tc.ketThuc);
  });

  it('coverage của cả bộ: all states 8/8, valid transitions 12/12', () => {
    const cov = coverage(TEST_CASES.map((tc) => chay(nextStatus, tc.events)));
    expect(cov).toEqual({ allStates: '8/8', validTransitions: '12/12', chuaPhu: [] });
  });

  it('chỉ luồng chính 6 sự kiện (không huỷ): valid transitions 6/12 — đủ 7/8 trạng thái nhưng thiếu mọi chuyển huỷ', () => {
    const cov = coverage([chay(nextStatus, ['search', 'accept', 'arrive', 'pickup', 'start', 'complete'])]);
    expect(cov.allStates).toBe('7/8');
    expect(cov.validTransitions).toBe('6/12');
  });
});

describe('TEST-034 · all transitions coverage: 72/72 ô của state table (domain)', () => {
  it.each(caBang.map((c) => [c.from, c.event, c.to ?? 'INVALID_TRANSITION'] as const))('(%s, %s) ⇒ %s', (from, event, mongDoi) => {
    expect(ketQua(nextStatus, from, event)).toBe(mongDoi);
  });
});

describe('TEST-034 · defect «khách huỷ được khi đã đón» (buggy-trip-state.ts)', () => {
  it('bộ 5 test case (100% valid transitions) vẫn XANH trên bản lỗi', () => {
    const runs = TEST_CASES.map((tc) => chay(buggyNextStatus, tc.events));
    expect(runs.map((r) => r.cuoi)).toEqual(TEST_CASES.map((tc) => tc.ketThuc));
    expect(coverage(runs).validTransitions).toBe('12/12');
  });

  it('thử 60 ô không hợp lệ: bản lỗi lệch đúng một ô', () => {
    const lech = caBang
      .filter((c) => c.to === null && ketQua(buggyNextStatus, c.from, c.event) !== 'INVALID_TRANSITION')
      .map((c) => `(${c.from}, ${c.event})`);
    expect(lech).toEqual(['(picked_up, customer_cancel)']);
  });
});

// ---------------------------------------------------------------- tầng use case: chuyến thật trong kho bộ nhớ

const KHACH = { role: 'customer' as const, id: 'C001' };
const D101 = { role: 'driver' as const, id: 'D101' }; // SG, xe máy
const D102 = { role: 'driver' as const, id: 'D102' }; // SG, xe máy

async function moi(paymentMethod: 'cash' | 'vixanh' = 'cash') {
  const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
  const system = createMemorySystem({ clock });
  const { deps } = system;
  let tripId = '';
  let taiXe = D101;
  let lanNhan = 0;
  const lam = async (e: TripEvent, ai?: typeof D101 | typeof KHACH): Promise<void> => {
    clock.advanceMs(30_000);
    switch (e) {
      case 'search': {
        const q = await createQuote(deps, KHACH, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 });
        tripId = (await createTrip(deps, KHACH, { quoteId: q.id, paymentMethod })).id;
        return;
      }
      case 'accept':
        if (!ai) taiXe = lanNhan++ % 2 === 0 ? D101 : D102; // luân phiên tài xế giữa các lần nhận
        await acceptTrip(deps, (ai as typeof D101) ?? taiXe, tripId, 5);
        return;
      case 'arrive': case 'pickup': case 'start': case 'complete':
        await driverStep(deps, (ai as typeof D101) ?? taiXe, tripId, e);
        return;
      case 'customer_cancel':
        await cancelByCustomer(deps, (ai as typeof KHACH) ?? KHACH, tripId);
        return;
      case 'driver_cancel':
        await cancelByDriver(deps, (ai as typeof D101) ?? taiXe, tripId);
        return;
      case 'system_cancel':
        clock.advanceMinutes(5);
        await expireStaleSearches(deps);
        return;
      default:
        throw new Error(e);
    }
  };
  const trip = async () => (await deps.trips.findById(tripId))!;
  const nhatKy = async () => (await deps.trips.listEvents(tripId)).map((x) => x.status);
  return { deps, system, lam, trip, nhatKy };
}

describe('TEST-034 · 5 test case trên use case thật: trạng thái + nhật ký trip_events (R-STATE-06)', () => {
  it.each(TEST_CASES.map((tc) => [tc.id, tc] as const))('%s', async (_id, tc) => {
    const h = await moi();
    const mongDoi = chay(nextStatus, tc.events).trangThai; // đường đi theo mô hình, gồm `requested`
    for (const e of tc.events) await h.lam(e);
    expect((await h.trip()).status).toBe(tc.ketThuc);
    expect(await h.nhatKy()).toEqual(mongDoi);
  });
});

/** Các cặp (trạng thái, sự kiện) KHÔNG hợp lệ — mỗi test case chỉ thử MỘT cặp (tránh defect masking). */
const KHONG_HOP_LE: { id: string; truoc: TripEvent[]; thu: TripEvent; ai?: typeof D101 | typeof KHACH; tt: TripStatus }[] = [
  { id: 'INV-01', truoc: ['search', 'accept', 'arrive', 'pickup'], thu: 'customer_cancel', tt: 'picked_up' },
  { id: 'INV-02', truoc: ['search', 'accept'], thu: 'complete', tt: 'accepted' },
  { id: 'INV-03', truoc: ['search', 'accept', 'arrive', 'pickup', 'start', 'complete'], thu: 'customer_cancel', tt: 'completed' },
  { id: 'INV-04', truoc: ['search', 'customer_cancel'], thu: 'accept', ai: D102, tt: 'cancelled' },
  { id: 'INV-05', truoc: ['search', 'accept', 'arrive', 'pickup', 'start'], thu: 'driver_cancel', tt: 'in_transit' },
  { id: 'INV-06', truoc: ['search', 'accept', 'arrive', 'pickup', 'start', 'complete'], thu: 'complete', tt: 'completed' },
];

describe('TEST-034 · 6 chuyển không hợp lệ qua use case ⇒ INVALID_TRANSITION, chuyến giữ nguyên (R-STATE-03)', () => {
  it.each(KHONG_HOP_LE.map((c) => [c.id, c.tt, c.thu, c] as const))('%s · (%s, %s)', async (_id, _tt, _thu, c) => {
    const h = await moi('vixanh');
    for (const e of c.truoc) await h.lam(e);
    const truoc = await h.trip();
    const nhatKyTruoc = await h.nhatKy();
    const daThu = h.system.payments.totalCharged();
    expect(truoc.status).toBe(c.tt);

    await expect(h.lam(c.thu, c.ai)).rejects.toMatchObject({ code: 'INVALID_TRANSITION' });

    expect(await h.trip()).toEqual(truoc); // không trường nào đổi
    expect(await h.nhatKy()).toEqual(nhatKyTruoc); // không thêm dòng trip_events
    expect(h.system.payments.totalCharged()).toBe(daThu); // không thu tiền lần hai
  });
});

describe('TEST-034 · ô trống của state table nhưng API trả mã KHÁC (kiểm quyền/idempotency đứng trước)', () => {
  it('(searching, arrive) bởi tài xế chưa được gán ⇒ TRIP_NOT_FOUND (R-API-10), không phải INVALID_TRANSITION', async () => {
    const h = await moi();
    await h.lam('search');
    await expect(h.lam('arrive', D101)).rejects.toMatchObject({ code: 'TRIP_NOT_FOUND' });
    expect((await h.trip()).status).toBe('searching');
  });

  it('(accepted, accept): tài xế khác ⇒ TRIP_ALREADY_TAKEN; chính tài xế đang giữ ⇒ trả lại chuyến, không đổi gì (R-ACCEPT-07)', async () => {
    const h = await moi();
    await h.lam('search');
    await h.lam('accept'); // D101 nhận
    const truoc = await h.trip();
    await expect(h.lam('accept', D102)).rejects.toMatchObject({ code: 'TRIP_ALREADY_TAKEN' });
    await h.lam('accept', D101);
    expect(await h.trip()).toEqual(truoc);
    expect(await h.nhatKy()).toEqual(['requested', 'searching', 'accepted']);
  });
});

it('TEST-034 · mô hình không có hai chuyển trùng cặp (trạng thái, sự kiện) — máy trạng thái tất định', () => {
  expect(VALID.map((t) => `${t.from}-${t.event}`)).toHaveLength(new Set(VALID.map((t) => `${t.from}-${t.event}`)).size);
});
