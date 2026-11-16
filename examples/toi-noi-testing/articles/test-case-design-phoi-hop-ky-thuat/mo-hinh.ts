/**
 * TCD-06 · Phối hợp kỹ thuật cho kiểm tra yêu cầu báo giá (POST /api/quotes, dịch vụ ride và parcel).
 *
 * Một tệp, bốn phần:
 *  1. Mô hình của từng kỹ thuật: phân vùng EP, giá trị biên BVA (2-value, giá trị thô độ chính xác 0,01),
 *     cột bảng quyết định (thứ tự kiểm R-VALID-01 cho bưu kiện).
 *  2. Oracle suy từ SPEC (R-VALID-01, R-GEN-03, R-FARE-01/02/07/08, R-PARCEL-01/02/03) — số học nguyên trên chuỗi thập phân,
 *     KHÔNG gọi hàm của src/.
 *  3. Phân loại: một test case thật sự phủ những coverage item nào (ca âm chỉ phủ đúng lỗi nó gây ra).
 *  4. Sinh bộ ca từ mô hình và rút gọn tham lam giữ nguyên độ phủ.
 * Điều kiện chung: 09:00 giờ VN (ngoài cao điểm), không mưa, không voucher.
 */

export type Dec = { dec: string }; // số JSON hợp lệ, viết bằng chuỗi thập phân để oracle tính chính xác
export type Bad = { bad: unknown }; // không phải số JSON (chuỗi, null…)
export type NumField = Dec | Bad;
export const d = (dec: string): Dec => ({ dec });
export const bad = (v: unknown): Bad => ({ bad: v });
const isDec = (f: NumField | undefined): f is Dec => !!f && 'dec' in f;

export type Source = 'EP' | 'BVA' | 'DT' | 'GHEP' | 'MASK';

export interface QuoteCase {
  id: string;
  source: Source;
  note: string;
  kind: string;
  cityCode: unknown;
  vehicle: unknown;
  distanceKm: NumField;
  parcelWeightKg?: NumField;
}

export function toBody(c: QuoteCase): Record<string, unknown> {
  const num = (f: NumField) => (isDec(f) ? Number(f.dec) : f.bad);
  const body: Record<string, unknown> = { kind: c.kind, cityCode: c.cityCode, vehicle: c.vehicle, distanceKm: num(c.distanceKm) };
  if (c.parcelWeightKg !== undefined) body.parcelWeightKg = num(c.parcelWeightKg);
  return body;
}

// ------------------------------------------------------------------ 1. mô hình kỹ thuật

export const EP_PARTITIONS = {
  K1: 'kind = ride', K2: 'kind = parcel', K3: 'kind khác (400)',
  C1: 'một trong 6 mã R-GEN-03', C2: 'chuỗi 1–10 ký tự, không được phục vụ (422)', C3: 'sai định dạng (400)',
  V1: 'bike', V2: 'car', V3: 'giá trị khác (400)',
  D1: 'sau làm tròn < 0,5 (422)', D2: 'sau làm tròn trong [0,5; 100]', D3: 'sau làm tròn > 100 (422)', D4: 'không phải số (400)',
  W1: 'sau làm tròn ≤ 0 (422)', W2: '(0; 20]', W3: '(20; 30]', W4: '> 30 (422)', W5: 'không phải số hoặc thiếu (400)',
  S1: 'đầu ra: phụ phí khối lượng 0 đ (w ≤ 5)', S2: 'đầu ra: phụ phí khối lượng > 0 (w > 5)',
} as const;
export type EpId = keyof typeof EP_PARTITIONS;

/** 2-value BVA trên giá trị THÔ client gửi (độ chính xác 0,01). */
export const BVA_ITEMS = {
  'QD-0,44': { field: 'distanceKm', dec: '0.44' }, 'QD-0,45': { field: 'distanceKm', dec: '0.45' },
  'QD-100,04': { field: 'distanceKm', dec: '100.04' }, 'QD-100,05': { field: 'distanceKm', dec: '100.05' },
  'KL-0,04': { field: 'parcelWeightKg', dec: '0.04' }, 'KL-0,05': { field: 'parcelWeightKg', dec: '0.05' },
  'KL-30,04': { field: 'parcelWeightKg', dec: '30.04' }, 'KL-30,05': { field: 'parcelWeightKg', dec: '30.05' },
  'XM-20,04': { field: 'parcelWeightKg', dec: '20.04', bikeOnly: true }, 'XM-20,05': { field: 'parcelWeightKg', dec: '20.05', bikeOnly: true },
  'PP-5,04': { field: 'parcelWeightKg', dec: '5.04' }, 'PP-5,05': { field: 'parcelWeightKg', dec: '5.05' },
} as const;
export type BvaId = keyof typeof BVA_ITEMS;

/** Bảng quyết định rút gọn cho báo giá bưu kiện — mỗi cột là «lỗi đầu tiên» theo R-VALID-01. */
export const DT_COLUMNS = {
  R1: 'định dạng sai ⇒ 400 VALIDATION_ERROR',
  R2: 'định dạng đúng, thành phố không phục vụ ⇒ CITY_NOT_SUPPORTED',
  R3: '… thành phố đúng, quãng đường ngoài khoảng ⇒ DISTANCE_OUT_OF_RANGE',
  R4: '… quãng đường đúng, khối lượng ngoài khoảng ⇒ WEIGHT_OUT_OF_RANGE',
  R5: '… khối lượng đúng, xe máy chở > 20 kg ⇒ VEHICLE_TOO_SMALL',
  R6: 'mọi điều kiện thoả ⇒ 200',
} as const;
export type DtId = keyof typeof DT_COLUMNS;

export const ALL_ITEMS: string[] = [
  ...Object.keys(EP_PARTITIONS).map((k) => `EP:${k}`),
  ...Object.keys(BVA_ITEMS).map((k) => `BVA:${k}`),
  ...Object.keys(DT_COLUMNS).map((k) => `DT:${k}`),
];

// ------------------------------------------------------------------ 2. oracle từ SPEC

/** Chuỗi thập phân ⇒ số phần mười, làm tròn half-up (R-FARE-02, R-PARCEL-01). */
export function tenthsHalfUp(dec: string): number {
  const neg = dec.startsWith('-');
  const [ip, fp = ''] = (neg ? dec.slice(1) : dec).split('.');
  const digits = Number(ip + fp);
  let tenths: number;
  if (fp.length <= 1) tenths = digits * 10 ** (1 - fp.length);
  else {
    const scale = 10 ** (fp.length - 1);
    tenths = Math.floor(digits / scale) + ((digits % scale) * 2 >= scale ? 1 : 0);
  }
  return neg ? -tenths : tenths;
}

const CITIES = ['SG', 'HN', 'DN', 'HP', 'CT', 'NT'];
const PRICE = { bike: { open: 12_000, perKm: 4_500, min: 15_000 }, car: { open: 25_000, perKm: 11_000, min: 30_000 } } as const;

export type Outcome =
  | { status: 200; total: number }
  | { status: 400; code: 'VALIDATION_ERROR'; fields: string[] }
  | { status: 422; code: string };

export function expectedOutcome(c: QuoteCase): Outcome {
  // định dạng (400) — kind sai thì không biết dùng lược đồ nào nên chỉ báo kind
  if (c.kind !== 'ride' && c.kind !== 'parcel') return { status: 400, code: 'VALIDATION_ERROR', fields: ['kind'] };
  const fields: string[] = [];
  if (typeof c.cityCode !== 'string' || c.cityCode.length < 1 || c.cityCode.length > 10) fields.push('cityCode');
  if (c.vehicle !== 'bike' && c.vehicle !== 'car') fields.push('vehicle');
  if (!isDec(c.distanceKm)) fields.push('distanceKm');
  if (c.kind === 'parcel' && !isDec(c.parcelWeightKg)) fields.push('parcelWeightKg');
  if (fields.length) return { status: 400, code: 'VALIDATION_ERROR', fields };
  // quy tắc nghiệp vụ theo thứ tự R-VALID-01
  if (!CITIES.includes(c.cityCode as string)) return { status: 422, code: 'CITY_NOT_SUPPORTED' };
  const dT = tenthsHalfUp((c.distanceKm as Dec).dec);
  if (dT < 5 || dT > 1000) return { status: 422, code: 'DISTANCE_OUT_OF_RANGE' };
  let surcharge = 0;
  if (c.kind === 'parcel') {
    const wT = tenthsHalfUp((c.parcelWeightKg as Dec).dec);
    if (wT <= 0 || wT > 300) return { status: 422, code: 'WEIGHT_OUT_OF_RANGE' };
    if (wT > 200 && c.vehicle === 'bike') return { status: 422, code: 'VEHICLE_TOO_SMALL' };
    if (wT > 50) surcharge = 2_000 * Math.ceil((wT - 50) / 10);
  }
  const p = PRICE[c.vehicle as 'bike' | 'car'];
  const fare = p.open + (p.perKm * dT) / 10 + surcharge; // luôn là số nguyên đồng
  return { status: 200, total: Math.ceil(Math.max(fare, p.min) / 1_000) * 1_000 };
}

// ------------------------------------------------------------------ 3. phân loại coverage item

function bvaHits(c: QuoteCase, only?: 'distanceKm' | 'parcelWeightKg'): string[] {
  return (Object.entries(BVA_ITEMS) as [BvaId, (typeof BVA_ITEMS)[BvaId]][])
    .filter(([, it]) => (only ? it.field === only : true))
    .filter(([, it]) => {
      const f = it.field === 'distanceKm' ? c.distanceKm : c.parcelWeightKg;
      if (it.field === 'parcelWeightKg' && c.kind !== 'parcel') return false;
      if ('bikeOnly' in it && c.vehicle !== 'bike') return false;
      return isDec(f) && f.dec === it.dec;
    })
    .map(([k]) => `BVA:${k}`);
}

/**
 * Các coverage item một ca THẬT SỰ phủ.
 * Ca dương phủ mọi phân vùng hợp lệ, giá trị biên và cột bảng của nó.
 * Ca âm chỉ phủ đúng lỗi được báo (R-VALID-01 dừng ở lỗi đầu): các giá trị sai đứng sau bị che.
 */
export function coverageItems(c: QuoteCase): string[] {
  const o = expectedOutcome(c);
  const parcel = c.kind === 'parcel';
  const dt = (r: DtId) => (parcel ? [`DT:${r}`] : []);
  if (o.status === 200) {
    const wT = parcel ? tenthsHalfUp((c.parcelWeightKg as Dec).dec) : 0;
    return [
      parcel ? 'EP:K2' : 'EP:K1', 'EP:C1', c.vehicle === 'bike' ? 'EP:V1' : 'EP:V2', 'EP:D2',
      ...(parcel ? [wT <= 200 ? 'EP:W2' : 'EP:W3', wT <= 50 ? 'EP:S1' : 'EP:S2'] : []),
      ...bvaHits(c), ...dt('R6'),
    ];
  }
  if (o.status === 400) {
    const map: Record<string, string> = { kind: 'EP:K3', cityCode: 'EP:C3', vehicle: 'EP:V3', distanceKm: 'EP:D4', parcelWeightKg: 'EP:W5' };
    return [...o.fields.map((f) => map[f]!), ...dt('R1')];
  }
  switch (o.code) {
    case 'CITY_NOT_SUPPORTED':
      return ['EP:C2', ...dt('R2')];
    case 'DISTANCE_OUT_OF_RANGE':
      return [tenthsHalfUp((c.distanceKm as Dec).dec) < 5 ? 'EP:D1' : 'EP:D3', ...bvaHits(c, 'distanceKm'), ...dt('R3')];
    case 'WEIGHT_OUT_OF_RANGE':
      return [tenthsHalfUp((c.parcelWeightKg as Dec).dec) <= 0 ? 'EP:W1' : 'EP:W4', ...bvaHits(c, 'parcelWeightKg').filter((x) => x.includes('KL-')), ...dt('R4')];
    case 'VEHICLE_TOO_SMALL':
      return [...bvaHits(c, 'parcelWeightKg').filter((x) => x.includes('XM-')), ...dt('R5')];
  }
  return [];
}

export interface Coverage { ep: [number, number]; bva: [number, number]; dt: [number, number]; all: [number, number] }

export function coverage(suite: QuoteCase[]): Coverage {
  const hit = new Set(suite.flatMap(coverageItems));
  const count = (prefix: string) => {
    const items = ALL_ITEMS.filter((i) => i.startsWith(prefix));
    return [items.filter((i) => hit.has(i)).length, items.length] as [number, number];
  };
  return { ep: count('EP:'), bva: count('BVA:'), dt: count('DT:'), all: [ALL_ITEMS.filter((i) => hit.has(i)).length, ALL_ITEMS.length] };
}

// ------------------------------------------------------------------ 4. sinh và rút gọn

const RIDE = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: d('7.4') } as const;
const PARCEL = { kind: 'parcel', cityCode: 'SG', vehicle: 'car', distanceKm: d('7.4'), parcelWeightKg: d('3') } as const;

/** EP: Each Choice — 3 ca dương ghép phân vùng hợp lệ + 10 ca âm, mỗi ca đúng một giá trị sai (đại diện ở giữa). */
export const EP_SUITE: QuoteCase[] = [
  { id: 'EP-01', source: 'EP', note: 'ride, bike, 7,4 km', ...RIDE },
  { id: 'EP-02', source: 'EP', note: 'parcel, car, 25 kg', ...PARCEL, parcelWeightKg: d('25') },
  { id: 'EP-03', source: 'EP', note: 'parcel, bike, 3 kg', ...PARCEL, vehicle: 'bike' },
  { id: 'EP-04', source: 'EP', note: 'kind = taxi', ...RIDE, kind: 'taxi' },
  { id: 'EP-05', source: 'EP', note: 'thành phố DL', ...RIDE, cityCode: 'DL' },
  { id: 'EP-06', source: 'EP', note: 'thành phố rỗng', ...RIDE, cityCode: '' },
  { id: 'EP-07', source: 'EP', note: 'vehicle = truck', ...RIDE, vehicle: 'truck' },
  { id: 'EP-08', source: 'EP', note: '0,2 km', ...RIDE, distanceKm: d('0.2') },
  { id: 'EP-09', source: 'EP', note: '150 km', ...RIDE, distanceKm: d('150') },
  { id: 'EP-10', source: 'EP', note: 'quãng đường "7.4" (chuỗi)', ...RIDE, distanceKm: bad('7.4') },
  { id: 'EP-11', source: 'EP', note: '−1 kg', ...PARCEL, parcelWeightKg: d('-1') },
  { id: 'EP-12', source: 'EP', note: '40 kg', ...PARCEL, parcelWeightKg: d('40') },
  { id: 'EP-13', source: 'EP', note: 'khối lượng "5" (chuỗi)', ...PARCEL, parcelWeightKg: bad('5') },
];

/** BVA 2-value trên giá trị thô: mỗi coverage item một ca, các tham số khác ở giá trị hợp lệ. */
export const BVA_SUITE: QuoteCase[] = (Object.entries(BVA_ITEMS) as [BvaId, (typeof BVA_ITEMS)[BvaId]][]).map(([k, it], i) => ({
  id: `BVA-${String(i + 1).padStart(2, '0')}`,
  source: 'BVA' as const,
  note: k,
  ...(it.field === 'distanceKm'
    ? { ...RIDE, distanceKm: d(it.dec) }
    : { ...PARCEL, vehicle: 'bikeOnly' in it ? 'bike' : 'car', parcelWeightKg: d(it.dec) }),
}));

/** Bảng quyết định: mỗi cột một ca bưu kiện; «–» luôn chọn giá trị HỢP LỆ để ca âm là single fault. */
export const DT_SUITE: QuoteCase[] = [
  { id: 'DT-R1', source: 'DT', note: 'thành phố rỗng', ...PARCEL, cityCode: '' },
  { id: 'DT-R2', source: 'DT', note: 'thành phố DL', ...PARCEL, cityCode: 'DL' },
  { id: 'DT-R3', source: 'DT', note: '150 km', ...PARCEL, distanceKm: d('150') },
  { id: 'DT-R4', source: 'DT', note: '40 kg', ...PARCEL, parcelWeightKg: d('40') },
  { id: 'DT-R5', source: 'DT', note: 'xe máy, 25 kg', ...PARCEL, vehicle: 'bike', parcelWeightKg: d('25') },
  { id: 'DT-R6', source: 'DT', note: 'ô tô, 3 kg', ...PARCEL },
];

/** Kho ca dương ghép: tích Đề-các của mọi giá trị HỢP LỆ mà các kỹ thuật đã chọn, bỏ tổ hợp bị từ chối. */
export const MERGED_POSITIVES: QuoteCase[] = (() => {
  const out: QuoteCase[] = [];
  const dists = ['7.4', '0.45', '100.04'];
  const weights = ['3', '25', '0.05', '30.04', '20.04', '5.04', '5.05'];
  for (const vehicle of ['bike', 'car'])
    for (const dist of dists) {
      out.push({ id: '', source: 'GHEP', note: `ride, ${vehicle}, ${dist} km`, ...RIDE, vehicle, distanceKm: d(dist) });
      for (const w of weights)
        out.push({ id: '', source: 'GHEP', note: `parcel, ${vehicle}, ${dist} km, ${w} kg`, ...PARCEL, vehicle, distanceKm: d(dist), parcelWeightKg: d(w) });
    }
  return out
    .filter((c) => expectedOutcome(c).status === 200)
    .map((c, i) => ({ ...c, id: `GH-${String(i + 1).padStart(2, '0')}` }));
})();

export const TECHNIQUE_SUITE = [...EP_SUITE, ...BVA_SUITE, ...DT_SUITE];
export const POOL = [...TECHNIQUE_SUITE, ...MERGED_POSITIVES];

/** Rút gọn tham lam: mỗi bước chọn ca phủ nhiều item chưa phủ nhất (hoà thì ca đứng trước), rồi bỏ ca thừa từ cuối lên. */
export function greedyMinimize(pool: QuoteCase[]): QuoteCase[] {
  const items = new Map(pool.map((c) => [c.id, new Set(coverageItems(c))]));
  const uncovered = new Set(pool.flatMap((c) => [...items.get(c.id)!]));
  const chosen: QuoteCase[] = [];
  while (uncovered.size > 0) {
    let best: QuoteCase | null = null;
    let bestGain = 0;
    for (const c of pool) {
      if (chosen.includes(c)) continue;
      const gain = [...items.get(c.id)!].filter((i) => uncovered.has(i)).length;
      if (gain > bestGain) { best = c; bestGain = gain; }
    }
    chosen.push(best!);
    items.get(best!.id)!.forEach((i) => uncovered.delete(i));
  }
  // bỏ ca thừa: ca nào mà các ca còn lại đã phủ hết item của nó
  for (let i = chosen.length - 1; i >= 0; i--) {
    const rest = chosen.filter((_, j) => j !== i);
    const restItems = new Set(rest.flatMap((c) => [...items.get(c.id)!]));
    if ([...items.get(chosen[i]!.id)!].every((x) => restItems.has(x))) chosen.splice(i, 1);
  }
  return chosen;
}

/** Ca âm gộp nhiều lỗi — cố ý viết sai cách để minh hoạ defect masking. */
export const MASKED_CASE: QuoteCase = {
  id: 'MASK-01', source: 'MASK', note: 'bưu kiện, thành phố DL + 0,44 km + xe máy 25 kg',
  kind: 'parcel', cityCode: 'DL', vehicle: 'bike', distanceKm: d('0.44'), parcelWeightKg: d('25'),
};
