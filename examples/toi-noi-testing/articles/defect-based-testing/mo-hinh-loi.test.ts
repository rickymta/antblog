/**
 * TEST-054 — test thiết kế từ BA cụm defect hay tái diễn (phan-loai.test.ts): lam-tron, gio-vn, trang-thai.
 * Mỗi cụm có một «mô hình lỗi» (fault model) ⇒ một bộ kiểm tra dạng hàm `check…(impl)` trả về danh sách ca sai.
 *  - Chạy với mã thật của lab (src/) ⇒ phải rỗng.
 *  - Chạy với bản tái tạo defect lịch sử (ban-loi.ts) ⇒ phải bắt được — bằng chứng bộ test nhắm đúng loại lỗi.
 * Chạy: npx vitest run articles/defect-based-testing/mo-hinh-loi.test.ts
 */
import { describe, expect, it } from 'vitest';
import { calculateFare, normalizeDistanceKm, type FareInput } from '../../src/domain/fare';
import { isPeakTime, vnTime } from '../../src/domain/time';
import { TRANSITIONS, TRIP_EVENTS, TRIP_STATUSES, type TripEvent, type TripStatus } from '../../src/domain/trip-state';
import { assertVoucherEligible } from '../../src/domain/voucher';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, cancelByCustomer, cancelByDriver, createTrip } from '../../src/app/trips';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { voucher } from '../../src/infra/seed';
import * as bug from './ban-loi';

const report = (name: string, failures: string[]) =>
  console.log(`${name}: ${failures.length} ca sai${failures.length ? ` — vd ${failures.slice(0, 2).join(' | ')}` : ''}`);

// =============================================================== FM-R · làm tròn tiền
// Mô hình lỗi: (R1) làm tròn đầu vào 0,1 sai ở giá trị x,x5; (R2) làm tròn giữa chừng; (R3) làm tròn cuối sai chiều.
// Oracle ĐỘC LẬP với src: số học BigInt theo đơn vị 1/4000 đồng (×1,25 = 5/4, percent/100, km/10 đều chia hết).

type Terms = { percent: number; maxDiscount: number; minAmount: number } | null;
const TABLE = { bike: [12_000n, 4_500n, 15_000n], car: [25_000n, 11_000n, 30_000n] } as const;

function oracleTotal(vehicle: 'bike' | 'car', tenths: number, peak: boolean, rain: boolean, v: Terms): number | 'MIN' {
  const [open, perKm, minimum] = TABLE[vehicle];
  const U = 4000n;
  const base = open * U + perKm * BigInt(tenths) * 400n;
  const afterPeak = peak ? (base * 5n) / 4n : base;
  const before = afterPeak + (rain ? 8_000n * U : 0n);
  if (v && before < BigInt(v.minAmount) * U) return 'MIN';
  const pct = v ? (before * BigInt(v.percent)) / 100n : 0n;
  const discount = v && pct > BigInt(v.maxDiscount) * U ? BigInt(v.maxDiscount) * U : pct;
  const after = before - discount > minimum * U ? before - discount : minimum * U;
  const thousand = 1000n * U;
  return Number(((after + thousand - 1n) / thousand) * 1000n);
}

interface RoundingImpl {
  normalizeDistance(raw: number): number;
  fareTotal(input: FareInput): number;
}

function checkRounding(impl: RoundingImpl): string[] {
  const failures: string[] = [];
  // R1: mọi giá trị x,x5 trong miền hợp lệ phải làm tròn LÊN (half-up) — 0,45 … 99,95
  for (let k = 4; k < 1000; k++) {
    const raw = Number(`${Math.floor(k / 10)}.${k % 10}5`);
    const got = impl.normalizeDistance(raw);
    if (got !== (k + 1) / 10) failures.push(`R1 ${raw} km ⇒ ${got}`);
  }
  // R2 + R3: toàn bộ 996 quãng đường × 2 loại xe × cao điểm × mưa × 4 voucher so với oracle
  const vouchers: Terms[] = [null, { percent: 20, maxDiscount: 30_000, minAmount: 0 }, { percent: 15, maxDiscount: 25_000, minAmount: 0 }, { percent: 10, maxDiscount: 15_000, minAmount: 40_000 }];
  for (let tenths = 5; tenths <= 1000; tenths++) {
    for (const vehicle of ['bike', 'car'] as const) {
      for (const peak of [false, true]) {
        for (const rain of [false, true]) {
          for (const v of vouchers) {
            const expected = oracleTotal(vehicle, tenths, peak, rain, v);
            let got: number | 'MIN';
            try {
              got = impl.fareTotal({ vehicle, distanceKm: tenths / 10, peak, raining: rain, voucher: v && { code: 'T', ...v } });
            } catch (error) {
              got = (error as { code?: string }).code === 'VOUCHER_MIN_AMOUNT' ? 'MIN' : NaN;
            }
            if (got !== expected) failures.push(`R2 ${vehicle} ${tenths / 10} km${peak ? ' cao điểm' : ''}${rain ? ' mưa' : ''}${v ? ` −${v.percent}%` : ''}: ${got} ≠ ${expected}`);
          }
        }
      }
    }
  }
  return failures;
}

const realRounding: RoundingImpl = { normalizeDistance: normalizeDistanceKm, fareTotal: (i) => calculateFare(i).total };

describe('FM-R · làm tròn tiền (D-01, D-04, D-06, D-09, D-13, D-17, D-22)', () => {
  it('mã thật của lab: 996 giá trị x,x5 và 31.872 tổ hợp cước khớp oracle tuyệt đối', () => {
    const failures = checkRounding(realRounding);
    report('src', failures);
    expect(failures).toEqual([]);
  });
  it('bắt được D-01 tái diễn (toFixed)', () => {
    const failures = checkRounding({ ...realRounding, normalizeDistance: bug.normalizeDistanceToFixed });
    report('D-01 toFixed', failures);
    expect(failures.length).toBeGreaterThan(0);
  });
  it('bắt được D-04 tái diễn (làm tròn gần nhất)', () => {
    const failures = checkRounding({ ...realRounding, fareTotal: bug.fareTotalRoundNearest });
    report('D-04 làm tròn gần nhất', failures);
    expect(failures.length).toBeGreaterThan(0);
  });
  it('bắt được D-06/D-09 tái diễn (làm tròn tiền giảm)', () => {
    const failures = checkRounding({ ...realRounding, fareTotal: bug.fareTotalRoundDiscount });
    report('D-06/D-09 làm tròn tiền giảm', failures);
    expect(failures.length).toBeGreaterThan(0);
  });
  it('biến thể làm tròn về đồng sau ×1,25 là TƯƠNG ĐƯƠNG: không ca nào khác oracle', () => {
    const failures = checkRounding({ ...realRounding, fareTotal: bug.fareTotalRoundAfterPeak });
    report('làm tròn về đồng sau ×1,25', failures);
    expect(failures).toEqual([]);
  });
});

// =============================================================== FM-T · giờ Việt Nam
// Mô hình lỗi: (T1) lấy giờ/ngày theo múi giờ máy thay vì UTC+7; (T2) sai đầu mút khoảng giờ [a, b);
// (T3) ranh giới ngày VN = 17:00:00 UTC. Ca kiểm: mọi mốc giờ của quy tắc ±1 giây, ở nhiều ngày khác nhau.

interface TimeImpl {
  isPeak(at: Date): boolean;
  voucherValid(at: Date, validFrom: string, validTo: string): boolean;
}

function checkVnTime(impl: TimeImpl): string[] {
  const failures: string[] = [];
  const days = [[2026, 1, 1], [2026, 6, 30], [2026, 11, 4], [2026, 12, 31]] as const;
  const edges: [number, number, number, boolean][] = [
    [10, 59, 59, false], [11, 0, 0, true], [12, 59, 59, true], [13, 0, 0, false],
    [16, 59, 59, false], [17, 0, 0, true], [18, 59, 59, true], [19, 0, 0, false],
    [0, 0, 0, false], [6, 59, 59, false], [7, 0, 0, false], [23, 59, 59, false],
  ];
  for (const [y, m, d] of days) {
    for (const [h, mi, s, peak] of edges) {
      const at = vnTime(y, m, d, h, mi, s);
      if (impl.isPeak(at) !== peak) failures.push(`T-peak ${d}/${m} ${h}:${String(mi).padStart(2, '0')}:${String(s).padStart(2, '0')} VN ⇒ ${!peak}`);
    }
    // voucher có hạn đúng một ngày (validFrom = validTo = ngày d): còn hạn suốt 00:00:00 → 23:59:59 giờ VN của ngày đó
    const day = `${y}-${String(m).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
    const inside = [vnTime(y, m, d, 0, 0, 0), vnTime(y, m, d, 6, 59, 59), vnTime(y, m, d, 7, 0, 0), vnTime(y, m, d, 23, 59, 59)];
    for (const at of inside) if (!impl.voucherValid(at, day, day)) failures.push(`T-day ${at.toISOString()} trong ngày VN ${day} bị coi là hết hạn`);
    const outside = [new Date(inside[0]!.getTime() - 1000), new Date(inside[3]!.getTime() + 1000)];
    for (const at of outside) if (impl.voucherValid(at, day, day)) failures.push(`T-day ${at.toISOString()} ngoài ngày VN ${day} vẫn còn hạn`);
  }
  return failures;
}

const realTime: TimeImpl = {
  isPeak: isPeakTime,
  voucherValid(at, validFrom, validTo) {
    try {
      assertVoucherEligible(voucher({ code: 'T', validFrom, validTo }), { kind: 'ride', cityCode: 'SG', at });
      return true;
    } catch {
      return false;
    }
  },
};

describe('FM-T · giờ Việt Nam (D-02, D-05, D-11, D-15, D-24, D-29)', () => {
  it('mã thật của lab: 48 mốc cao điểm và 24 mốc ngày voucher đều đúng', () => {
    const failures = checkVnTime(realTime);
    report('src', failures);
    expect(failures).toEqual([]);
  });
  it.each([
    ['D-02 giờ máy chủ UTC', { ...realTime, isPeak: bug.isPeakServerUtc }],
    ['D-24 đầu mút <=', { ...realTime, isPeak: bug.isPeakEndInclusive }],
    ['D-05 ngày theo toISOString', { ...realTime, voucherValid: bug.voucherValidIsoDate }],
  ])('bắt được %s tái diễn', (name, impl) => {
    const failures = checkVnTime(impl);
    report(name, failures);
    expect(failures.length).toBeGreaterThan(0);
  });
});

// =============================================================== FM-S · trạng thái chuyến
// Mô hình lỗi: (S1) chuyển không hợp lệ được chấp nhận; (S2) chuyển hợp lệ tới sai trạng thái; (S3) trạng thái kết thúc
// bị «sống lại». Ca kiểm: TOÀN BỘ 8 × 9 = 72 cặp (trạng thái, sự kiện) so với bảng R-STATE-02 chép tay từ SPEC.
// Ngoài mô hình: lỗi đồng thời (D-03, D-14, D-28) cần kiểm thử đồng thời — Series 08.

const SPEC_R_STATE_02: Record<string, TripStatus> = {
  'requested/search': 'searching',
  'searching/accept': 'accepted', 'searching/customer_cancel': 'cancelled', 'searching/system_cancel': 'cancelled',
  'accepted/arrive': 'arriving', 'accepted/customer_cancel': 'cancelled', 'accepted/driver_cancel': 'searching',
  'arriving/pickup': 'picked_up', 'arriving/customer_cancel': 'cancelled', 'arriving/driver_cancel': 'searching',
  'picked_up/start': 'in_transit', 'in_transit/complete': 'completed',
};

type NextImpl = (from: TripStatus, event: TripEvent) => TripStatus | 'INVALID';

function checkStates(next: NextImpl): string[] {
  const failures: string[] = [];
  for (const from of TRIP_STATUSES) {
    for (const event of TRIP_EVENTS) {
      const expected = SPEC_R_STATE_02[`${from}/${event}`] ?? 'INVALID';
      const got = next(from, event);
      if (got !== expected) failures.push(`${from} + ${event} ⇒ ${got} (đặc tả: ${expected})`);
    }
  }
  return failures;
}

const realNext: NextImpl = (from, event) => TRANSITIONS[from][event] ?? 'INVALID';

describe('FM-S · trạng thái chuyến (D-08, D-18; D-10, D-23 ở tầng use case)', () => {
  it('mã thật của lab: 72/72 cặp (trạng thái, sự kiện) đúng đặc tả', () => {
    const failures = checkStates(realNext);
    report('src', failures);
    expect(failures).toEqual([]);
  });
  it.each([
    ['D-08 huỷ sau khi đón', bug.nextCustomerCancelTooLate],
    ['D-18 huỷ nhận sau hoàn thành', bug.nextDriverCancelAfterComplete],
  ])('bắt được %s tái diễn', (name, next) => {
    const failures = checkStates(next);
    report(name, failures);
    expect(failures.length).toBeGreaterThan(0);
  });
});

// S4 (dọn dữ liệu khi rời trạng thái — D-10, D-23): mỗi lối RA khỏi accepted/arriving phải xoá dấu vết của lần nhận
// và trả tài xế về online. Kiểm ở tầng use case với kho bộ nhớ; tiêu chí «tài xế online» = nhận được chuyến kế tiếp.
describe('FM-S · dọn dữ liệu khi rời trạng thái (D-10, D-23)', () => {
  const customer = { role: 'customer', id: 'C001' } as const;
  const driver = { role: 'driver', id: 'D101' } as const;
  async function newTrip(system: ReturnType<typeof createMemorySystem>) {
    const quote = await createQuote(system.deps, customer, { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 });
    return createTrip(system.deps, customer, { quoteId: quote.id });
  }

  it.each(['driver_cancel', 'customer_cancel'] as const)('%s từ accepted: không còn acceptedAt/ETA cũ, tài xế nhận được chuyến kế tiếp', async (exit) => {
    const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
    const system = createMemorySystem({ clock });
    const trip = await newTrip(system);
    await acceptTrip(system.deps, driver, trip.id, 5);
    clock.advanceMinutes(3);
    const after = exit === 'driver_cancel' ? await cancelByDriver(system.deps, driver, trip.id) : (await cancelByCustomer(system.deps, customer, trip.id)).trip;
    const leftovers = exit === 'driver_cancel' ? [after.driverId, after.acceptedAt, after.etaMinutes] : [];
    expect(leftovers.every((v) => v === null)).toBe(true);
    const next = await newTrip(system);
    expect((await acceptTrip(system.deps, driver, next.id, 5)).status).toBe('accepted');
  });
});
