/**
 * TCD-06 · Defect masking: chạy các bộ ca trên bản chép có hai lỗi cố ý (ban-loi.ts).
 */
import { describe, expect, it } from 'vitest';
import { buggyOutcome } from './ban-loi';
import {
  BVA_SUITE, DT_SUITE, EP_SUITE, MASKED_CASE, POOL, coverageItems, expectedOutcome, greedyMinimize, type QuoteCase,
} from './mo-hinh';

/** Ca nào cho kết quả khác oracle trên bản lỗi (status hoặc mã lỗi). */
function redOn(suite: QuoteCase[]): string[] {
  return suite
    .filter((c) => {
      const exp = expectedOutcome(c);
      const got = buggyOutcome(c);
      return got.status !== exp.status || (exp.status !== 200 && got.code !== exp.code);
    })
    .map((c) => c.id);
}

describe('MASK-01: khai «phủ» nhiều, thật sự phủ ít', () => {
  it('chỉ phủ C2 và cột R2 — 0,44 km và xe máy 25 kg bị che', () => {
    expect(coverageItems(MASKED_CASE)).toEqual(['EP:C2', 'DT:R2']);
  });

  it('bản lỗi vẫn trả CITY_NOT_SUPPORTED ⇒ ca gộp xanh trên cả bản đúng lẫn bản lỗi', () => {
    expect(buggyOutcome(MASKED_CASE)).toEqual({ status: 422, code: 'CITY_NOT_SUPPORTED' });
    expect(redOn([MASKED_CASE])).toEqual([]);
  });
});

describe('Bộ ca nào bắt được lỗi Q (quên cận dưới quãng đường) và lỗi X (quên R-PARCEL-02)?', () => {
  it('bộ EP: bắt Q (EP-08 · 0,2 km), bỏ lọt X', () => {
    expect(redOn(EP_SUITE)).toEqual(['EP-08']);
  });
  it('bộ DT: bắt X (DT-R5), bỏ lọt Q vì cột R3 chọn 150 km', () => {
    expect(redOn(DT_SUITE)).toEqual(['DT-R5']);
  });
  it('bộ BVA: bắt cả hai (BVA-01 · 0,44 km; BVA-10 · xe máy 20,05 kg)', () => {
    expect(redOn(BVA_SUITE)).toEqual(['BVA-01', 'BVA-10']);
  });
  it('bộ cuối 18 ca: bắt cả hai', () => {
    expect(redOn(greedyMinimize(POOL)).sort()).toEqual(['BVA-01', 'BVA-10']);
  });
});
