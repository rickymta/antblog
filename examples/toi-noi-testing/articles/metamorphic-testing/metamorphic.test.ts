/**
 * TEST-042 · Metamorphic testing cho thuật toán cước của lab.
 * Chạy: npx vitest run articles/metamorphic-testing
 * Xem báo cáo gốc của fast-check (đỏ cố ý): SHOW_RED=1 npx vitest run articles/metamorphic-testing -t SHOW_RED
 */
import fc from 'fast-check';
import { describe, expect, it } from 'vitest';
import { MR1, MR1_BUOC_LON, MR2, MR3, MR4, MR_SAI, MR_YEU, mrProperty, type MR, type Source } from './relations';
import { BUGS, buggyPrice, srcPrice, type PriceFn } from './variants';
import { unguidedCase } from '../random-testing/generators';
import { judge } from '../random-testing/oracle';

const SEED = 42;
const RUNS = 2_000;
const RELATIONS = [MR1, MR2, MR3, MR4] as MR<unknown>[];

function check(price: PriceFn, mr: MR<unknown>, numRuns = RUNS) {
  return fc.check(mrProperty(price, mr), { seed: SEED, numRuns });
}

function show(s: Source): string {
  return JSON.stringify({ ...s.req, peak: s.c.peak, raining: s.c.raining, voucher: s.c.voucher && `${s.c.voucher.percent}% trần ${s.c.voucher.maxDiscount} tối thiểu ${s.c.voucher.minAmount}` });
}

describe('TEST-042 · metamorphic testing — thuật toán cước', () => {
  it('MT-01 ví dụ tay: source R-FARE-09 và ba follow-up', () => {
    const req = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 } as const;
    const voucher = { code: 'GIAM20', percent: 20, maxDiscount: 1_000_000, minAmount: 0 };
    const rows: [string, number, boolean, boolean][] = [
      ['source (R-FARE-09)', 7.4, true, true],
      ['MR1 follow-up: 7,5 km', 7.5, true, true],
      ['MR2 follow-up: tắt mưa', 7.4, true, false],
      ['MR3 follow-up: giờ thường', 7.4, false, true],
    ];
    console.log('\nMT-01 · tổng và «trước giảm» của source/follow-up:');
    for (const [name, km, peak, raining] of rows) {
      const r = srcPrice({ ...req, distanceKm: km }, { peak, raining, voucher });
      console.log(`  ${name.padEnd(26)} trước giảm ${String(r.fare.beforeDiscount).padStart(8)}  tổng ${r.total}`);
    }
    expect(srcPrice(req, { peak: true, raining: true, voucher }).total).toBe(52_000);
  });

  it('MT-02 bản thật: MR1–MR4 đều thoả trên 2.000 cặp mỗi MR', () => {
    const lines = RELATIONS.map((mr) => {
      const d = check(srcPrice, mr);
      expect(d.failed, mr.id).toBe(false);
      return `  ${mr.id.padEnd(4)} ${mr.text.padEnd(48)} chạy ${d.numRuns}, bỏ qua ${d.numSkips}`;
    });
    console.log('\nMT-02 · src (seed 42):\n' + lines.join('\n'));
  });

  it('MT-03 ma trận bản lỗi × MR (10.000 cặp mỗi ô): MR nào «giết» lỗi nào', () => {
    const expected: Record<string, string> = {
      'B1 rain-times-peak': '·X··',
      'B2 minimum-before-voucher': 'X···',
      'B3 round-nearest': '····',
      'B4 voucher-on-base': '···X',
    };
    console.log('\nMT-03 · X = MR phát hiện (đỏ), · = MR vẫn thoả');
    console.log('  ' + 'bản lỗi'.padEnd(28) + RELATIONS.map((m) => m.id.padEnd(5)).join(''));
    for (const bug of BUGS) {
      const row = RELATIONS.map((mr) => (check(buggyPrice(bug), mr, 10_000).failed ? 'X' : '·')).join('');
      console.log('  ' + bug.padEnd(28) + row.split('').map((x) => x.padEnd(5)).join(''));
      expect(row, bug).toBe(expected[bug]);
    }
  });

  it('MT-04 chẩn đoán: phản ví dụ MR2 trên B1 — ca nào trong cặp sai?', () => {
    const d = check(buggyPrice('B1 rain-times-peak'), MR2 as MR<unknown>);
    expect(d.failed).toBe(true);
    const s = d.counterexample![0] as Source;
    const dry = buggyPrice('B1 rain-times-peak')(s.req, { ...s.c, raining: false });
    const wet = buggyPrice('B1 rain-times-peak')(s.req, { ...s.c, raining: true });
    const wetSrc = srcPrice(s.req, { ...s.c, raining: true });
    console.log(`\nMT-04 · thu nhỏ ${d.numShrinks} bước: ${show(s)}`);
    console.log(`  source (khô)   : trước giảm ${dry.fare.beforeDiscount}, tổng ${dry.total}`);
    console.log(`  follow-up (mưa): trước giảm ${wet.fare.beforeDiscount}, tổng ${wet.total}  ⇒ chênh ${wet.fare.beforeDiscount - dry.fare.beforeDiscount}`);
    console.log(`  đối chiếu bản thật cho follow-up: trước giảm ${wetSrc.fare.beforeDiscount}, tổng ${wetSrc.total}`);
  });

  it('MT-05 MR sai và MR yếu', () => {
    const wrong = check(srcPrice, MR_SAI as MR<unknown>);
    expect(wrong.failed).toBe(true); // bản thật «trượt» ⇒ lỗi ở MR, không ở code
    const s = wrong.counterexample![0] as Source;
    const dry = srcPrice(s.req, { ...s.c, raining: false });
    const wet = srcPrice(s.req, { ...s.c, raining: true });
    console.log(`\nMT-05 · MR-sai trên BẢN THẬT: ${show(s)}`);
    console.log(`  khô: sau giảm ${dry.fare.afterDiscount}, tổng ${dry.total} · mưa: sau giảm ${wet.fare.afterDiscount}, tổng ${wet.total} ⇒ chênh ${wet.total - dry.total}`);
    const weak = check(buggyPrice('B1 rain-times-peak'), MR_YEU as MR<unknown>);
    console.log(`  MR-yếu trên B1: failed=${weak.failed} (MR2 cùng nguồn sinh thì đỏ)`);
    expect(weak.failed).toBe(false);
  });

  it('MT-06 cỡ biến đổi của follow-up: MR1 bước 0,1 km vs bước 0,1–5 km, 50 seed × 2.000 cặp trên B2', () => {
    const seeds = Array.from({ length: 50 }, (_, i) => 500 + i);
    const hit = (mr: MR<unknown>) =>
      seeds.filter((seed) => fc.check(mrProperty(buggyPrice('B2 minimum-before-voucher'), mr), { seed, numRuns: RUNS }).failed).length;
    const small = hit(MR1 as MR<unknown>);
    const large = hit(MR1_BUOC_LON as MR<unknown>);
    console.log(`
MT-06 · số seed (trên 50) MR1 phát hiện B2: bước 0,1 km ⇒ ${small}; bước 0,1–5 km ⇒ ${large}`);
    expect(small).toBeGreaterThan(large);
  });

  it('MT-07 đối chiếu: oracle tính chất O1–O6 của TEST-041 trên cùng bốn bản lỗi (seed 41, 10.000 ca)', () => {
    const row = BUGS.map((bug) => fc.check(fc.property(unguidedCase, (c) => judge(buggyPrice(bug), c).violations.length === 0), { seed: 41, numRuns: 10_000 }).failed ? 'X' : '·').join('');
    console.log(`
MT-07 · oracle tính chất TEST-041 — B1 B2 B3 B4: ${row.split('').join('  ')}`);
    expect(row).toBe('·XX·');
  });

  it.runIf(process.env.SHOW_RED)('SHOW_RED · báo cáo gốc của fast-check: MR1 trên B2', () => {
    fc.assert(mrProperty(buggyPrice('B2 minimum-before-voucher'), MR1), { seed: SEED, numRuns: 10_000 });
  });
});
