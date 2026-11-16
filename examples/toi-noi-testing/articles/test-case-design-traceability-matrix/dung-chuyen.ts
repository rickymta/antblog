/**
 * TCD-08 — dựng chuyến tới trạng thái cần thiết bằng use case thật (src/app), kho bộ nhớ, FixedClock.
 * Mốc chung: tạo chuyến 09:00:00 giờ VN 04/11/2026 (ngoài cao điểm), tài xế nhận 09:01:00, ETA 5 phút.
 */
import { createQuote } from '../../src/app/quote';
import { acceptTrip, createTrip, driverStep } from '../../src/app/trips';
import type { PriceRequest } from '../../src/domain/pricing';
import { vnTime } from '../../src/domain/time';
import type { Vehicle } from '../../src/domain/types';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';

export const T0 = vnTime(2026, 11, 4, 9, 0);
export const NHAN_LUC = vnTime(2026, 11, 4, 9, 1);
export const ETA = 5;
export const PHUT = 60_000;
export const KHACH = { role: 'customer' as const, id: 'C001' };
export const taiXe = (xe: Vehicle) => ({ role: 'driver' as const, id: xe === 'bike' ? 'D101' : 'D103' });

export type Moc = 'searching' | 'accepted' | 'arriving' | 'picked_up' | 'in_transit' | 'completed';

export async function dungChuyen(opts: {
  xe?: Vehicle;
  toi: Moc;
  dichVu?: 'ride' | 'food';
  thanhToan?: 'cash' | 'vixanh';
}) {
  const xe = opts.xe ?? 'bike';
  const clock = new FixedClock(T0);
  const system = createMemorySystem({ clock });
  const { deps } = system;
  const tx = taiXe(xe);
  const yeuCau: PriceRequest =
    opts.dichVu === 'food'
      ? { kind: 'food', cityCode: 'SG', distanceKm: 3, items: [{ name: 'Cơm tấm', unitPrice: 45_000, quantity: 1 }] }
      : { kind: 'ride', cityCode: 'SG', vehicle: xe, distanceKm: 5 };
  const quote = await createQuote(deps, KHACH, yeuCau);
  const trip = await createTrip(deps, KHACH, { quoteId: quote.id, paymentMethod: opts.thanhToan ?? 'cash' });

  const buoc: Moc[] = ['accepted', 'arriving', 'picked_up', 'in_transit', 'completed'];
  const den = buoc.indexOf(opts.toi as (typeof buoc)[number]);
  if (den >= 0) {
    clock.set(NHAN_LUC);
    await acceptTrip(deps, tx, trip.id, ETA);
    const suKien = ['arrive', 'pickup', 'start', 'complete'] as const;
    for (let i = 0; i < den; i++) {
      clock.advanceMs(10_000);
      await driverStep(deps, tx, trip.id, suKien[i]!);
    }
  }
  return { system, deps, clock, tripId: trip.id, taiXe: tx };
}
