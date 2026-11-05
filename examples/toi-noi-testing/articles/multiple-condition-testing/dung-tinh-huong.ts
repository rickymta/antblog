/**
 * TEST-046 — dựng dữ liệu cho MỘT dòng bảng chân trị rồi gọi lệnh nhận chuyến.
 *
 * Chuyến thử: xe máy (bike) ở SG, do khách C001 đặt. Tài xế thử DX:
 * - cungThanhPho = F ⇒ DX ở HN; dungLoaiXe = F ⇒ DX chạy ô tô (car);
 * - khongOffline = F ⇒ DX được nạp với trạng thái offline;
 * - khongBusy = F ⇒ DX nhận trước một chuyến phụ ở thành phố/loại xe của chính DX (R-ACCEPT-05);
 * - dangSearching = F ⇒ tài xế DK (SG, bike) nhận chuyến thử trước.
 */
import type { Actor, AppDeps } from '../../src/app/deps';
import { createQuote } from '../../src/app/quote';
import { acceptTrip, createTrip } from '../../src/app/trips';
import { DomainError } from '../../src/domain/errors';
import { vnTime } from '../../src/domain/time';
import type { Trip } from '../../src/domain/trip';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { labSeed } from '../../src/infra/seed';
import type { To } from './bang-chan-tri';

export type LenhNhan = (deps: AppDeps, actor: Actor, tripId: string, etaMinutes: number) => Promise<Trip>;

const DX: Actor = { role: 'driver', id: 'DX' };
const DK: Actor = { role: 'driver', id: 'DK' };

export async function chayDong(to: To, lenhNhan: LenhNhan = acceptTrip): Promise<string> {
  const base = labSeed();
  const city = to.cungThanhPho ? 'SG' : 'HN';
  const vehicle = to.dungLoaiXe ? 'bike' : 'car';
  const seed = {
    ...base,
    vouchers: [],
    drivers: [
      { id: 'DX', fullName: 'Tài xế thử', cityCode: city, vehicle, status: to.khongOffline ? 'online' : 'offline' },
      { id: 'DK', fullName: 'Tài xế khác', cityCode: 'SG', vehicle: 'bike', status: 'online' },
    ] as typeof base.drivers,
  };
  const { deps } = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)), seed });

  const datChuyen = async (customer: string, cityCode: 'SG' | 'HN', v: 'bike' | 'car') => {
    const actor: Actor = { role: 'customer', id: customer };
    const quote = await createQuote(deps, actor, { kind: 'ride', cityCode, vehicle: v, distanceKm: 5 });
    return createTrip(deps, actor, { quoteId: quote.id });
  };

  if (!to.khongBusy) await acceptTrip(deps, DX, (await datChuyen('C002', city, vehicle)).id, 5);
  const chuyenThu = await datChuyen('C001', 'SG', 'bike');
  if (!to.dangSearching) await acceptTrip(deps, DK, chuyenThu.id, 5);

  try {
    const trip = await lenhNhan(deps, DX, chuyenThu.id, 5);
    return trip.driverId === 'DX' && trip.status === 'accepted' ? 'OK' : `SAI:${trip.status}`;
  } catch (e) {
    if (e instanceof DomainError) return e.code;
    throw e;
  }
}
