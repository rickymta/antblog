/**
 * TEST-041 · Random testing trên lõi định giá THẬT (src/domain/pricing.ts).
 * Chạy: npx vitest run articles/random-testing
 */
import fc from 'fast-check';
import { describe, expect, it } from 'vitest';
import { profileCase, unguidedCase, type QuoteCase } from './generators';
import { judge, srcPrice } from './oracle';

const SEED = 41;
const RUNS = 10_000;

function tally(map: Map<string, number>, key: string): void {
  map.set(key, (map.get(key) ?? 0) + 1);
}

function printTable(title: string, map: Map<string, number>, total: number): void {
  const rows = [...map.entries()].sort((a, b) => b[1] - a[1]);
  console.log(`\n${title}`);
  for (const [k, n] of rows) console.log(`  ${k.padEnd(26)} ${String(n).padStart(6)}  ${((100 * n) / total).toFixed(1).padStart(5)}%`);
}

describe('TEST-041 · random testing — lõi định giá thật', () => {
  it('RT-01 phân phối cố định (verification): 10.000 yêu cầu ngẫu nhiên, oracle O1–O6 + mã lỗi R-VALID-01', () => {
    const outcomes = new Map<string, number>();
    let foodNotThousand = 0;
    const started = performance.now();
    fc.assert(
      fc.property(unguidedCase, (c: QuoteCase) => {
        const { outcome, violations } = judge(srcPrice, c);
        tally(outcomes, outcome.kind === 'ok' ? `OK ${c.req.kind}` : outcome.code);
        if (outcome.kind === 'ok' && c.req.kind === 'food' && outcome.result.total % 1_000 !== 0) foodNotThousand++;
        return violations.length === 0;
      }),
      { seed: SEED, numRuns: RUNS },
    );
    const ms = performance.now() - started;
    printTable(`RT-01 · ${RUNS} ca, seed ${SEED}, ${ms.toFixed(0)} ms — kết cục:`, outcomes, RUNS);
    console.log(`  đơn đồ ăn hợp lệ có tổng KHÔNG chia hết 1.000 đ: ${foodNotThousand} (đúng SPEC: chỉ phí giao được làm tròn)`);
    expect(foodNotThousand).toBeGreaterThan(0);
  });

  it('RT-02 operational profile (validation): 10.000 yêu cầu theo tỉ lệ dùng thật, không vi phạm nào', () => {
    const kinds = new Map<string, number>();
    let failures = 0;
    fc.assert(
      fc.property(profileCase, (c: QuoteCase) => {
        const { outcome, violations } = judge(srcPrice, c);
        tally(kinds, `${c.req.kind}${c.peak ? ' cao điểm' : ''}`);
        if (outcome.kind === 'error') tally(kinds, `lỗi ${outcome.code}`);
        if (violations.length > 0) failures++;
        return violations.length === 0;
      }),
      { seed: SEED, numRuns: RUNS },
    );
    printTable(`RT-02 · ${RUNS} ca theo profile — phân bố:`, kinds, RUNS);
    // «Quy tắc ba» (ngoài syllabus): 0 failure trong n lần thử độc lập ⇒ xác suất failure < 3/n với độ tin cậy ~95%.
    console.log(`  failure: ${failures} ⇒ cận trên xác suất failure mỗi yêu cầu ≈ ${((3 / RUNS) * 100).toFixed(2)}% (theo profile này)`);
    expect(failures).toBe(0);
  });

  it('RT-03 tái lập được: cùng seed ⇒ cùng chuỗi đầu vào', () => {
    const a = fc.sample(unguidedCase, { seed: SEED, numRuns: 3 });
    const b = fc.sample(unguidedCase, { seed: SEED, numRuns: 3 });
    expect(a).toEqual(b);
    console.log('\nRT-03 · 3 ca đầu với seed 41:');
    for (const c of a) console.log('  ' + JSON.stringify({ ...c.req, peak: c.peak, raining: c.raining, voucher: c.voucher?.percent ?? null }));
  });
});
