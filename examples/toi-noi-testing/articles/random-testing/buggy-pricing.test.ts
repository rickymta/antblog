/**
 * TEST-041 · Random testing trên hai bản CÓ LỖI (buggy-pricing.ts).
 * Chạy: npx vitest run articles/random-testing
 * Xem báo cáo gốc của fast-check (test đỏ thật): SHOW_RED=1 npx vitest run articles/random-testing
 */
import fc from 'fast-check';
import { describe, expect, it } from 'vitest';
import { buggyPrice } from './buggy-pricing';
import { uniformCase, unguidedCase, type QuoteCase } from './generators';
import { judge, run, srcPrice } from './oracle';

const property = (bug: Parameters<typeof buggyPrice>[0], arb = unguidedCase) =>
  fc.property(arb, (c: QuoteCase) => judge(buggyPrice(bug), c).violations.length === 0);

function brief(c: QuoteCase): string {
  const { req, peak, raining, voucher } = c;
  return JSON.stringify({ ...req, peak, raining, voucher });
}

describe('TEST-041 · random testing — bản có lỗi', () => {
  it('BG-01 «tối thiểu trước voucher»: oracle O3 bắt được, fast-check thu nhỏ phản ví dụ', () => {
    const d = fc.check(property('minimum-before-voucher'), { seed: 41, numRuns: 10_000 });
    expect(d.failed).toBe(true);
    const c = d.counterexample![0];
    const out = run(buggyPrice('minimum-before-voucher'), c);
    console.log(`\nBG-01 · thất bại ở lần chạy thứ ${d.numRuns}, thu nhỏ ${d.numShrinks} bước`);
    console.log('  phản ví dụ: ' + brief(c));
    console.log('  vi phạm   : ' + judge(buggyPrice('minimum-before-voucher'), c).violations.join('; '));
    if (out.kind === 'ok') console.log(`  bản lỗi ra ${out.result.total} đ; bản đúng ra ${(run(srcPrice, c) as { result: { total: number } }).result.total} đ`);
  });

  it('BG-02 xác suất phát hiện phụ thuộc số lần chạy: 20 seed × {100, 1.000} lần', () => {
    const found = (numRuns: number) =>
      Array.from({ length: 20 }, (_, i) => fc.check(property('minimum-before-voucher'), { seed: 1_000 + i, numRuns }).failed).filter(Boolean).length;
    const at100 = found(100);
    const at1000 = found(1_000);
    console.log(`\nBG-02 · số seed (trên 20) tìm ra lỗi: numRuns=100 ⇒ ${at100}; numRuns=1.000 ⇒ ${at1000}`);
    expect(at1000).toBeGreaterThanOrEqual(at100);
  });

  it('BG-04 đổi phân phối theo hiểu biết miền (chuyến 0–3 km): lỗi lộ sau vài chục lần chạy', () => {
    const runs = Array.from({ length: 20 }, (_, i) => {
      const d = fc.check(property('minimum-before-voucher', uniformCase(3_000)), { seed: 1_000 + i, numRuns: 1_000 });
      return d.failed ? d.numRuns : Infinity;
    });
    const hit = runs.filter(Number.isFinite).sort((a, b) => a - b);
    console.log(`
BG-04 · 0–3 km: ${hit.length}/20 seed tìm ra lỗi trong 1.000 lần; số lần chạy tới failure: trung vị ${hit[Math.floor(hit.length / 2)]}, nhỏ nhất ${hit[0]}, lớn nhất ${hit[hit.length - 1]}`);
    expect(hit.length).toBe(20);
  });

  it('BG-03 «mưa bị nhân cao điểm»: 10.000 ca vẫn XANH — oracle tính chất quá yếu', () => {
    const d = fc.check(property('rain-times-peak'), { seed: 41, numRuns: 10_000 });
    expect(d.failed).toBe(false);
    // Một ca có oracle cụ thể (R-FARE-09) lộ lỗi ngay: xe máy 7,4 km, cao điểm, mưa, giảm 20%.
    const r09: QuoteCase = {
      req: { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 }, meters: 7_400, grams: null,
      peak: true, raining: true, voucher: { code: 'GIAM20', percent: 20, maxDiscount: 1_000_000, minAmount: 0 },
    };
    const buggy = run(buggyPrice('rain-times-peak'), r09);
    const good = run(srcPrice, r09);
    console.log(`\nBG-03 · 10.000 ca: failed=${d.failed}. R-FARE-09: bản lỗi ${buggy.kind === 'ok' ? buggy.result.total : '?'} đ, bản đúng ${good.kind === 'ok' ? good.result.total : '?'} đ (oracle 52.000 đ)`);
    expect(buggy.kind === 'ok' && buggy.result.total).toBe(54_000);
  });

  it.runIf(process.env.SHOW_RED)('SHOW_RED · báo cáo gốc của fast-check cho bản «tối thiểu trước voucher»', () => {
    fc.assert(property('minimum-before-voucher'), { seed: 41, numRuns: 10_000 });
  });
});
