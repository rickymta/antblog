import { describe, expect, it, vi } from 'vitest';
import type { Actor } from '../../src/app/deps';
import type { WeatherService } from '../../src/app/ports';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, cancelByDriver, createTrip, expireStaleSearches } from '../../src/app/trips';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';

const customer: Actor = { role: 'customer', id: 'C001' };
const driver: Actor = { role: 'driver', id: 'D101' };
const input = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 2 } as const;
const setup = (at = '2026-11-07T02:00:00Z') => {
  const clock = new FixedClock(new Date(at));
  return { clock, ...createMemorySystem({ clock }) };
};

describe('TEST-058 — clock và weather qua port', () => {
  it.each([
    ['2026-11-07T03:59:59Z', false, 21_000], ['2026-11-07T04:00:00Z', true, 27_000],
    ['2026-11-07T05:59:59Z', true, 27_000], ['2026-11-07T06:00:00Z', false, 21_000],
    ['2026-11-07T09:59:59Z', false, 21_000], ['2026-11-07T10:00:00Z', true, 27_000],
    ['2026-11-07T11:59:59Z', true, 27_000], ['2026-11-07T12:00:00Z', false, 21_000],
  ] as const)('R-GEN-02/FARE-04: %s → peak=%s, total=%s', async (at, peak, total) => {
    const { deps } = setup(at);
    const quote = await createQuote(deps, customer, input);
    expect(quote.details).toMatchObject({ peak, raining: false, total });
    expect(quote.createdAt.toISOString()).toBe(at.replace('Z', '.000Z'));
  });

  it('weather stub nhận đúng thành phố và thời điểm, tạo phụ phí mưa', async () => {
    const { deps, clock } = setup('2026-11-07T04:00:00Z');
    const isRaining = vi.fn<WeatherService['isRaining']>().mockResolvedValue(true);
    deps.weather = { isRaining };
    const quote = await createQuote(deps, customer, input);
    expect(isRaining).toHaveBeenCalledWith('SG', clock.now());
    expect(quote.details).toMatchObject({ peak: true, raining: true, total: 35_000 });
    expect(await deps.quotes.findById(quote.id)).toEqual(quote);
  });

  it('weather lỗi: không lưu một báo giá giả định trời khô', async () => {
    const { deps } = setup();
    const unavailable = new Error('Weather unavailable');
    deps.weather = { isRaining: async () => { throw unavailable; } };
    const save = vi.spyOn(deps.quotes, 'save');
    try {
      await expect(createQuote(deps, customer, input)).rejects.toBe(unavailable);
      expect(save).not.toHaveBeenCalled();
    } finally { save.mockRestore(); }
  });

  it.each([299_999, 300_000])('R-QUOTE-01: tuổi báo giá %s ms', async ageMs => {
    const { deps, clock } = setup();
    const quote = await createQuote(deps, customer, input);
    expect(quote.expiresAt.getTime() - quote.createdAt.getTime()).toBe(300_000);
    clock.advanceMs(ageMs);
    if (ageMs < 300_000) {
      expect(await createTrip(deps, customer, { quoteId: quote.id })).toMatchObject({ status: 'searching' });
    } else {
      await expect(createTrip(deps, customer, { quoteId: quote.id }))
        .rejects.toMatchObject({ code: 'QUOTE_EXPIRED' });
    }
  });

  it('R-STATE-05: chưa đủ 5 phút giữ searching; đúng 5 phút huỷ một lần', async () => {
    const { deps, clock, payments } = setup();
    const quote = await createQuote(deps, customer, input);
    const trip = await createTrip(deps, customer, { quoteId: quote.id, paymentMethod: 'vixanh' });
    clock.advanceMs(299_999);
    expect(await expireStaleSearches(deps)).toBe(0);
    expect(await deps.trips.findById(trip.id)).toMatchObject({ status: 'searching' });
    clock.advanceMs(1);
    expect(await expireStaleSearches(deps)).toBe(1);
    expect(await deps.trips.findById(trip.id)).toMatchObject({
      status: 'cancelled', cancelReason: 'no_driver', cancellationFee: 0, paymentStatus: 'none',
    });
    expect(await expireStaleSearches(deps)).toBe(0);
    expect(payments.calls).toHaveLength(0);
  });

  it('chuyến accepted không bị bộ quét hết hạn searching huỷ', async () => {
    const { deps, clock } = setup();
    const quote = await createQuote(deps, customer, input);
    const trip = await createTrip(deps, customer, { quoteId: quote.id });
    await acceptTrip(deps, driver, trip.id, 3);
    clock.advanceMinutes(5);
    expect(await expireStaleSearches(deps)).toBe(0);
    expect(await deps.trips.findById(trip.id)).toMatchObject({ status: 'accepted' });
  });

  it('R-STATE-04: tài xế huỷ đặt lại mốc tìm, không dùng tuổi chuyến ban đầu', async () => {
    const { deps, clock } = setup();
    const quote = await createQuote(deps, customer, input);
    const trip = await createTrip(deps, customer, { quoteId: quote.id });
    await acceptTrip(deps, driver, trip.id, 3);
    clock.advanceMinutes(4);
    await cancelByDriver(deps, driver, trip.id);
    clock.advanceMinutes(1);
    expect(await expireStaleSearches(deps)).toBe(0);
    clock.advanceMinutes(4);
    expect(await expireStaleSearches(deps)).toBe(1);
  });
});
