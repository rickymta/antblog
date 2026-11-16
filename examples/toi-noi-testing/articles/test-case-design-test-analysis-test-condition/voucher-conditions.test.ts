/**
 * TCD-01 — bằng chứng chạy được cho các test condition voucher (tầng use case, kho trong bộ nhớ, đồng hồ giả).
 * Mỗi `it` ghi mã TCOND nó chứng minh; số tiền mong đợi tính tay từ SPEC (công thức ghi trong chú thích).
 * Dữ liệu: voucher mẫu của SPEC Phụ lục A (GIAM20, FOOD15, HN10, XMAS50), khách C001.
 */
import { beforeEach, describe, expect, it } from 'vitest';
import { createQuote, type QuoteInput, type StoredQuote } from '../../src/app/quote';
import { cancelByCustomer, createTrip } from '../../src/app/trips';
import type { AppDeps } from '../../src/app/deps';
import { DomainError } from '../../src/domain/errors';
import { toMilli } from '../../src/domain/money';
import { vnTime } from '../../src/domain/time';
import { voucherDiscountMilli } from '../../src/domain/voucher';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';

const C001 = { role: 'customer' as const, id: 'C001' };
const T0 = vnTime(2026, 11, 4, 9, 0); // thứ Tư 04/11/2026, 09:00 giờ VN — ngoài cao điểm

let clock: FixedClock;
let system: LabSystem;
let deps: AppDeps;

beforeEach(() => {
  clock = new FixedClock(T0);
  system = createMemorySystem({ clock });
  deps = system.deps;
});

/** Đặt xe xe máy 7,4 km ở SG: cước cơ bản = 12.000 + 4.500 × 7,4 = 45.300 đ. */
const ride74 = (voucherCode?: string, extra: Partial<QuoteInput> = {}): QuoteInput =>
  ({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4, voucherCode, ...extra }) as QuoteInput;

/** Trả báo giá, hoặc mã lỗi nghiệp vụ nếu bị từ chối. */
async function quote(input: QuoteInput): Promise<StoredQuote | string> {
  try {
    return await createQuote(deps, C001, input);
  } catch (e) {
    if (e instanceof DomainError) return e.code;
    throw e;
  }
}

async function errorOf(p: Promise<unknown>): Promise<string | null> {
  try {
    await p;
    return null;
  } catch (e) {
    if (e instanceof DomainError) return e.code;
    throw e;
  }
}

const ok = (q: StoredQuote | string): StoredQuote => {
  if (typeof q === 'string') throw new Error(`mong đợi báo giá, nhận lỗi ${q}`);
  return q;
};

describe('TCD-01 · nhận diện mã (R-VOUCHER-02, -03)', () => {
  it('TCOND-VOUCHER-01 · « giam20 » được chuẩn hoá thành GIAM20 và được giảm', async () => {
    const q = ok(await quote(ride74(' giam20 ')));
    expect(q.details.voucherCode).toBe('GIAM20');
    expect(q.details.discount).toBe(9_060); // 20% × 45.300
  });

  it('TCOND-VOUCHER-02 · mã chỉ gồm khoảng trắng = không nhập voucher', async () => {
    const q = ok(await quote(ride74('   ')));
    expect(q.details.voucherCode).toBeNull();
    expect(q.details.discount).toBe(0);
    expect(q.details.total).toBe(46_000); // 45.300 làm tròn lên
  });

  it('TCOND-VOUCHER-03 · mã không tồn tại ⇒ VOUCHER_NOT_FOUND; «GIAM 20» (khoảng trắng giữa) cũng vậy — xem Q-02', async () => {
    expect(await quote(ride74('KHONGCO'))).toBe('VOUCHER_NOT_FOUND');
    expect(await quote(ride74('GIAM 20'))).toBe('VOUCHER_NOT_FOUND');
  });
});

describe('TCD-01 · thời hạn (R-VOUCHER-01, -04, R-GEN-02)', () => {
  it('TCOND-VOUCHER-04 · XMAS50 dùng được cả ngày đầu 24/12/2025 và tới 23:59:59 ngày cuối 26/12/2025', async () => {
    clock.set(vnTime(2025, 12, 24, 9, 0));
    expect(ok(await quote(ride74('XMAS50'))).details.total).toBe(23_000); // 45.300 − 50% (22.650 < trần 50.000) = 22.650 ⇒ 23.000
    clock.set(vnTime(2025, 12, 26, 23, 59, 59));
    expect(typeof (await quote(ride74('XMAS50')))).toBe('object');
  });

  it('TCOND-VOUCHER-05 · XMAS50 lúc 00:00 ngày 27/12/2025 và vào 04/11/2026 ⇒ VOUCHER_EXPIRED', async () => {
    clock.set(vnTime(2025, 12, 27, 0, 0));
    expect(await quote(ride74('XMAS50'))).toBe('VOUCHER_EXPIRED');
    clock.set(T0);
    expect(await quote(ride74('XMAS50'))).toBe('VOUCHER_EXPIRED');
  });

  it('TCOND-VOUCHER-06 · GIAM20 (từ 01/01/2026): 2025-12-31T17:00Z là 00:00 giờ VN ⇒ hợp lệ; 16:59:59Z ⇒ hết hạn', async () => {
    clock.set(new Date('2025-12-31T17:00:00Z'));
    expect(ok(await quote(ride74('GIAM20'))).details.discount).toBe(9_060);
    clock.set(new Date('2025-12-31T16:59:59Z'));
    expect(await quote(ride74('GIAM20'))).toBe('VOUCHER_EXPIRED');
  });
});

describe('TCD-01 · phạm vi (R-VOUCHER-01, -05)', () => {
  it('TCOND-VOUCHER-07 · FOOD15 (chỉ food) cho đặt xe; HN10 (chỉ ride) cho bưu kiện ở HN ⇒ VOUCHER_NOT_APPLICABLE', async () => {
    expect(await quote(ride74('FOOD15'))).toBe('VOUCHER_NOT_APPLICABLE');
    expect(await quote({ kind: 'parcel', cityCode: 'HN', vehicle: 'bike', distanceKm: 7.4, parcelWeightKg: 3, voucherCode: 'HN10' })).toBe('VOUCHER_NOT_APPLICABLE');
  });

  it('TCOND-VOUCHER-08 · HN10 (chỉ HN) ở SG ⇒ VOUCHER_NOT_APPLICABLE', async () => {
    expect(await quote(ride74('HN10'))).toBe('VOUCHER_NOT_APPLICABLE');
  });

  it('TCOND-VOUCHER-09 · GIAM20 (kinds, cityCodes rỗng) áp được cho đồ ăn ở Cần Thơ', async () => {
    const q = ok(await quote({ kind: 'food', cityCode: 'CT', distanceKm: 3, items: [{ name: 'Cơm tấm', unitPrice: 30_000, quantity: 2 }], voucherCode: 'GIAM20' } as QuoteInput));
    expect(q.details.itemsDiscount).toBe(12_000); // 20% × 60.000
    expect(q.details.total).toBe(74_000); // 60.000 − 12.000 + phí giao 26.000
  });
});

describe('TCD-01 · tiền giảm (R-VOUCHER-06, R-FARE-06, R-FARE-07, R-FOOD-05)', () => {
  it('TCOND-VOUCHER-10 · HN10 (tối thiểu 40.000 đ) ở HN: 6,2 km (A = 39.900) bị từ chối; 6,3 km (A = 40.350) được giảm', async () => {
    const hn = (km: number) => ({ kind: 'ride', cityCode: 'HN', vehicle: 'bike', distanceKm: km, voucherCode: 'HN10' }) as QuoteInput;
    expect(await quote(hn(6.2))).toBe('VOUCHER_MIN_AMOUNT');
    const q = ok(await quote(hn(6.3)));
    expect(q.details.discount).toBe(4_035); // 10% × 40.350
    expect(q.details.total).toBe(37_000); // 36.315 ⇒ 37.000
  });

  it('TCOND-VOUCHER-10 · đúng biên A = minAmount (hàm miền): 40.000 đ hợp lệ, giảm 4.000 đ; 39.999 đ bị từ chối', () => {
    const hn10 = { code: 'HN10', percent: 10, maxDiscount: 15_000, minAmount: 40_000 };
    expect(voucherDiscountMilli(hn10, toMilli(40_000))).toBe(toMilli(4_000));
    expect(() => voucherDiscountMilli(hn10, toMilli(39_999))).toThrow(expect.objectContaining({ code: 'VOUCHER_MIN_AMOUNT' }));
  });

  it('TCOND-VOUCHER-11 · GIAM20, 7,4 km, ngoài cao điểm, không mưa: giảm 20% × 45.300 = 9.060 ⇒ 37.000 đ', async () => {
    const q = ok(await quote(ride74('GIAM20')));
    expect(q.details.fare.beforeDiscount).toBe(45_300);
    expect(q.details.discount).toBe(9_060);
    expect(q.details.total).toBe(37_000); // 36.240 ⇒ 37.000
  });

  it('TCOND-VOUCHER-12 · GIAM20 cho ô tô 12 km: 20% × 157.000 = 31.400 > trần ⇒ giảm 30.000 ⇒ 127.000 đ', async () => {
    const q = ok(await quote({ kind: 'ride', cityCode: 'SG', vehicle: 'car', distanceKm: 12, voucherCode: 'GIAM20' }));
    expect(q.details.fare.beforeDiscount).toBe(157_000); // 25.000 + 11.000 × 12
    expect(q.details.discount).toBe(30_000);
    expect(q.details.total).toBe(127_000);
  });

  it('TCOND-VOUCHER-13 · A gồm phụ phí mưa: SG đang mưa ⇒ A = 53.300, giảm 10.660 ⇒ 43.000 đ (không phải 45.000)', async () => {
    system.weather.setRaining(['SG']);
    const q = ok(await quote(ride74('GIAM20')));
    expect(q.details.fare.beforeDiscount).toBe(53_300);
    expect(q.details.discount).toBe(10_660);
    expect(q.details.total).toBe(43_000);
  });

  it('TCOND-VOUCHER-13 · A gồm hệ số cao điểm: ví dụ R-FARE-09 (cao điểm, mưa) ⇒ A = 64.625, giảm 12.925 ⇒ 52.000 đ', async () => {
    clock.set(vnTime(2026, 11, 4, 11, 30));
    system.weather.setRaining(['SG']);
    const q = ok(await quote(ride74('GIAM20')));
    expect(q.details.fare.beforeDiscount).toBe(64_625);
    expect(q.details.discount).toBe(12_925);
    expect(q.details.total).toBe(52_000);
  });

  it('TCOND-VOUCHER-14 · FOOD15: giảm 15% tiền món 60.000 = 9.000, phí giao 26.000 không bị trừ ⇒ 77.000 đ', async () => {
    const q = ok(await quote({ kind: 'food', cityCode: 'SG', distanceKm: 3, items: [{ name: 'Cơm tấm', unitPrice: 30_000, quantity: 2 }], voucherCode: 'FOOD15' } as QuoteInput));
    expect(q.details.itemsDiscount).toBe(9_000);
    expect(q.details.fare.discount).toBe(0); // bảng chi tiết của phí giao không có tiền giảm
    expect(q.details.deliveryFee).toBe(26_000); // 12.000 + 4.500 × 3 = 25.500 ⇒ 26.000
    expect(q.details.total).toBe(77_000);
  });

  it('TCOND-VOUCHER-14 · FOOD15 so minAmount với tiền món: 40.000 + phí giao 26.000 vẫn ⇒ VOUCHER_MIN_AMOUNT', async () => {
    expect(await quote({ kind: 'food', cityCode: 'SG', distanceKm: 3, items: [{ name: 'Phở', unitPrice: 40_000, quantity: 1 }], voucherCode: 'FOOD15' } as QuoteInput)).toBe('VOUCHER_MIN_AMOUNT');
  });

  it('TCOND-VOUCHER-15 · GIAM20 cho xe máy 1 km: 16.500 − 3.300 = 13.200 < 15.000 ⇒ 15.000 đ; fare.discount vẫn ghi 3.300 (Q-01)', async () => {
    const q = ok(await quote({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 1, voucherCode: 'GIAM20' }));
    expect(q.details.fare.afterDiscount).toBe(13_200);
    expect(q.details.total).toBe(15_000);
    expect(q.details.discount).toBe(3_300); // khách thực chất chỉ được giảm 16.500 ⇒ 17.000 so với 15.000
  });
});

describe('TCD-01 · lượt dùng (R-VOUCHER-07, R-QUOTE-03)', () => {
  it('TCOND-VOUCHER-16 · TCOND-VOUCHER-17 · TCOND-VOUCHER-18 · báo giá không tiêu lượt, tạo chuyến tiêu lượt, hết lượt bị chặn ở cả hai lần kiểm, huỷ trả lại lượt', async () => {
    const q1 = ok(await quote(ride74('GIAM20')));
    const q2 = ok(await quote(ride74('GIAM20'))); // báo giá thứ hai vẫn được: báo giá không tiêu lượt
    const trip = await createTrip(deps, C001, { quoteId: q1.id }); // tiêu lượt duy nhất trong ngày
    expect(trip.discount).toBe(9_060);
    expect(await quote(ride74('GIAM20'))).toBe('VOUCHER_USAGE_LIMIT'); // kiểm lần 1: lúc báo giá
    expect(await errorOf(createTrip(deps, C001, { quoteId: q2.id }))).toBe('VOUCHER_USAGE_LIMIT'); // kiểm lần 2: lúc tạo chuyến
    await cancelByCustomer(deps, C001, trip.id); // huỷ khi đang searching
    expect(typeof (await quote(ride74('GIAM20')))).toBe('object'); // lượt đã được trả lại
  });

  it('TCOND-VOUCHER-19 · lượt đếm theo ngày giờ VN: 23:59:59 cùng ngày vẫn hết lượt; 00:00 hôm sau dùng lại được', async () => {
    await createTrip(deps, C001, { quoteId: ok(await quote(ride74('GIAM20'))).id });
    clock.set(vnTime(2026, 11, 4, 23, 59, 59));
    expect(await quote(ride74('GIAM20'))).toBe('VOUCHER_USAGE_LIMIT');
    clock.set(vnTime(2026, 11, 5, 0, 0));
    expect(typeof (await quote(ride74('GIAM20')))).toBe('object');
  });

  it('TCOND-VOUCHER-20 · FOOD15 (2 lần/ngày): chuyến thứ hai được, báo giá thứ ba ⇒ VOUCHER_USAGE_LIMIT', async () => {
    const food = { kind: 'food', cityCode: 'SG', distanceKm: 3, items: [{ name: 'Cơm tấm', unitPrice: 30_000, quantity: 2 }], voucherCode: 'FOOD15' } as QuoteInput;
    await createTrip(deps, C001, { quoteId: ok(await quote(food)).id });
    await createTrip(deps, C001, { quoteId: ok(await quote(food)).id });
    expect(await quote(food)).toBe('VOUCHER_USAGE_LIMIT');
  });

  it('TCOND-VOUCHER-21 · báo giá GIAM20 lúc 23:58 ngày cuối 31/12/2030, đặt lúc 00:01 hôm sau vẫn được; báo giá mới ⇒ hết hạn', async () => {
    clock.set(vnTime(2030, 12, 31, 23, 58));
    const q = ok(await quote(ride74('GIAM20')));
    clock.set(vnTime(2031, 1, 1, 0, 1)); // báo giá còn hạn tới 00:03
    const trip = await createTrip(deps, C001, { quoteId: q.id });
    expect(trip.voucherCode).toBe('GIAM20');
    expect(trip.fare).toBe(37_000);
    expect(await quote(ride74('GIAM20'))).toBe('VOUCHER_EXPIRED');
  });
});

describe('TCD-01 · thứ tự lỗi (R-VOUCHER-08)', () => {
  it('TCOND-VOUCHER-22 · HN10 ở SG với A = 21.000 < 40.000: NOT_APPLICABLE đứng trước MIN_AMOUNT', async () => {
    expect(await quote({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 2, voucherCode: 'HN10' })).toBe('VOUCHER_NOT_APPLICABLE');
  });

  it('TCOND-VOUCHER-22 · HN10 ở SG năm 2031 (vừa hết hạn vừa sai thành phố): EXPIRED đứng trước NOT_APPLICABLE', async () => {
    clock.set(vnTime(2031, 1, 1, 9, 0));
    expect(await quote(ride74('HN10'))).toBe('VOUCHER_EXPIRED');
  });
});
