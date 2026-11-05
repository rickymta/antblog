/**
 * Sinh bộ each-choice / 2-wise / 3-wise / 4-wise cho điều kiện voucher, chạy trên bản đúng và bốn bản lỗi,
 * so với bộ ngẫu nhiên cùng cỡ và với mô hình trộn giá trị lỗi.
 * Chạy: npx tsx articles/combinatorial-testing/do-suc-bat-loi.ts
 */
import { generate, allRows, coverage, randomSuite, type Row } from './t-wise';
import { fails, VARIANTS, type Variant } from './voucher-flow';
import { MIXED_MODEL, negativeCases, toScenario, VOUCHER_CONSTRAINTS, VOUCHER_MODEL } from './voucher-model';

const full = allRows(VOUCHER_MODEL, VOUCHER_CONSTRAINTS);
const failing = (v: Variant, rows: readonly Row[], model = VOUCHER_MODEL) =>
  rows.filter((r, i) => fails(v, toScenario(model, r, `#${i + 1}`))).length;

console.log(`Tích Descartes: ${allRows(VOUCHER_MODEL).length} · hợp lệ sau ràng buộc: ${full.length}`);
console.log('\nbộ | số test | thời gian sinh | ' + VARIANTS.join(' | ') + '   (số test đỏ)');
const suites: Record<string, Row[]> = {};
for (const t of [1, 2, 3, 4]) {
  const t0 = performance.now();
  const suite = generate(VOUCHER_MODEL, t, VOUCHER_CONSTRAINTS);
  const ms = performance.now() - t0;
  suites[t] = suite;
  const cov = coverage(VOUCHER_MODEL, suite, t, VOUCHER_CONSTRAINTS);
  if (cov.covered !== cov.feasible) throw new Error(`t=${t} chưa đủ phủ`);
  console.log(`t=${t} | ${suite.length} | ${ms.toFixed(0)} ms | ${VARIANTS.map((v) => failing(v, suite)).join(' | ')}   (${cov.feasible} bộ-${t} khả thi)`);
}
console.log(`đầy đủ | ${full.length} | — | ${VARIANTS.map((v) => failing(v, full)).join(' | ')}`);

const negatives = negativeCases();
console.log(`âm (base choice) | ${negatives.length} | — | ${VARIANTS.map((v) => negatives.filter((s) => fails(v, s)).length).join(' | ')}`);

const RUNS = 1000;
for (const t of [2, 3]) {
  const size = (suites[t] as Row[]).length;
  const hits = Object.fromEntries(VARIANTS.map((v) => [v, 0]));
  for (let seed = 1; seed <= RUNS; seed++) {
    const suite = randomSuite(full, size, seed);
    for (const v of VARIANTS) if (failing(v, suite) > 0) hits[v] = (hits[v] ?? 0) + 1;
  }
  console.log(`ngẫu nhiên ${size} hàng × ${RUNS}: ` + VARIANTS.map((v) => `${v} ${((100 * (hits[v] ?? 0)) / RUNS).toFixed(1)}%`).join(' · '));
}

const mixed3 = generate(MIXED_MODEL, 3, VOUCHER_CONSTRAINTS);
const errorRows = mixed3.filter((r) => !toScenario(MIXED_MODEL, r, '').expected.ok).length;
console.log(
  `\nmô hình trộn, t=3: ${mixed3.length} test, ${errorRows} test kết thúc bằng lỗi voucher; ` +
    VARIANTS.map((v) => `${v} ${failing(v, mixed3, MIXED_MODEL)}`).join(' · '),
);
// Hàng đi tới cuối (không lỗi) chỉ chứa giá trị hợp lệ, cùng chỉ số với VOUCHER_MODEL.
const reachedEnd = mixed3.filter((r) => toScenario(MIXED_MODEL, r, '').expected.ok);
const real3 = coverage(VOUCHER_MODEL, reachedEnd, 3, VOUCHER_CONSTRAINTS);
console.log(`bộ-3 hợp lệ thực sự chạy tới bước lượt dùng: ${real3.covered}/${real3.feasible} (${real3.percent.toFixed(1)}%)`);
