/**
 * TEST-046 — bản cài đặt SAI (lỗi gài, không có trong src/) của lệnh nhận chuyến: kiểm loại xe
 * TRƯỚC thành phố, ngược R-ACCEPT-02. Phần còn lại chép theo cấu trúc src/app/trips.ts.
 */
import type { Actor, AppDeps } from '../../src/app/deps';
import { DomainError } from '../../src/domain/errors';
import type { Trip } from '../../src/domain/trip';
import { ACTIVE_STATUSES, InvalidTransitionError } from '../../src/domain/trip-state';

export async function acceptTripSaiThuTu(deps: AppDeps, actor: Actor, tripId: string, etaMinutes: number): Promise<Trip> {
  const trip = await deps.trips.findById(tripId);
  if (!trip) throw new DomainError('TRIP_NOT_FOUND', 'Không tìm thấy chuyến.');
  if (trip.driverId === actor.id && ACTIVE_STATUSES.includes(trip.status)) return trip;
  const driver = await deps.drivers.findById(actor.id);
  if (!driver) throw new DomainError('UNAUTHENTICATED', 'Tài xế không tồn tại.');

  if (driver.status === 'offline') throw new DomainError('DRIVER_OFFLINE', 'Tài xế đang ngoại tuyến.');
  if (driver.status === 'busy') throw new DomainError('DRIVER_BUSY', 'Tài xế đang có chuyến khác.');
  if (driver.vehicle !== trip.vehicle) throw new DomainError('VEHICLE_MISMATCH', 'Chuyến cần loại xe khác.'); // ← đảo thứ tự
  if (driver.cityCode !== trip.cityCode) throw new DomainError('CITY_MISMATCH', 'Chuyến không thuộc thành phố của tài xế.');
  if (trip.status === 'cancelled' || trip.status === 'requested') throw new InvalidTransitionError(trip.status, 'accept');
  if (trip.status !== 'searching') throw new DomainError('TRIP_ALREADY_TAKEN', 'Chuyến đã có tài xế khác nhận.');

  if (!(await deps.drivers.tryMarkBusy(driver.id))) throw new DomainError('DRIVER_BUSY', 'Tài xế đang có chuyến khác.');
  const accepted = await deps.trips.acceptIfSearching({ tripId, driverId: driver.id, acceptedAt: deps.clock.now(), etaMinutes });
  if (!accepted) {
    await deps.drivers.markOnline(driver.id);
    throw new DomainError('TRIP_ALREADY_TAKEN', 'Chuyến đã có tài xế khác nhận.');
  }
  return accepted;
}
