/**
 * In bộ pairwise và bảng so sánh của bài TEST-037.
 * Chạy: npx tsx articles/pairwise-testing/in-bo-test.ts
 */
import { APP_MODEL, FAULTS, faultsFound, L9_SUITE } from './app-config-model';
import { allRows, coverage, generate, label, randomSuite } from './pairwise';

const full = allRows(APP_MODEL);
const eachChoice = generate(APP_MODEL, 1);
const pairwise = generate(APP_MODEL, 2);
const threeWise = generate(APP_MODEL, 3);

const show = (prefix: string, suite: readonly (readonly number[])[]) =>
  suite.forEach((row, i) => {
    const v = label(APP_MODEL, row);
    console.log(`${prefix}${String(i + 1).padStart(2, '0')} | ${v.nenTang} | ${v.phienBan} | ${v.thanhToan} | ${v.ngonNgu} | ${v.mang}`);
  });

console.log('Bộ pairwise sinh tham lam:');
show('P', pairwise);
console.log('\nBộ 9 hàng (L9 + điền tay):');
show('L', L9_SUITE);

console.log('\nSo sánh (bộ test | số test | phủ cặp | lỗi chạm tới):');
for (const [name, suite] of [
  ['each-choice', eachChoice],
  ['pairwise (tham lam)', pairwise],
  ['pairwise (L9)', L9_SUITE],
  ['3-wise', threeWise],
  ['đầy đủ', full],
] as const) {
  const pc = coverage(APP_MODEL, suite, 2);
  console.log(`${name} | ${suite.length} | ${pc.covered}/${pc.feasible} | ${faultsFound(suite).join(',')}`);
}

const RUNS = 1000;
const hits = Object.fromEntries(FAULTS.map((f) => [f.id, 0]));
let pairSum = 0;
let fullPair = 0;
for (let seed = 1; seed <= RUNS; seed++) {
  const suite = randomSuite(APP_MODEL, pairwise.length, seed);
  const pc = coverage(APP_MODEL, suite, 2);
  pairSum += pc.percent;
  if (pc.covered === pc.feasible) fullPair++;
  for (const id of faultsFound(suite)) hits[id] = (hits[id] ?? 0) + 1;
}
console.log(`\n${RUNS} bộ ngẫu nhiên, mỗi bộ ${pairwise.length} hàng khác nhau:`);
console.log(`phủ cặp trung bình ${(pairSum / RUNS).toFixed(1)}%, bộ phủ đủ 100% cặp: ${fullPair}`);
console.log(`tỉ lệ chạm lỗi: ${FAULTS.map((f) => `${f.id} ${((100 * (hits[f.id] ?? 0)) / RUNS).toFixed(1)}%`).join(' · ')}`);
