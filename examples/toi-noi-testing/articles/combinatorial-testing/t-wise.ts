/**
 * Bộ sinh t-wise có ràng buộc cho bài TEST-038 (Combinatorial Testing).
 *
 * Giống bản của TEST-037: liệt kê mọi hàng hợp lệ (tích Descartes lọc qua ràng buộc), rồi chọn tham lam
 * hàng phủ nhiều bộ-t KHẢ THI chưa phủ nhất; hoà thì lấy hàng đứng trước (tất định).
 * Bản này mã hoá bộ-t thành chỉ số mảng để chạy được t = 4 trên vài nghìn hàng.
 * "Khả thi" = xuất hiện trong ít nhất một hàng hợp lệ ⇒ ràng buộc tự loại bộ-t không thể có khỏi mẫu số.
 */

export interface Parameter {
  name: string;
  values: readonly string[];
}
export type Model = readonly Parameter[];
export type Row = readonly number[];
export type Constraint = (row: Row) => boolean;

const always: Constraint = () => true;

export function allRows(model: Model, allowed: Constraint = always): number[][] {
  let rows: number[][] = [[]];
  for (const p of model) rows = rows.flatMap((r) => p.values.map((_, v) => [...r, v]));
  return rows.filter((r) => allowed(r));
}

export function paramCombos(n: number, t: number): number[][] {
  const out: number[][] = [];
  const pick = (start: number, acc: number[]) => {
    if (acc.length === t) return void out.push(acc);
    for (let i = start; i < n; i++) pick(i + 1, [...acc, i]);
  };
  pick(0, []);
  return out;
}

function encoder(model: Model, t: number) {
  const combos = paramCombos(model.length, t);
  const base = Math.max(...model.map((p) => p.values.length));
  const span = base ** t;
  const keysOf = (row: Row): Int32Array =>
    Int32Array.from(combos, (combo, ci) => ci * span + combo.reduce((k, p) => k * base + (row[p] as number), 0));
  return { size: combos.length * span, keysOf };
}

export function generate(model: Model, t: number, allowed: Constraint = always): number[][] {
  const rows = allRows(model, allowed);
  const { size, keysOf } = encoder(model, t);
  const keys = rows.map(keysOf);
  const need = new Uint8Array(size); // 1 = bộ-t khả thi chưa phủ
  let remaining = 0;
  for (const k of keys) for (const key of k) if (need[key] === 0) (need[key] = 1), remaining++;

  const chosen = new Uint8Array(rows.length);
  const suite: number[][] = [];
  while (remaining > 0) {
    let best = -1;
    let bestGain = 0;
    for (let i = 0; i < rows.length; i++) {
      if (chosen[i]) continue;
      let gain = 0;
      for (const key of keys[i] as Int32Array) gain += need[key] as number;
      if (gain > bestGain) (best = i), (bestGain = gain);
    }
    chosen[best] = 1;
    suite.push(rows[best] as number[]);
    for (const key of keys[best] as Int32Array) if (need[key] === 1) (need[key] = 0), remaining--;
  }
  return suite;
}

export function coverage(model: Model, suite: readonly Row[], t: number, allowed: Constraint = always) {
  const { keysOf } = encoder(model, t);
  const feasible = new Set(allRows(model, allowed).flatMap((r) => [...keysOf(r)]));
  const covered = new Set(suite.flatMap((r) => [...keysOf(r)]).filter((k) => feasible.has(k)));
  return { covered: covered.size, feasible: feasible.size, percent: (100 * covered.size) / feasible.size };
}

export function label(model: Model, row: Row): Record<string, string> {
  return Object.fromEntries(model.map((p, i) => [p.name, p.values[row[i] as number] as string]));
}

/** PRNG tất định mulberry32. */
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

/** `size` hàng hợp lệ khác nhau chọn ngẫu nhiên (xáo Fisher–Yates rồi cắt). */
export function randomSuite(pool: readonly Row[], size: number, seed: number): Row[] {
  const copy = [...pool];
  const next = rng(seed);
  for (let i = copy.length - 1; i > 0; i--) {
    const j = Math.floor(next() * (i + 1));
    [copy[i], copy[j]] = [copy[j] as Row, copy[i] as Row];
  }
  return copy.slice(0, size);
}
