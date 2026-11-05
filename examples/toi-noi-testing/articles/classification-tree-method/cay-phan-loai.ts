/**
 * TEST-036 · Classification Tree Method — cây phân loại cho POST /api/quotes (SPEC R-API-07).
 * Mô hình là DỮ LIỆU: classification, class, quan hệ cha–con, ràng buộc. Các hàm thuần ở đây
 * sinh tổ hợp, đếm, kiểm each-choice coverage và dựng bộ base choice; test ở cay-phan-loai.test.ts.
 */

export type Kind = 'ride' | 'parcel' | 'food';
export type Weight = 'nhe' | 'vua' | 'nang';
export type Vehicle = 'bike' | 'car';
export type City = 'SG' | 'HN' | 'VT';
export type VoucherClass = 'khong' | 'hop-le' | 'sai-pham-vi';

/** Một tổ hợp = chọn đúng một class ở mỗi classification đang «mở» (weight chỉ mở khi kind = parcel). */
export interface Combo {
  kind: Kind;
  weight: Weight | null;
  vehicle: Vehicle;
  city: City;
  rain: boolean;
  peak: boolean;
  voucher: VoucherClass;
}

type Key = keyof Combo;

interface Classification {
  key: Key;
  name: string;
  classes: readonly (string | boolean)[];
  /** Classification con chỉ có nghĩa dưới một class của cha. */
  parent?: { key: Key; value: string };
}

export const TREE: readonly Classification[] = [
  { key: 'kind', name: 'Dịch vụ', classes: ['ride', 'parcel', 'food'] },
  { key: 'weight', name: 'Khối lượng', classes: ['nhe', 'vua', 'nang'], parent: { key: 'kind', value: 'parcel' } },
  { key: 'vehicle', name: 'Loại xe', classes: ['bike', 'car'] },
  { key: 'city', name: 'Thành phố', classes: ['SG', 'HN', 'VT'] },
  { key: 'rain', name: 'Mưa', classes: [true, false] },
  { key: 'peak', name: 'Cao điểm', classes: [true, false] },
  { key: 'voucher', name: 'Voucher', classes: ['khong', 'hop-le', 'sai-pham-vi'] },
];

/** Tất cả tổ hợp theo cây (tích Đề-các có tôn trọng quan hệ cha–con). */
export function allCombos(): Combo[] {
  const out: Combo[] = [];
  const kindWeights: [Kind, Weight | null][] = [['ride', null], ['parcel', 'nhe'], ['parcel', 'vua'], ['parcel', 'nang'], ['food', null]];
  for (const [kind, weight] of kindWeights)
    for (const vehicle of ['bike', 'car'] as const)
      for (const city of ['SG', 'HN', 'VT'] as const)
        for (const rain of [true, false])
          for (const peak of [true, false])
            for (const voucher of ['khong', 'hop-le', 'sai-pham-vi'] as const)
              out.push({ kind, weight, vehicle, city, rain, peak, voucher });
  return out;
}

/** Ràng buộc LOẠI TRỪ: tổ hợp không dựng được — dịch vụ thời tiết không có trạng thái cho thành phố không hỗ trợ. */
export function isInfeasible(c: Combo): boolean {
  return c.city === 'VT' && c.rain;
}

export type Outcome = 'OK' | 'VALIDATION_ERROR' | 'CITY_NOT_SUPPORTED' | 'VEHICLE_TOO_SMALL' | 'VOUCHER_NOT_APPLICABLE';

/**
 * Oracle mức «loại kết quả», theo thứ tự kiểm của R-VALID-01. Hai ràng buộc QUY TẮC (dựng được nhưng hệ thống
 * phải từ chối): đồ ăn chỉ xe máy (R-FOOD-06), > 20 kg phải ô tô (R-PARCEL-02).
 */
export function expectedOutcome(c: Combo): Outcome {
  if (c.kind === 'food' && c.vehicle === 'car') return 'VALIDATION_ERROR';
  if (c.city === 'VT') return 'CITY_NOT_SUPPORTED';
  if (c.weight === 'nang' && c.vehicle === 'bike') return 'VEHICLE_TOO_SMALL';
  if (c.voucher === 'sai-pham-vi') return 'VOUCHER_NOT_APPLICABLE';
  return 'OK';
}

const label = (key: Key, value: unknown) => `${key}=${String(value)}`;

/** Each-choice coverage: class nào chưa xuất hiện trong bất kỳ tổ hợp nào. */
export function uncoveredClasses(rows: readonly Combo[]): string[] {
  return TREE.flatMap((cl) => cl.classes.filter((v) => !rows.some((r) => r[cl.key] === v)).map((v) => label(cl.key, v)));
}

/**
 * Bản chặt hơn: một class chỉ tính là đã phủ ở dòng mà nó THỰC SỰ được xử lý — dòng trả OK, hoặc chính class đó
 * gây ra lỗi. Class nằm trong dòng bị lỗi khác chặn trước thì bị che (masking).
 */
export function uncoveredEffective(rows: readonly Combo[]): string[] {
  const cause = (r: Combo): string | null =>
    expectedOutcome(r) === 'CITY_NOT_SUPPORTED' ? label('city', r.city) : expectedOutcome(r) === 'VOUCHER_NOT_APPLICABLE' ? label('voucher', r.voucher) : null;
  const effective = (r: Combo, key: Key, v: unknown) => expectedOutcome(r) === 'OK' || cause(r) === label(key, v);
  return TREE.flatMap((cl) => cl.classes.filter((v) => !rows.some((r) => r[cl.key] === v && effective(r, cl.key, v))).map((v) => label(cl.key, v)));
}

/** Base choice (CTAL-TA 3.1.2): dòng gốc + mỗi class không phải gốc thay vào đúng một lần. */
export function baseChoice(base: Combo, baseWeight: Weight = 'nhe'): Combo[] {
  const rows: Combo[] = [base];
  for (const cl of TREE) {
    for (const v of cl.classes) {
      if (cl.key === 'weight') {
        if (v === baseWeight) continue;
        rows.push({ ...base, kind: 'parcel', weight: v as Weight });
      } else if (cl.key === 'kind') {
        if (v === base.kind) continue;
        rows.push({ ...base, kind: v as Kind, weight: v === 'parcel' ? baseWeight : null });
      } else if (v !== base[cl.key]) {
        rows.push({ ...base, [cl.key]: v });
      }
    }
  }
  return rows;
}

const TIME = { peak: { hour: 11, minute: 30 }, normal: { hour: 9, minute: 0 } };
const DISTANCE: Record<Kind, number> = { ride: 7.4, parcel: 10, food: 3 };
const WEIGHT_KG: Record<Weight, number> = { nhe: 3, vua: 12, nang: 25 };

/** Biến tổ hợp thành test data cụ thể: thân yêu cầu + giờ (VN) + danh sách thành phố đang mưa. */
export function toRequest(c: Combo) {
  const body: Record<string, unknown> = { kind: c.kind, cityCode: c.city, vehicle: c.vehicle, distanceKm: DISTANCE[c.kind] };
  if (c.weight) body.parcelWeightKg = WEIGHT_KG[c.weight];
  if (c.kind === 'food') body.items = [{ name: 'Cơm tấm', unitPrice: 60_000, quantity: 1 }];
  if (c.voucher === 'hop-le') body.voucherCode = 'GIAM20';
  if (c.voucher === 'sai-pham-vi') body.voucherCode = c.kind === 'ride' && c.city === 'HN' ? 'FOOD15' : 'HN10';
  return { body, time: c.peak ? TIME.peak : TIME.normal, raining: c.rain && c.city !== 'VT' ? [c.city] : [] };
}

export function moTa(c: Combo): string {
  const w = c.weight ? `/${c.weight}` : '';
  return `${c.kind}${w} ${c.vehicle} ${c.city} ${c.rain ? 'mưa' : 'khô'} ${c.peak ? 'cao-điểm' : 'thường'} voucher:${c.voucher}`;
}
