/**
 * TCD-05 · Cùng ba test case về lượt dùng voucher (R-VOUCHER-07), viết theo hai cách dựng test data:
 *  - «fixture dùng chung»: một hệ thống cho cả bộ (kiểu beforeAll), các ca dựa vào dữ liệu ca trước để lại;
 *  - «builder»: mỗi ca tự dựng dữ liệu tiền điều kiện của mình.
 * Ta chạy cả 6 thứ tự có thể của ba ca và đếm số thứ tự cho kết quả xanh trọn vẹn.
 */
import { describe, expect, it } from 'vitest';
import type { Actor, AppDeps } from '../../src/app/deps';
import { createQuote } from '../../src/app/quote';
import { createTrip } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { aRideRequest, aVoucher, aWorld } from './test-data-builder';

type CaseId = 'TC-VU-001' | 'TC-VU-002' | 'TC-VU-003';
type Ctx = { deps: AppDeps; customer: Actor };
type Mode = 'fixture-dung-chung' | 'fixture-co-reset' | 'builder';

const NINE_AM = vnTime(2026, 11, 16, 9, 0, 0);

// ---- thân của ba test case: giống hệt nhau ở hai chế độ, chỉ khác chỗ lấy Ctx
const body: Record<CaseId, (ctx: Ctx) => Promise<void>> = {
  // Báo giá với voucher khi khách CHƯA dùng lượt nào hôm nay ⇒ được giảm 20%
  'TC-VU-001': async ({ deps, customer }) => {
    const q = await createQuote(deps, customer, aRideRequest({ voucherCode: 'THU20' }));
    expect(q.details).toMatchObject({ discount: 9_060, total: 37_000 });
  },
  // Đặt chuyến bằng voucher ⇒ chuyến được tạo và mang mã voucher (tiêu một lượt)
  'TC-VU-002': async ({ deps, customer }) => {
    const q = await createQuote(deps, customer, aRideRequest({ voucherCode: 'THU20' }));
    const trip = await createTrip(deps, customer, { quoteId: q.id });
    expect(trip).toMatchObject({ voucherCode: 'THU20', fare: 37_000, status: 'searching' });
  },
  // Khách ĐÃ có một chuyến dùng voucher hôm nay ⇒ báo giá bị VOUCHER_USAGE_LIMIT
  'TC-VU-003': async ({ deps, customer }) => {
    await expect(createQuote(deps, customer, aRideRequest({ voucherCode: 'THU20' }))).rejects.toMatchObject({
      code: 'VOUCHER_USAGE_LIMIT',
    });
  },
};

/** Chạy ba ca theo `order`, ghi lại đạt/trượt của từng ca (không dừng ở ca trượt đầu tiên). */
async function runSuite(order: CaseId[], mode: Mode): Promise<Record<CaseId, 'pass' | 'fail'>> {
  const result = {} as Record<CaseId, 'pass' | 'fail'>;
  // fixture dùng chung: dựng MỘT lần cho cả bộ, như beforeAll
  const shared = (() => {
    const clock = new FixedClock(NINE_AM);
    const system = createMemorySystem({
      clock,
      seed: { customers: [{ id: 'C901', phone: '0900000901', fullName: 'Khách Thử 01' }], drivers: [], vouchers: [aVoucher()] },
    });
    return { system, deps: system.deps, customer: { role: 'customer', id: 'C901' } as Actor };
  })();

  for (const id of order) {
    let ctx: Ctx;
    if (mode === 'fixture-dung-chung') {
      ctx = shared;
    } else if (mode === 'fixture-co-reset') {
      await shared.system.reset(); // dọn về dữ liệu gốc trước MỖI ca, như beforeEach(reset)
      ctx = shared;
    } else {
      // builder: mỗi ca khai báo đúng tiền điều kiện của nó
      const builder = aWorld().at(NINE_AM);
      if (id === 'TC-VU-003') builder.withVoucherTrip(vnTime(2026, 11, 16, 8, 30, 0)); // đã dùng 1 lượt lúc 08:30
      const w = await builder.build();
      ctx = { deps: w.deps, customer: w.customer };
    }
    try {
      await body[id](ctx);
      result[id] = 'pass';
    } catch {
      result[id] = 'fail';
    }
  }
  return result;
}

const ORDERS: CaseId[][] = [
  ['TC-VU-001', 'TC-VU-002', 'TC-VU-003'],
  ['TC-VU-001', 'TC-VU-003', 'TC-VU-002'],
  ['TC-VU-002', 'TC-VU-001', 'TC-VU-003'],
  ['TC-VU-002', 'TC-VU-003', 'TC-VU-001'],
  ['TC-VU-003', 'TC-VU-001', 'TC-VU-002'],
  ['TC-VU-003', 'TC-VU-002', 'TC-VU-001'],
];

const short = (o: CaseId[]) => o.map((x) => x.slice(-1)).join('-');

describe('Fixture dùng chung: kết quả phụ thuộc thứ tự chạy', () => {
  // Ca 001 chỉ đạt nếu 002 chạy SAU nó; ca 003 chỉ đạt nếu 002 chạy TRƯỚC nó.
  const expected: Record<string, Record<CaseId, 'pass' | 'fail'>> = {
    '1-2-3': { 'TC-VU-001': 'pass', 'TC-VU-002': 'pass', 'TC-VU-003': 'pass' },
    '1-3-2': { 'TC-VU-001': 'pass', 'TC-VU-002': 'pass', 'TC-VU-003': 'fail' },
    '2-1-3': { 'TC-VU-001': 'fail', 'TC-VU-002': 'pass', 'TC-VU-003': 'pass' },
    '2-3-1': { 'TC-VU-001': 'fail', 'TC-VU-002': 'pass', 'TC-VU-003': 'pass' },
    '3-1-2': { 'TC-VU-001': 'pass', 'TC-VU-002': 'pass', 'TC-VU-003': 'fail' },
    '3-2-1': { 'TC-VU-001': 'fail', 'TC-VU-002': 'pass', 'TC-VU-003': 'fail' },
  };
  it.each(ORDERS.map((o) => [short(o), o] as const))('thứ tự %s', async (key, order) => {
    expect(await runSuite(order, 'fixture-dung-chung')).toEqual(expected[key]);
  });
});

describe('Builder: mỗi ca tự dựng tiền điều kiện ⇒ mọi thứ tự đều xanh', () => {
  it.each(ORDERS.map((o) => [short(o), o] as const))('thứ tự %s', async (_key, order) => {
    const r = await runSuite(order, 'builder');
    expect(Object.values(r)).toEqual(['pass', 'pass', 'pass']);
  });
});

describe('Fixture dùng chung + reset trước mỗi ca: hết rò dữ liệu, nhưng ca 003 thiếu tiền điều kiện', () => {
  it('ca 001, 002 luôn đạt; ca 003 luôn trượt ⇒ 0/6 thứ tự xanh', async () => {
    for (const o of ORDERS) {
      expect(await runSuite(o, 'fixture-co-reset')).toEqual({ 'TC-VU-001': 'pass', 'TC-VU-002': 'pass', 'TC-VU-003': 'fail' });
    }
  });
});

describe('Tổng kết', () => {
  it('fixture dùng chung: 1/6 thứ tự xanh, 6/18 lượt chạy ca bị đỏ; builder: 6/6 thứ tự, 0/18', async () => {
    const tally = async (mode: Mode) => {
      let greenOrders = 0;
      let failedRuns = 0;
      for (const o of ORDERS) {
        const r = Object.values(await runSuite(o, mode));
        if (r.every((x) => x === 'pass')) greenOrders++;
        failedRuns += r.filter((x) => x === 'fail').length;
      }
      return { greenOrders, failedRuns };
    };
    expect(await tally('fixture-dung-chung')).toEqual({ greenOrders: 1, failedRuns: 6 });
    expect(await tally('builder')).toEqual({ greenOrders: 6, failedRuns: 0 });
  });
});
