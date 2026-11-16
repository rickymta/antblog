/**
 * TCD-05 · Dữ liệu thời gian và môi trường như một phần của test data:
 *  - hạn báo giá R-QUOTE-01 (đúng bằng expiresAt là hết hạn) — kết quả mong đợi ghi bằng chuỗi ISO UTC tường minh;
 *  - thời tiết R-GEN-05/R-FARE-05 là dữ liệu tiền điều kiện của môi trường, không phải đầu vào của khách;
 *  - R-QUOTE-04: thời tiết đổi sau khi báo giá không làm đổi giá chuyến.
 */
import { describe, expect, it } from 'vitest';
import { createQuote } from '../../src/app/quote';
import { createTrip } from '../../src/app/trips';
import { aRideRequest, aWorld } from './test-data-builder';

describe('R-QUOTE-01 · hạn báo giá (mỗi ca một báo giá riêng)', () => {
  // Dữ liệu mong đợi viết tay từ SPEC: báo giá lúc 16/11 09:00:00 giờ VN = 02:00:00Z ⇒ hết hạn 02:05:00Z
  const CREATED = '2026-11-16T02:00:00.000Z';
  const EXPIRES = '2026-11-16T02:05:00.000Z';

  it('TC-QE-001 · tạo chuyến ở expiresAt − 1 ms ⇒ searching', async () => {
    const w = await aWorld().build();
    const q = await createQuote(w.deps, w.customer, aRideRequest());
    expect([q.createdAt.toISOString(), q.expiresAt.toISOString()]).toEqual([CREATED, EXPIRES]);
    w.clock.set(new Date(Date.parse(EXPIRES) - 1));
    await expect(createTrip(w.deps, w.customer, { quoteId: q.id })).resolves.toMatchObject({ status: 'searching' });
  });

  it('TC-QE-002 · tạo chuyến đúng expiresAt ⇒ QUOTE_EXPIRED', async () => {
    const w = await aWorld().build();
    const q = await createQuote(w.deps, w.customer, aRideRequest());
    w.clock.set(new Date(EXPIRES));
    await expect(createTrip(w.deps, w.customer, { quoteId: q.id })).rejects.toMatchObject({ code: 'QUOTE_EXPIRED' });
  });
});

describe('Thời tiết là dữ liệu tiền điều kiện của môi trường', () => {
  it.each([
    { id: 'TC-WX-001', rain: [] as const, raining: false, total: 46_000 }, // 45.300 ⇒ 46.000
    { id: 'TC-WX-002', rain: ['SG'] as const, raining: true, total: 54_000 }, // 45.300 + 8.000 = 53.300 ⇒ 54.000
    { id: 'TC-WX-003', rain: ['HN'] as const, raining: false, total: 46_000 }, // mưa ở thành phố khác
  ])('$id · đang mưa ở $rain ⇒ báo giá SG raining=$raining, $total đ', async ({ rain, raining, total }) => {
    const w = await aWorld().withRain(...rain).build();
    const q = await createQuote(w.deps, w.customer, aRideRequest());
    expect(q.details).toMatchObject({ raining, total });
  });

  it('TC-WX-004 · trời đổ mưa SAU khi báo giá ⇒ chuyến giữ giá và cờ của báo giá (R-QUOTE-04)', async () => {
    const w = await aWorld().build();
    const q = await createQuote(w.deps, w.customer, aRideRequest());
    w.system.weather.setRaining(['SG']);
    w.clock.advanceMinutes(1);
    const trip = await createTrip(w.deps, w.customer, { quoteId: q.id });
    expect(trip).toMatchObject({ fare: 46_000, raining: false });
  });
});
