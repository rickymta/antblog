/**
 * TCD-09 · Các chiến lược ưu tiên test case của CTFL v4.0.1 mục 5.1.5, viết thành hàm thuần.
 * Mọi chiến lược: hoà điểm ⇒ mã test case tăng dần (quy tắc tất định của bài, không phải của CTFL).
 */
import type { Risk, TestCaseSpec } from './bo-test-case';

export type Order = string[];

const byId = (a: TestCaseSpec, b: TestCaseSpec) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0);

/** Mức rủi ro của một test case = mức cao nhất trong các rủi ro nó giảm (khả năng × tác động). */
export function riskScore(tc: TestCaseSpec, risks: Risk[]): number {
  return Math.max(0, ...tc.risks.map((id) => {
    const r = risks.find((x) => x.id === id);
    if (!r) throw new Error(`${tc.id}: rủi ro ${id} không có trong sổ rủi ro`);
    return r.likelihood * r.impact;
  }));
}

/** Ưu tiên yêu cầu của test case = ưu tiên CAO nhất (số nhỏ nhất) trong các yêu cầu nó truy vết tới. */
export function requirementPriority(tc: TestCaseSpec, priorities: Record<string, number>): number {
  return Math.min(...tc.reqs.map((r) => priorities[r] ?? 99));
}

/** Risk-based: rủi ro cao chạy trước. */
export function riskBased(cases: TestCaseSpec[], risks: Risk[]): Order {
  return [...cases].sort((a, b) => riskScore(b, risks) - riskScore(a, risks) || byId(a, b)).map((c) => c.id);
}

/** Requirements-based: test case truy vết tới yêu cầu quan trọng nhất chạy trước. */
export function requirementsBased(cases: TestCaseSpec[], priorities: Record<string, number>): Order {
  return [...cases]
    .sort((a, b) => requirementPriority(a, priorities) - requirementPriority(b, priorities) || byId(a, b))
    .map((c) => c.id);
}

/** Coverage-based (tổng): test case phủ nhiều coverage item nhất chạy trước. */
export function totalCoverage(cases: TestCaseSpec[]): Order {
  return [...cases].sort((a, b) => b.covers.length - a.covers.length || byId(a, b)).map((c) => c.id);
}

/**
 * Coverage-based kiểu «additional coverage»: ca đầu là ca phủ nhiều nhất; mỗi ca tiếp theo
 * là ca phủ thêm được nhiều coverage item CHƯA phủ nhất. Trả kèm phần phủ thêm của từng bước.
 */
export function additionalCoverage(cases: TestCaseSpec[]): { order: Order; gains: number[] } {
  const remaining = [...cases].sort(byId);
  const covered = new Set<string>();
  const order: Order = [];
  const gains: number[] = [];
  while (remaining.length > 0) {
    let best = 0;
    let bestGain = -1;
    remaining.forEach((tc, i) => {
      const gain = tc.covers.filter((c) => !covered.has(c)).length;
      if (gain > bestGain) {
        best = i;
        bestGain = gain;
      }
    });
    const [tc] = remaining.splice(best, 1);
    tc!.covers.forEach((c) => covered.add(c));
    order.push(tc!.id);
    gains.push(bestGain);
  }
  return { order, gains };
}

/**
 * Sửa thứ tự theo phụ thuộc (CTFL 5.1.5): nếu ca ưu tiên cao phụ thuộc ca ưu tiên thấp hơn,
 * ca thấp hơn phải chạy trước. Cách làm: duyệt theo thứ tự ưu tiên, trước khi xếp một ca
 * thì xếp (đệ quy) các ca nó phụ thuộc mà chưa được xếp. Phát hiện vòng phụ thuộc.
 */
export function respectDependencies(order: Order, cases: TestCaseSpec[]): Order {
  const deps = new Map(cases.map((c) => [c.id, c.dependsOn]));
  const scheduled: Order = [];
  const done = new Set<string>();
  const visiting = new Set<string>();
  const visit = (id: string) => {
    if (done.has(id)) return;
    if (visiting.has(id)) throw new Error(`Vòng phụ thuộc tại ${id}`);
    if (!deps.has(id)) throw new Error(`Phụ thuộc vào ca không tồn tại: ${id}`);
    visiting.add(id);
    // tiền điều kiện cũng xếp theo thứ tự ưu tiên của chúng
    const pre = [...(deps.get(id) ?? [])].sort((a, b) => order.indexOf(a) - order.indexOf(b));
    pre.forEach(visit);
    visiting.delete(id);
    done.add(id);
    scheduled.push(id);
  };
  order.forEach(visit);
  return scheduled;
}

/** Kiểm một lịch chạy có tôn trọng mọi phụ thuộc không. */
export function violations(order: Order, cases: TestCaseSpec[]): string[] {
  const pos = new Map(order.map((id, i) => [id, i]));
  return cases.flatMap((c) =>
    c.dependsOn.filter((d) => (pos.get(d) ?? Infinity) > (pos.get(c.id) ?? -1)).map((d) => `${c.id} chạy trước ${d}`),
  );
}

/** Số coverage item đã phủ sau mỗi test case của lịch chạy. */
export function cumulativeCoverage(order: Order, cases: TestCaseSpec[]): number[] {
  const covered = new Set<string>();
  return order.map((id) => {
    cases.find((c) => c.id === id)!.covers.forEach((x) => covered.add(x));
    return covered.size;
  });
}

/** Rủi ro đã có ít nhất một test case chạy, sau k test case đầu. */
export function risksTouched(order: Order, cases: TestCaseSpec[], k: number): string[] {
  const touched = new Set<string>();
  order.slice(0, k).forEach((id) => cases.find((c) => c.id === id)!.risks.forEach((r) => touched.add(r)));
  return [...touched].sort();
}

/** Tổng mức rủi ro (Σ khả năng × tác động) của các rủi ro đã được chạm sau k ca. */
export function riskExposureCovered(order: Order, cases: TestCaseSpec[], risks: Risk[], k: number): number {
  return risksTouched(order, cases, k).reduce((sum, id) => {
    const r = risks.find((x) => x.id === id)!;
    return sum + r.likelihood * r.impact;
  }, 0);
}
