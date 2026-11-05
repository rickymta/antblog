/**
 * TEST-020 · Impact analysis cho bản thay đổi fare-2026.2 (tăng giá km xe máy, ngừng Nha Trang).
 *
 * Mỗi ca là một test đang có trong bộ regression. Người phân tích DỰ ĐOÁN trước (cột `doi`):
 * kết quả mong đợi của ca có đổi sau thay đổi hay không. Test kiểm ba điều:
 *   1. bản hiện hành (src) cho đúng kết quả cũ — tính tay từ SPEC v1.1;
 *   2. bản thay đổi (pricing-2026-2.ts) cho đúng kết quả mới — tính tay từ quy tắc mới;
 *   3. dự đoán «đổi / giữ» khớp với thực tế.
 * Ca «đổi» ⇒ phải cập nhật kết quả mong đợi trong bộ regression; ca «giữ» ⇒ regression thuần.
 */
import { describe, expect, it } from 'vitest';
import { calculateFare, type FareInput } from '../../src/domain/fare';
import { itemsSubtotal, priceFood } from '../../src/domain/food';
import { toDong, toMilli } from '../../src/domain/money';
import { normalizeWeightKg, parcelSurcharge } from '../../src/domain/parcel';
import { normalizePriceRequest, PRICING_VERSION } from '../../src/domain/pricing';
import { voucherDiscountMilli, type VoucherTerms } from '../../src/domain/voucher';
import { customerCancellationFee } from '../../src/domain/cancel';
import { assertCitySupported20262, calculateFare20262, PRICING_VERSION_2026_2 } from './pricing-2026-2';

type Version = 'cu' | 'moi';

const GIAM20: VoucherTerms = { code: 'GIAM20', percent: 20, maxDiscount: 30_000, minAmount: 0 };
const FOOD15: VoucherTerms = { code: 'FOOD15', percent: 15, maxDiscount: 25_000, minAmount: 50_000 };
const HN10: VoucherTerms = { code: 'HN10', percent: 10, maxDiscount: 15_000, minAmount: 40_000 };

function outcome(fn: () => unknown): unknown {
  try {
    return fn();
  } catch (error) {
    return (error as { code?: string }).code ?? String(error);
  }
}

const fare = (v: Version, input: FareInput) => (v === 'cu' ? calculateFare(input) : calculateFare20262(input)).total;
const off = { peak: false, raining: false };

/** Đơn đồ ăn: [tiền giảm, tổng]. Phí giao dùng công thức xe máy của từng phiên bản (R-FOOD-04). */
function food(v: Version, distanceKm: number, subtotal: number, voucher: VoucherTerms | null): [number, number] {
  const items = [{ name: 'Món', unitPrice: subtotal, quantity: 1 }];
  if (v === 'cu') {
    const p = priceFood({ distanceKm, items, ...off, voucher });
    return [p.itemsDiscount, p.total];
  }
  const discount = voucher ? toDong(voucherDiscountMilli(voucher, toMilli(itemsSubtotal(items)))) : 0;
  return [discount, subtotal - discount + calculateFare20262({ vehicle: 'bike', distanceKm, ...off }).total];
}

function city(v: Version, cityCode: string): unknown {
  return outcome(() => {
    if (v === 'cu') normalizePriceRequest({ kind: 'ride', cityCode, vehicle: 'bike', distanceKm: 5 });
    else assertCitySupported20262(cityCode);
    return 'OK';
  });
}

interface Case {
  id: string;
  basis: string;
  title: string;
  doi: boolean; // dự đoán của người phân tích
  cu: unknown; // mong đợi theo SPEC v1.1
  moi: unknown; // mong đợi theo quy tắc fare-2026.2
  run: (v: Version) => unknown;
}

const CASES: Case[] = [
  { id: 'M-01', basis: 'R-FARE-01', title: 'xe máy 7,4 km, 09:00, không mưa', doi: true, cu: 46_000, moi: 49_000,
    run: (v) => fare(v, { vehicle: 'bike', distanceKm: 7.4, ...off }) },
  { id: 'M-02', basis: 'R-FARE-09', title: 'oracle 7,4 km cao điểm, mưa, GIAM20', doi: true, cu: 52_000, moi: 56_000,
    run: (v) => fare(v, { vehicle: 'bike', distanceKm: 7.4, peak: true, raining: true, voucher: GIAM20 }) },
  { id: 'M-03', basis: 'R-FARE-01', title: 'ô tô 7,4 km', doi: false, cu: 107_000, moi: 107_000,
    run: (v) => fare(v, { vehicle: 'car', distanceKm: 7.4, ...off }) },
  { id: 'M-04', basis: 'R-FARE-07', title: 'xe máy 0,5 km — cước tối thiểu', doi: false, cu: 15_000, moi: 15_000,
    run: (v) => fare(v, { vehicle: 'bike', distanceKm: 0.5, ...off }) },
  { id: 'M-05', basis: 'R-FARE-08', title: 'xe máy 1,0 km — làm tròn lên nghìn', doi: false, cu: 17_000, moi: 17_000,
    run: (v) => fare(v, { vehicle: 'bike', distanceKm: 1.0, ...off }) },
  { id: 'M-06', basis: 'R-PARCEL-03', title: 'bưu kiện xe máy 3 km, 6,1 kg', doi: true, cu: 30_000, moi: 31_000,
    run: (v) => fare(v, { vehicle: 'bike', distanceKm: 3, parcelSurcharge: parcelSurcharge(normalizeWeightKg(6.1)), ...off }) },
  { id: 'M-07', basis: 'R-PARCEL-02', title: 'bưu kiện ô tô 3 km, 21 kg', doi: false, cu: 90_000, moi: 90_000,
    run: (v) => fare(v, { vehicle: 'car', distanceKm: 3, parcelSurcharge: parcelSurcharge(normalizeWeightKg(21)), ...off }) },
  { id: 'M-08', basis: 'R-FOOD-04', title: 'đồ ăn 2 km, món 50.000 đ — [giảm, tổng]', doi: true, cu: [0, 71_000], moi: [0, 72_000],
    run: (v) => food(v, 2, 50_000, null) },
  { id: 'M-09', basis: 'R-FOOD-05', title: 'đồ ăn 2 km, món 100.000 đ, FOOD15 — [giảm, tổng]', doi: true, cu: [15_000, 106_000], moi: [15_000, 107_000],
    run: (v) => food(v, 2, 100_000, FOOD15) },
  { id: 'M-10', basis: 'R-VOUCHER-06', title: 'HN10, xe máy 6 km (A so với tối thiểu 40.000 đ)', doi: true, cu: 'VOUCHER_MIN_AMOUNT', moi: 38_000,
    run: (v) => outcome(() => fare(v, { vehicle: 'bike', distanceKm: 6, ...off, voucher: HN10 })) },
  { id: 'M-11', basis: 'R-VOUCHER-06', title: 'GIAM20, xe máy 30 km (trần 30.000 đ)', doi: true, cu: 118_000, moi: 132_000,
    run: (v) => fare(v, { vehicle: 'bike', distanceKm: 30, ...off, voucher: GIAM20 }) },
  { id: 'M-12', basis: 'R-GEN-03', title: 'báo giá ở Nha Trang (NT)', doi: true, cu: 'OK', moi: 'CITY_NOT_SUPPORTED',
    run: (v) => city(v, 'NT') },
  { id: 'M-13', basis: 'R-GEN-03', title: 'báo giá ở Đà Nẵng (DN)', doi: false, cu: 'OK', moi: 'OK',
    run: (v) => city(v, 'DN') },
  { id: 'M-14', basis: 'R-CANCEL-03', title: 'phí huỷ xe máy ở arriving', doi: false, cu: 10_000, moi: 10_000,
    run: () => customerCancellationFee({ status: 'arriving', vehicle: 'bike', acceptedAt: new Date(0), etaMinutes: 10, now: new Date(5 * 60_000) }).fee },
  { id: 'M-15', basis: 'R-QUOTE-02', title: 'pricingVersion lưu vào báo giá', doi: true, cu: 'fare-2026.1', moi: 'fare-2026.2',
    run: (v) => (v === 'cu' ? PRICING_VERSION : PRICING_VERSION_2026_2) },
];

describe('TEST-020 · impact analysis fare-2026.2', () => {
  it.each(CASES)('$id ($basis) $title — dự đoán đổi: $doi', (c) => {
    const cu = c.run('cu');
    const moi = c.run('moi');
    expect(cu, 'bản hiện hành khớp SPEC v1.1').toEqual(c.cu);
    expect(moi, 'bản thay đổi khớp quy tắc mới').toEqual(c.moi);
    expect(JSON.stringify(cu) !== JSON.stringify(moi), 'dự đoán đổi/giữ').toBe(c.doi);
  });

  it('tổng kết: 9 ca phải cập nhật kết quả mong đợi, 6 ca là regression thuần', () => {
    const changed = CASES.filter((c) => JSON.stringify(c.run('cu')) !== JSON.stringify(c.run('moi'))).map((c) => c.id);
    expect(changed).toEqual(['M-01', 'M-02', 'M-06', 'M-08', 'M-09', 'M-10', 'M-11', 'M-12', 'M-15']);
  });
});
