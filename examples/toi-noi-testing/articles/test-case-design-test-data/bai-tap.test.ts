/**
 * TCD-05 · Kiểm lời giải Bài tập 1 của bài: biên ngày hiệu lực voucher (R-VOUCHER-04) theo giờ Việt Nam,
 * với một voucher giả lập có cùng thông số như XMAS50 của Phụ lục A (24/12/2025–26/12/2025).
 */
import { describe, expect, it } from 'vitest';
import { createQuote } from '../../src/app/quote';
import { vnTime } from '../../src/domain/time';
import { aRideRequest, aVoucher, aWorld } from './test-data-builder';

const XMAS = aVoucher({ code: 'XMAS50', percent: 50, maxDiscount: 50_000, validFrom: '2025-12-24', validTo: '2025-12-26' });

describe('Bài tập 1 · biên ngày hiệu lực theo giờ VN', () => {
  it.each([
    { label: '23/12 23:59:59 VN (= 16:59:59Z)', at: vnTime(2025, 12, 23, 23, 59, 59), ok: false },
    { label: '24/12 00:00:00 VN (= 23/12 17:00:00Z)', at: vnTime(2025, 12, 24, 0, 0, 0), ok: true },
    { label: '26/12 23:59:59 VN (= 16:59:59Z)', at: vnTime(2025, 12, 26, 23, 59, 59), ok: true },
    { label: '27/12 00:00:00 VN (= 26/12 17:00:00Z)', at: vnTime(2025, 12, 27, 0, 0, 0), ok: false },
  ])('$label ⇒ hợp lệ: $ok', async ({ at, ok }) => {
    const w = await aWorld().withVoucher(XMAS).at(at).build();
    const attempt = createQuote(w.deps, w.customer, aRideRequest({ voucherCode: 'xmas50 ' }));
    if (ok) await expect(attempt).resolves.toMatchObject({ details: { discount: 22_650, total: 23_000 } });
    else await expect(attempt).rejects.toMatchObject({ code: 'VOUCHER_EXPIRED' });
  });
});
