/**
 * TEST-040 · Miền đầu vào của báo giá bưu kiện: (quãng đường d, khối lượng w, loại xe v).
 * - `expectedOutcome`: oracle viết ĐỘC LẬP với src, từ SPEC R-FARE-02, R-PARCEL-01…03, R-VALID-01
 *   (số học nguyên trên phần trăm/phần mười — đầu vào có tối đa 2 chữ số thập phân).
 * - `domainPoints`: sinh điểm ON/OFF (và IN/OUT) cho từng biên của từng phân vùng — CTAL-TA v4.0 mục 3.1.1.
 */
export type Vehicle = 'bike' | 'car';
export type Code = 'OK' | 'DISTANCE_OUT_OF_RANGE' | 'WEIGHT_OUT_OF_RANGE' | 'VEHICLE_TOO_SMALL';
export interface Outcome { code: Code; surcharge?: number }
export interface Point { d: number; w: number; v: Vehicle; kind: string }

/** Làm tròn half-up tới 0,1 rồi trả về số phần mười (nguyên): 0,45 ⇒ 5; 100,05 ⇒ 1001. */
export function tenths(x: number): number {
  const hundredths = Math.round(x * 100);
  return Math.floor((hundredths + 5) / 10);
}

/** Oracle theo SPEC: thứ tự lỗi R-VALID-01 (quãng đường → khối lượng → loại xe), phụ phí R-PARCEL-03. */
export function expectedOutcome(d: number, w: number, v: Vehicle): Outcome {
  const dt = tenths(d);
  const wt = tenths(w);
  if (dt < 5 || dt > 1000) return { code: 'DISTANCE_OUT_OF_RANGE' };
  if (wt <= 0 || wt > 300) return { code: 'WEIGHT_OUT_OF_RANGE' };
  if (v === 'bike' && wt > 200) return { code: 'VEHICLE_TOO_SMALL' };
  return { code: 'OK', surcharge: wt <= 50 ? 0 : 2_000 * Math.ceil((wt - 50) / 10) };
}

// ------------------------------------------------------------ phân vùng và biên

export type Op = '<' | '<=' | '>' | '>=';
export interface Border { x: 'd' | 'w'; op: Op; c: number }
export interface Partition { id: string; v: Vehicle; borders: Border[]; inPoint: { d: number; w: number } }

const b = (x: 'd' | 'w', op: Op, c: number): Border => ({ x, op, c });

/** Phân vùng trên giá trị ĐÃ làm tròn (độ chính xác 0,1). Các vùng lỗi là phần bù — điểm OFF rơi vào đó. */
export const ROUNDED: Partition[] = [
  { id: 'MIEN_PHI_xe_may', v: 'bike', borders: [b('d', '>=', 0.5), b('d', '<=', 100), b('w', '>', 0), b('w', '<=', 5)], inPoint: { d: 10, w: 3 } },
  { id: 'CO_PHI_xe_may', v: 'bike', borders: [b('d', '>=', 0.5), b('d', '<=', 100), b('w', '>', 5), b('w', '<=', 20)], inPoint: { d: 10, w: 12 } },
  { id: 'XE_NHO_xe_may', v: 'bike', borders: [b('d', '>=', 0.5), b('d', '<=', 100), b('w', '>', 20), b('w', '<=', 30)], inPoint: { d: 10, w: 25 } },
  { id: 'HOP_LE_o_to', v: 'car', borders: [b('d', '>=', 0.5), b('d', '<=', 100), b('w', '>', 0), b('w', '<=', 30)], inPoint: { d: 10, w: 12 } },
];

/**
 * Cùng các phân vùng, nhưng trên giá trị THÔ khách gửi (độ chính xác 0,01). Làm tròn half-up biến
 * biên d' ≥ 0,5 thành d ≥ 0,45 (đóng) và d' ≤ 100 thành d < 100,05 (mở) — suy ra bằng phân tích R-FARE-02.
 */
export const RAW: Partition[] = [
  { id: 'MIEN_PHI_xe_may', v: 'bike', borders: [b('d', '>=', 0.45), b('d', '<', 100.05), b('w', '>=', 0.05), b('w', '<', 5.05)], inPoint: { d: 10, w: 3 } },
  { id: 'CO_PHI_xe_may', v: 'bike', borders: [b('d', '>=', 0.45), b('d', '<', 100.05), b('w', '>=', 5.05), b('w', '<', 20.05)], inPoint: { d: 10, w: 12 } },
  { id: 'XE_NHO_xe_may', v: 'bike', borders: [b('d', '>=', 0.45), b('d', '<', 100.05), b('w', '>=', 20.05), b('w', '<', 30.05)], inPoint: { d: 10, w: 25 } },
  { id: 'HOP_LE_o_to', v: 'car', borders: [b('d', '>=', 0.45), b('d', '<', 100.05), b('w', '>=', 0.05), b('w', '<', 30.05)], inPoint: { d: 10, w: 12 } },
];

const fix = (x: number) => Math.round(x * 100) / 100;

/**
 * ON/OFF theo CTAL-TA 3.1.1 với độ chính xác p:
 * biên đóng (≤, ≥): ON nằm trên biên, OFF ở ngoài sát nhất; biên mở (<, >): ON ở trong sát nhất, OFF nằm trên biên.
 */
export function onOff(border: Border, p: number): { on: number; off: number } {
  const { op, c } = border;
  if (op === '<=') return { on: c, off: fix(c + p) };
  if (op === '>=') return { on: c, off: fix(c - p) };
  if (op === '<') return { on: fix(c - p), off: c };
  return { on: fix(c + p), off: c };
}

/** OUT: ngoài phân vùng, cách biên `distance` (không phải điểm OFF). */
function outValue(border: Border, distance: number): number {
  return border.op === '<' || border.op === '<=' ? fix(border.c + distance) : fix(border.c - distance);
}

/**
 * Sinh coverage item. `reliable = false`: simplified domain coverage (một ON + một OFF mỗi biên).
 * `reliable = true`: reliable domain coverage (thêm một IN và một OUT mỗi biên; IN dùng chung).
 * Điểm trùng (cùng d, w, v) chỉ giữ một — tối ưu mà syllabus gợi ý.
 */
export function domainPoints(parts: Partition[], p: number, reliable: boolean, outDistance = 2): Point[] {
  const points = new Map<string, Point>();
  const add = (d: number, w: number, v: Vehicle, kind: string) => {
    const k = `${d}|${w}|${v}`;
    if (!points.has(k)) points.set(k, { d, w, v, kind });
  };
  for (const part of parts) {
    for (const border of part.borders) {
      const { on, off } = onOff(border, p);
      const at = (value: number) => (border.x === 'd' ? { d: value, w: part.inPoint.w } : { d: part.inPoint.d, w: value });
      const tag = `${part.id} ${border.x}${border.op}${border.c}`;
      add(at(on).d, at(on).w, part.v, `ON  ${tag}`);
      add(at(off).d, at(off).w, part.v, `OFF ${tag}`);
      if (reliable) {
        add(part.inPoint.d, part.inPoint.w, part.v, `IN  ${part.id}`);
        const out = at(outValue(border, outDistance));
        add(out.d, out.w, part.v, `OUT ${tag}`);
      }
    }
  }
  return [...points.values()];
}
