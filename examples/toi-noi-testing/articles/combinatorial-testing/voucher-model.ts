/**
 * Mô hình tổ hợp của điều kiện voucher (TEST-038, SPEC R-VOUCHER-01…08) và cách biến một hàng
 * thành một test case cụ thể: voucher, yêu cầu báo giá, thời điểm, lịch sử lượt dùng, kết quả mong đợi.
 */
import type { QuoteInput } from '../../src/app/quote';
import { vnTime } from '../../src/domain/time';
import type { TripKind } from '../../src/domain/types';
import type { Voucher } from '../../src/domain/voucher';
import { label, type Constraint, type Model, type Row } from './t-wise';

/** Mô hình HỢP LỆ: mọi giá trị đều dẫn tới báo giá thành công — giá trị lỗi để riêng (NEGATIVE_CASES). */
export const VOUCHER_MODEL: Model = [
  { name: 'dichVu', values: ['ride', 'parcel', 'food'] },
  { name: 'kinds', values: ['rỗng', 'gồm dịch vụ'] },
  { name: 'cityCodes', values: ['rỗng', 'gồm SG'] },
  { name: 'ngay', values: ['đầu hạn', 'giữa hạn', 'cuối hạn'] },
  { name: 'gio', values: ['06:00', '10:00', '20:00'] },
  { name: 'minAmount', values: ['0', 'dương'] },
  { name: 'aSoVoiMin', values: ['bằng', 'lớn hơn'] },
  { name: 'tran', values: ['dưới trần', 'chạm trần'] },
  { name: 'luotNgay', values: ['1', '2'] },
  { name: 'lichSu', values: ['chưa dùng', '1 lượt tối qua', '1 lượt hôm nay'] },
  { name: 'maNhap', values: ['COMBO20', '  combo20 '] },
];

/**
 * Ba ràng buộc:
 * C1 minAmount = 0 ⇒ A không thể "bằng" min (A luôn > 0).
 * C2 luotNgay = 1 ⇒ không có "1 lượt hôm nay" (tổ hợp đó là lỗi VOUCHER_USAGE_LIMIT — thuộc test âm).
 * C3 "1 lượt tối qua" ⇒ không phải "đầu hạn" (hôm qua voucher chưa có hiệu lực, không thể đã dùng).
 */
export const VOUCHER_CONSTRAINTS: Constraint = (r) =>
  !(r[5] === 0 && r[6] === 0) && !(r[8] === 0 && r[9] === 2) && !(r[9] === 1 && r[3] === 0);

/**
 * Mô hình TRỘN (để thấy hiện tượng che lỗi): thêm giá trị không hợp lệ vào ba tham số.
 * Thứ tự giá trị hợp lệ giữ nguyên, giá trị lỗi thêm vào cuối.
 */
export const MIXED_MODEL: Model = VOUCHER_MODEL.map((p) =>
  p.name === 'kinds' || p.name === 'cityCodes'
    ? { ...p, values: [...p.values, 'không gồm'] }
    : p.name === 'ngay'
      ? { ...p, values: [...p.values, 'trước hạn', 'sau hạn'] }
      : p,
);

export type Outcome = { ok: true; discount: number } | { ok: false; code: string };

export interface Scenario {
  id: string;
  input: QuoteInput;
  at: Date;
  voucher: Voucher;
  /** Thời điểm các chuyến CHƯA huỷ đã dùng mã (R-VOUCHER-07). */
  uses: Date[];
  expected: Outcome;
}

const CITY = 'SG' as const;
const DAY: Record<string, number> = { 'trước hạn': 0, 'đầu hạn': 1, 'giữa hạn': 15, 'cuối hạn': 30, 'sau hạn': 31 };
const HOUR: Record<string, number> = { '06:00': 6, '10:00': 10, '20:00': 20 };
/** Số tiền áp voucher A (R-VOUCHER-06): xe máy 2 km giờ thường không mưa = 21.000 đ; đồ ăn 1 món 50.000 đ. */
const AMOUNT: Record<TripKind, number> = { ride: 21_000, parcel: 21_000, food: 50_000 };

function inputFor(kind: TripKind, code: string): QuoteInput {
  if (kind === 'food') {
    return { kind, cityCode: CITY, distanceKm: 2, items: [{ name: 'Cơm tấm', unitPrice: 50_000, quantity: 1 }], voucherCode: code };
  }
  if (kind === 'parcel') return { kind, cityCode: CITY, vehicle: 'bike', distanceKm: 2, parcelWeightKg: 3, voucherCode: code };
  return { kind, cityCode: CITY, vehicle: 'bike', distanceKm: 2, voucherCode: code };
}

const OTHER_KIND: Record<TripKind, TripKind> = { ride: 'parcel', parcel: 'food', food: 'ride' };

/** Biến một hàng (của VOUCHER_MODEL hoặc MIXED_MODEL) thành test case kèm kết quả mong đợi. */
export function toScenario(model: Model, row: Row, id: string): Scenario {
  const v = label(model, row);
  const kind = v.dichVu as TripKind;
  const a = AMOUNT[kind];
  // vnTime chấp nhận ngày 0 / 31 của tháng 11 ⇒ 31/10 / 01/12 (Date.UTC tự chuyển tháng).
  const at = vnTime(2026, 11, DAY[v.ngay as string] as number, HOUR[v.gio as string] as number);
  const minAmount = v.minAmount === '0' ? 0 : v.aSoVoiMin === 'bằng' ? a : a - 1_000;
  const maxDiscount = v.tran === 'chạm trần' ? 3_000 : 100_000;
  const voucher: Voucher = {
    code: 'COMBO20',
    percent: 20,
    maxDiscount,
    minAmount,
    kinds: v.kinds === 'rỗng' ? [] : v.kinds === 'gồm dịch vụ' ? [kind] : [OTHER_KIND[kind]],
    cityCodes: v.cityCodes === 'rỗng' ? [] : v.cityCodes === 'gồm SG' ? ['SG', 'HN'] : ['HN'],
    validFrom: '2026-11-01',
    validTo: '2026-11-30',
    perCustomerPerDay: Number(v.luotNgay),
  };
  const uses =
    v.lichSu === '1 lượt tối qua'
      ? [vnTime(2026, 11, (DAY[v.ngay as string] as number) - 1, 18)]
      : v.lichSu === '1 lượt hôm nay'
        ? [new Date(at.getTime() - 30 * 60_000)]
        : [];
  return { id, input: inputFor(kind, v.maNhap as string), at, voucher, uses, expected: oracle(v, a, maxDiscount) };
}

/** Oracle độc lập với mã nguồn: theo thứ tự R-VOUCHER-08, rồi R-VOUCHER-06 cho số tiền giảm. */
function oracle(v: Record<string, string>, a: number, maxDiscount: number): Outcome {
  if (v.ngay === 'trước hạn' || v.ngay === 'sau hạn') return { ok: false, code: 'VOUCHER_EXPIRED' };
  if (v.kinds === 'không gồm' || v.cityCodes === 'không gồm') return { ok: false, code: 'VOUCHER_NOT_APPLICABLE' };
  return { ok: true, discount: Math.min((a * 20) / 100, maxDiscount) };
}

/** Hàng gốc (base choice) — giá trị "điển hình nhất" của mỗi tham số. */
export const BASE_ROW: Row = [0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0];

/** Test âm: mỗi giá trị lỗi một lần, mọi tham số khác giữ giá trị gốc (R-VOUCHER-03…07). */
export function negativeCases(): Scenario[] {
  const base = toScenario(VOUCHER_MODEL, BASE_ROW, 'base');
  const make = (id: string, code: string, patch: Partial<Scenario> & { voucherPatch?: Partial<Voucher> }): Scenario => ({
    ...base,
    ...patch,
    id,
    voucher: { ...base.voucher, ...patch.voucherPatch },
    expected: { ok: false, code },
  });
  return [
    make('N1', 'VOUCHER_NOT_FOUND', { input: { ...base.input, voucherCode: 'KHONGCO' } }),
    make('N2', 'VOUCHER_EXPIRED', { at: vnTime(2026, 10, 31, 10) }),
    make('N3', 'VOUCHER_EXPIRED', { at: vnTime(2026, 12, 1, 10) }),
    make('N4', 'VOUCHER_NOT_APPLICABLE', { voucherPatch: { kinds: ['parcel'] } }),
    make('N5', 'VOUCHER_NOT_APPLICABLE', { voucherPatch: { cityCodes: ['HN'] } }),
    make('N6', 'VOUCHER_MIN_AMOUNT', { voucherPatch: { minAmount: 21_001 } }),
    make('N7', 'VOUCHER_USAGE_LIMIT', { uses: [vnTime(2026, 11, 15, 9, 30)] }),
  ];
}
