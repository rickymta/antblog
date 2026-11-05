/**
 * Bộ sinh tổ hợp t-wise tối giản cho bài TEST-037 (Pairwise Testing).
 *
 * Cách làm: liệt kê MỌI hàng hợp lệ của mô hình (tích Descartes, lọc qua ràng buộc), rồi chọn tham lam
 * từng hàng phủ được nhiều bộ-t (t-tuple) chưa phủ nhất; hoà thì lấy hàng đứng trước (tất định).
 * Đủ cho mô hình vài nghìn hàng như trong bài. Công cụ thật (PICT, ACTS…) dựng bộ test mà không cần
 * liệt kê toàn bộ không gian — xem mục 6 của bài.
 */

export interface Parameter {
  name: string;
  values: readonly string[];
}
export type Model = readonly Parameter[];
/** Một hàng = chỉ số giá trị của từng tham số, theo thứ tự tham số trong mô hình. */
export type Row = readonly number[];
/** Trả `true` nếu hàng hợp lệ (thoả mọi ràng buộc). */
export type Constraint = (row: Row) => boolean;

const always: Constraint = () => true;

/** Mọi tổ hợp đầy đủ thoả ràng buộc. */
export function allRows(model: Model, allowed: Constraint = always): number[][] {
  let rows: number[][] = [[]];
  for (const p of model) {
    rows = rows.flatMap((r) => p.values.map((_, v) => [...r, v]));
  }
  return rows.filter((r) => allowed(r));
}

/** Mọi cách chọn t chỉ số tham số trong n tham số (thứ tự tăng dần). */
export function paramCombos(n: number, t: number): number[][] {
  const out: number[][] = [];
  const pick = (start: number, acc: number[]) => {
    if (acc.length === t) {
      out.push(acc);
      return;
    }
    for (let i = start; i < n; i++) pick(i + 1, [...acc, i]);
  };
  pick(0, []);
  return out;
}

/** Mã hoá mọi bộ-t của một hàng thành số nguyên (mỗi bộ-t một khoá duy nhất). */
function tupleKeys(row: Row, combos: number[][], base: number): number[] {
  return combos.map((combo, ci) => combo.reduce((key, p) => key * base + (row[p] as number), ci));
}

function baseOf(model: Model): number {
  return Math.max(...model.map((p) => p.values.length));
}

/** Bộ test phủ mọi bộ-t KHẢ THI (bộ-t xuất hiện trong ít nhất một hàng hợp lệ). */
export function generate(model: Model, t: number, allowed: Constraint = always): number[][] {
  const rows = allRows(model, allowed);
  const combos = paramCombos(model.length, t);
  const base = baseOf(model);
  const keys = rows.map((r) => tupleKeys(r, combos, base));
  const feasible = new Set(keys.flat());
  const covered = new Set<number>();
  const chosen = new Set<number>();
  const suite: number[][] = [];

  while (covered.size < feasible.size) {
    let best = -1;
    let bestGain = 0;
    keys.forEach((k, i) => {
      if (chosen.has(i)) return;
      const gain = k.filter((key) => !covered.has(key)).length;
      if (gain > bestGain) {
        best = i;
        bestGain = gain;
      }
    });
    chosen.add(best);
    suite.push(rows[best] as number[]);
    for (const key of keys[best] as number[]) covered.add(key);
  }
  return suite;
}

/** Độ phủ t-wise của một bộ test: số bộ-t khả thi đã phủ / tổng số bộ-t khả thi. */
export function coverage(model: Model, suite: readonly Row[], t: number, allowed: Constraint = always) {
  const combos = paramCombos(model.length, t);
  const base = baseOf(model);
  const feasible = new Set(allRows(model, allowed).flatMap((r) => tupleKeys(r, combos, base)));
  const covered = new Set(suite.flatMap((r) => tupleKeys(r, combos, base)).filter((k) => feasible.has(k)));
  return { covered: covered.size, feasible: feasible.size, percent: (100 * covered.size) / feasible.size };
}

/** Đọc một hàng thành bản ghi { tên tham số: giá trị }. */
export function label(model: Model, row: Row): Record<string, string> {
  return Object.fromEntries(model.map((p, i) => [p.name, p.values[row[i] as number] as string]));
}

/** PRNG tất định (mulberry32) — để so sánh với bộ chọn ngẫu nhiên mà kết quả lặp lại được. */
export function rng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let x = a;
    x = Math.imul(x ^ (x >>> 15), x | 1);
    x ^= x + Math.imul(x ^ (x >>> 7), x | 61);
    return ((x ^ (x >>> 14)) >>> 0) / 4294967296;
  };
}

/** Chọn ngẫu nhiên `size` hàng hợp lệ KHÁC NHAU. */
export function randomSuite(model: Model, size: number, seed: number, allowed: Constraint = always): number[][] {
  const pool = allRows(model, allowed);
  const next = rng(seed);
  for (let i = pool.length - 1; i > 0; i--) {
    const j = Math.floor(next() * (i + 1));
    [pool[i], pool[j]] = [pool[j] as number[], pool[i] as number[]];
  }
  return pool.slice(0, size);
}
