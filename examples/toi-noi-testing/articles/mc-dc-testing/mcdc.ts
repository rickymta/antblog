/**
 * TEST-047 · Bộ kiểm MC/DC tối giản (không phụ thuộc gì ngoài TypeScript).
 *
 * Mỗi ca test được mô tả bằng một VECTƠ giá trị các điều kiện nguyên tử + kết quả của quyết định.
 * Giá trị `null` = điều kiện KHÔNG được đánh giá ở ca đó (ngắt mạch, hoặc không có dữ liệu để đánh giá);
 * khi so cặp, `null` được coi là «không quan tâm» (cách làm thường gặp khi ngôn ngữ luôn ngắt mạch như JS/TS).
 */
export type CondValue = boolean | null;

export interface TestVector {
  id: string;
  conditions: readonly CondValue[];
  outcome: boolean;
}

export interface IndependencePair {
  /** Chỉ số điều kiện (0-based). */
  condition: number;
  /** Hai ca chứng minh điều kiện đó ảnh hưởng ĐỘC LẬP tới kết quả. */
  pair: [string, string];
}

export interface McdcReport {
  conditionCount: number;
  /** Phần tử i = cặp độc lập của điều kiện i, hoặc null nếu chưa có. */
  pairs: (IndependencePair | null)[];
  /** Số điều kiện đã chứng minh được ảnh hưởng độc lập. */
  shown: number;
  /** MC/DC coverage (%) = shown / conditionCount × 100. */
  percent: number;
  /** Decision coverage: quyết định đã nhận cả TRUE lẫn FALSE. */
  bothOutcomes: boolean;
}

/** Hai vectơ chỉ khác nhau ở điều kiện i (các điều kiện khác bằng nhau hoặc một bên không đánh giá). */
export function differsOnlyAt(x: readonly CondValue[], y: readonly CondValue[], i: number): boolean {
  const xi = x[i];
  const yi = y[i];
  if (xi === null || yi === null || xi === undefined || yi === undefined || xi === yi) return false;
  return x.every((v, j) => j === i || v === null || y[j] === null || v === y[j]);
}

/** Tìm cặp độc lập (unique-cause, cho phép điều kiện không được đánh giá) cho từng điều kiện. */
export function mcdcReport(vectors: readonly TestVector[]): McdcReport {
  const conditionCount = vectors[0]?.conditions.length ?? 0;
  const pairs: (IndependencePair | null)[] = [];
  for (let i = 0; i < conditionCount; i++) {
    let found: IndependencePair | null = null;
    for (let a = 0; a < vectors.length && !found; a++) {
      for (let b = a + 1; b < vectors.length && !found; b++) {
        const va = vectors[a]!;
        const vb = vectors[b]!;
        if (va.outcome !== vb.outcome && differsOnlyAt(va.conditions, vb.conditions, i)) {
          found = { condition: i, pair: [va.id, vb.id] };
        }
      }
    }
    pairs.push(found);
  }
  const shown = pairs.filter((p) => p !== null).length;
  const outcomes = new Set(vectors.map((v) => v.outcome));
  return {
    conditionCount,
    pairs,
    shown,
    percent: conditionCount === 0 ? 100 : Math.round((shown / conditionCount) * 1000) / 10,
    bothOutcomes: outcomes.size === 2,
  };
}
