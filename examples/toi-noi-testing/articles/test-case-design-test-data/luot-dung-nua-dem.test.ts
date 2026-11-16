/**
 * TCD-05 · Lượt dùng voucher trong ngày quanh nửa đêm giờ Việt Nam (R-VOUCHER-07, R-GEN-02).
 * Mỗi dòng TD tự dựng dữ liệu tiền điều kiện bằng builder; kết quả mong đợi suy từ SPEC.
 * Một bản cài đặt lỗi «tính ngày theo UTC» được chạy trên cùng bảng để xem dòng nào bắt được nó.
 */
import { describe, expect, it } from 'vitest';
import type { AppDeps } from '../../src/app/deps';
import { createQuote } from '../../src/app/quote';
import { createTrip } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import { aCustomer, aRideRequest, aVoucher, aWorld } from './test-data-builder';

const vn = (d: number, h: number, m: number, s: number) => vnTime(2026, 11, d, h, m, s);

interface TdRow {
  id: string;
  /** Mô tả dữ liệu tiền điều kiện. */
  pre: string;
  prior: { at: Date; cancelled?: boolean; customerId?: string }[];
  perDay?: number;
  quoteAt: Date;
  expected: 'OK' | 'VOUCHER_USAGE_LIMIT';
}

const TD: TdRow[] = [
  { id: 'TD-01', pre: 'chuyến lúc 15/11 23:59:58', prior: [{ at: vn(15, 23, 59, 58) }], quoteAt: vn(15, 23, 59, 59), expected: 'VOUCHER_USAGE_LIMIT' },
  { id: 'TD-02', pre: 'chuyến lúc 15/11 23:59:58', prior: [{ at: vn(15, 23, 59, 58) }], quoteAt: vn(16, 0, 0, 0), expected: 'OK' },
  { id: 'TD-03', pre: 'chuyến lúc 16/11 00:00:00', prior: [{ at: vn(16, 0, 0, 0) }], quoteAt: vn(16, 0, 0, 1), expected: 'VOUCHER_USAGE_LIMIT' },
  { id: 'TD-04', pre: 'chuyến lúc 16/11 06:59:59 (= 15/11 23:59:59Z)', prior: [{ at: vn(16, 6, 59, 59) }], quoteAt: vn(16, 7, 0, 0), expected: 'VOUCHER_USAGE_LIMIT' },
  { id: 'TD-05', pre: 'chuyến lúc 15/11 23:59:58 đã bị khách huỷ', prior: [{ at: vn(15, 23, 59, 58), cancelled: true }], quoteAt: vn(15, 23, 59, 59), expected: 'OK' },
  { id: 'TD-06', pre: 'KHÁCH KHÁC có chuyến lúc 16/11 08:30', prior: [{ at: vn(16, 8, 30, 0), customerId: 'C902' }], quoteAt: vn(16, 9, 0, 0), expected: 'OK' },
  { id: 'TD-07', pre: 'voucher 2 lượt/ngày, đã dùng 1 lượt lúc 08:00', perDay: 2, prior: [{ at: vn(16, 8, 0, 0) }], quoteAt: vn(16, 9, 0, 0), expected: 'OK' },
  { id: 'TD-08', pre: 'voucher 2 lượt/ngày, đã dùng 2 lượt lúc 08:00 và 08:30', perDay: 2, prior: [{ at: vn(16, 8, 0, 0) }, { at: vn(16, 8, 30, 0) }], quoteAt: vn(16, 9, 0, 0), expected: 'VOUCHER_USAGE_LIMIT' },
];

async function worldFor(row: TdRow) {
  const b = aWorld().withCustomer(aCustomer(2)).withVoucher(aVoucher({ perCustomerPerDay: row.perDay ?? 1 })).at(row.quoteAt);
  for (const p of row.prior) b.withVoucherTrip(p.at, { cancelled: p.cancelled, customerId: p.customerId });
  return b.build();
}

describe('Bảng TD: lượt dùng trong ngày theo giờ Việt Nam (cài đặt của lab)', () => {
  it.each(TD)('$id · $pre ⇒ $expected', async (row) => {
    const w = await worldFor(row);
    const attempt = createQuote(w.deps, w.customer, aRideRequest({ voucherCode: 'THU20' }));
    if (row.expected === 'OK') {
      await expect(attempt).resolves.toMatchObject({ details: { discount: 9_060, total: 37_000 } });
    } else {
      await expect(attempt).rejects.toMatchObject({ code: 'VOUCHER_USAGE_LIMIT' });
    }
  });
});

/** Bản lỗi cố ý: đếm lượt trong NGÀY UTC thay vì ngày giờ Việt Nam. Không sửa src/ — chỉ thay hàm khoảng ngày. */
async function buggyUtcDayQuotaOk(deps: AppDeps, customerId: string, at: Date, perDay: number): Promise<boolean> {
  const start = new Date(Date.UTC(at.getUTCFullYear(), at.getUTCMonth(), at.getUTCDate()));
  const end = new Date(start.getTime() + 24 * 3_600_000);
  const used = await deps.trips.countVoucherUses({ customerId, voucherCode: 'THU20', from: start, to: end });
  return used < perDay;
}

describe('Bản lỗi «ngày UTC» chạy trên cùng bảng TD', () => {
  it('chỉ TD-02 (nửa đêm VN) và TD-04 (nửa đêm UTC = 07:00 VN) bắt được lỗi', async () => {
    const caught: string[] = [];
    for (const row of TD) {
      const w = await worldFor(row);
      const ok = await buggyUtcDayQuotaOk(w.deps, w.customer.id, row.quoteAt, row.perDay ?? 1);
      if ((ok ? 'OK' : 'VOUCHER_USAGE_LIMIT') !== row.expected) caught.push(row.id);
    }
    expect(caught).toEqual(['TD-02', 'TD-04']);
  });
});

describe('Lượt được kiểm lại khi tạo chuyến, theo ngày của thời điểm tạo chuyến (R-VOUCHER-07)', () => {
  async function twoQuotesBeforeMidnight() {
    const w = await aWorld().at(vn(15, 23, 57, 0)).build();
    const q1 = await createQuote(w.deps, w.customer, aRideRequest({ voucherCode: 'THU20' }));
    w.clock.set(vn(15, 23, 57, 30));
    const q2 = await createQuote(w.deps, w.customer, aRideRequest({ voucherCode: 'THU20' })); // báo giá không tiêu lượt
    w.clock.set(vn(15, 23, 58, 0));
    await createTrip(w.deps, w.customer, { quoteId: q1.id }); // tiêu lượt ngày 15/11
    return { w, q2 };
  }

  it('TD-09 · tạo chuyến từ báo giá thứ hai lúc 15/11 23:59:00 ⇒ VOUCHER_USAGE_LIMIT', async () => {
    const { w, q2 } = await twoQuotesBeforeMidnight();
    w.clock.set(vn(15, 23, 59, 0));
    await expect(createTrip(w.deps, w.customer, { quoteId: q2.id })).rejects.toMatchObject({ code: 'VOUCHER_USAGE_LIMIT' });
  });

  it('TD-10 · cùng báo giá đó, tạo chuyến lúc 16/11 00:00:00 ⇒ được (báo giá còn hạn tới 00:02:30); lượt tính cho 16/11', async () => {
    const { w, q2 } = await twoQuotesBeforeMidnight();
    expect(q2.expiresAt.toISOString()).toBe('2026-11-15T17:02:30.000Z'); // 16/11 00:02:30 giờ VN
    w.clock.set(vn(16, 0, 0, 0));
    const trip = await createTrip(w.deps, w.customer, { quoteId: q2.id });
    expect(trip).toMatchObject({ voucherCode: 'THU20', fare: 37_000 });
    expect(trip.requestedAt.toISOString()).toBe('2026-11-15T17:00:00.000Z');
    // ngày 16/11 đã hết lượt vì chính chuyến vừa tạo
    w.clock.set(vn(16, 0, 1, 0));
    await expect(createQuote(w.deps, w.customer, aRideRequest({ voucherCode: 'THU20' }))).rejects.toMatchObject({
      code: 'VOUCHER_USAGE_LIMIT',
    });
  });
});
