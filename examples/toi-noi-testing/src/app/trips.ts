import { customerCancellationFee, type FeeReason } from '../domain/cancel';
import { DomainError } from '../domain/errors';
import { addMinutes } from '../domain/time';
import type { Driver, PaymentMethod, Trip } from '../domain/trip';
import { ACTIVE_STATUSES, InvalidTransitionError, nextStatus, type TripEvent } from '../domain/trip-state';
import type { Actor, AppDeps } from './deps';
import { chargeWithRetry } from './payment';
import { assertVoucherQuota, requireRole } from './quote';

export interface CreateTripInput {
  quoteId: string;
  paymentMethod?: PaymentMethod;
}

const tripNotFound = () => new DomainError('TRIP_NOT_FOUND', 'Không tìm thấy chuyến.');
const tripTaken = () => new DomainError('TRIP_ALREADY_TAKEN', 'Chuyến đã có tài xế khác nhận.');
const quoteUsed = () => new DomainError('QUOTE_ALREADY_USED', 'Báo giá này đã được dùng để đặt một chuyến.');

// ---------------------------------------------------------------- khách

/**
 * Tạo chuyến từ một báo giá còn hạn của chính khách, giữ nguyên giá của báo giá
 * (SPEC R-QUOTE-03, R-API-08); `requested` → `searching` ngay (R-STATE-02).
 */
export async function createTrip(deps: AppDeps, actor: Actor, input: CreateTripInput): Promise<Trip> {
  requireRole(actor, 'customer');
  const now = deps.clock.now();
  const quote = await deps.quotes.findById(input.quoteId);
  if (!quote || quote.customerId !== actor.id) {
    throw new DomainError('QUOTE_NOT_FOUND', 'Không tìm thấy báo giá.');
  }
  if (now.getTime() >= quote.expiresAt.getTime()) {
    throw new DomainError('QUOTE_EXPIRED', 'Báo giá đã hết hạn, vui lòng báo giá lại.');
  }
  if (quote.usedByTripId !== null) throw quoteUsed();
  const q = quote.details;
  if (q.voucherCode) {
    const voucher = await deps.vouchers.findByCode(q.voucherCode);
    if (voucher) await assertVoucherQuota(deps, actor.id, voucher, now); // lượt có thể đã bị chuyến khác dùng
  }

  const tripId = deps.ids.next();
  if (!(await deps.quotes.markUsed(quote.id, tripId))) throw quoteUsed();
  const paymentMethod = input.paymentMethod ?? 'cash';

  const trip: Trip = {
    id: tripId,
    quoteId: quote.id,
    pricingVersion: quote.pricingVersion,
    kind: q.kind,
    cityCode: q.cityCode,
    customerId: actor.id,
    driverId: null,
    vehicle: q.vehicle,
    status: nextStatus('requested', 'search'),
    distanceKm: q.distanceKm,
    parcelWeightKg: q.parcelWeightKg,
    items: q.items,
    itemsSubtotal: q.itemsSubtotal,
    deliveryFee: q.deliveryFee,
    peak: q.peak,
    raining: q.raining,
    voucherCode: q.voucherCode,
    discount: q.discount,
    fare: q.total,
    fareBreakdown: q.fare,
    paymentMethod,
    paymentStatus: paymentMethod === 'cash' ? 'cash' : 'pending',
    requestedAt: now,
    searchingSince: now,
    acceptedAt: null,
    etaMinutes: null,
    completedAt: null,
    cancelledAt: null,
    cancelledBy: null,
    cancelReason: null,
    cancellationFee: 0,
    updatedAt: now,
  };
  await deps.trips.create(trip, [
    { tripId: trip.id, status: 'requested', occurredAt: now },
    { tripId: trip.id, status: 'searching', occurredAt: now },
  ]);
  return trip;
}

/** Xem chuyến theo quy tắc chống IDOR (SPEC R-API-05). */
export async function getTrip(deps: AppDeps, actor: Actor, tripId: string): Promise<Trip> {
  const trip = await deps.trips.findById(tripId);
  if (!trip) throw tripNotFound();
  if (actor.role === 'customer') {
    if (trip.customerId !== actor.id) throw tripNotFound(); // cùng thông điệp: không lộ chuyến có tồn tại
    return trip;
  }
  if (trip.driverId === actor.id) return trip;
  const driver = await deps.drivers.findById(actor.id);
  if (driver && trip.status === 'searching' && isEligible(driver, trip)) return trip;
  throw tripNotFound();
}

export interface CustomerCancelResult {
  trip: Trip;
  cancellationFee: number;
  feeReason: FeeReason;
}

/** Khách huỷ (SPEC R-CANCEL, R-PAY-05). */
export async function cancelByCustomer(
  deps: AppDeps,
  actor: Actor,
  tripId: string,
  reason?: string,
): Promise<CustomerCancelResult> {
  requireRole(actor, 'customer');
  const trip = await getTrip(deps, actor, tripId);
  const now = deps.clock.now();
  const { fee, reason: feeReason } = customerCancellationFee({
    status: trip.status,
    vehicle: trip.vehicle,
    acceptedAt: trip.acceptedAt,
    etaMinutes: trip.etaMinutes,
    now,
  });

  let cancelled: Trip = {
    ...trip,
    status: nextStatus(trip.status, 'customer_cancel'),
    cancelledAt: now,
    cancelledBy: 'customer',
    cancelReason: reason ?? 'customer_request',
    cancellationFee: fee,
    paymentStatus: fee === 0 ? 'none' : trip.paymentMethod === 'cash' ? 'cash' : 'pending',
    updatedAt: now,
  };
  await saveTransition(deps, cancelled, trip, 'customer_cancel');
  if (trip.driverId && ACTIVE_STATUSES.includes(trip.status)) await deps.drivers.markOnline(trip.driverId);

  if (fee > 0 && trip.paymentMethod === 'vixanh') {
    cancelled = await collectPayment(deps, cancelled, fee, `trip:${trip.id}:cancel-fee`, 'Phí huỷ chuyến');
  }
  return { trip: cancelled, cancellationFee: fee, feeReason };
}

// ---------------------------------------------------------------- tài xế

/** Chuyến đang chờ mà tài xế nhận được (SPEC R-ACCEPT-06). */
export async function listAvailableTrips(deps: AppDeps, actor: Actor): Promise<Trip[]> {
  requireRole(actor, 'driver');
  const driver = await loadDriver(deps, actor);
  return deps.trips.listSearching({ cityCode: driver.cityCode, vehicle: driver.vehicle });
}

/** Tài xế nhận chuyến — đúng một người thắng (SPEC R-ACCEPT-01…07). */
export async function acceptTrip(deps: AppDeps, actor: Actor, tripId: string, etaMinutes: number): Promise<Trip> {
  requireRole(actor, 'driver');
  const trip = await deps.trips.findById(tripId);
  if (!trip) throw tripNotFound();
  // Gửi lại lệnh nhận cho chuyến mình đang giữ ⇒ trả lại đúng phân công cũ (SPEC R-ACCEPT-07).
  if (trip.driverId === actor.id && ACTIVE_STATUSES.includes(trip.status)) return trip;
  const driver = await loadDriver(deps, actor);

  assertDriverAvailable(driver);
  if (driver.cityCode !== trip.cityCode) {
    throw new DomainError('CITY_MISMATCH', 'Chuyến không thuộc thành phố của tài xế.');
  }
  if (driver.vehicle !== trip.vehicle) {
    throw new DomainError('VEHICLE_MISMATCH', 'Chuyến cần loại xe khác.');
  }
  assertAcceptable(trip);

  // Giữ tài xế trước (online → busy có điều kiện), rồi giành chuyến (searching → accepted có điều kiện).
  if (!(await deps.drivers.tryMarkBusy(driver.id))) {
    assertDriverAvailable((await deps.drivers.findById(driver.id)) ?? driver);
    throw new DomainError('DRIVER_BUSY', 'Tài xế đang có chuyến khác.');
  }
  const accepted = await deps.trips.acceptIfSearching({ tripId, driverId: driver.id, acceptedAt: deps.clock.now(), etaMinutes });
  if (!accepted) {
    await deps.drivers.markOnline(driver.id); // thua cuộc ⇒ trả tài xế về online
    const latest = await deps.trips.findById(tripId);
    if (latest) assertAcceptable(latest);
    throw tripTaken();
  }
  return accepted;
}

/** `/arrive`, `/pickup`, `/start`, `/complete` của tài xế được gán (SPEC R-API-10). */
export async function driverStep(
  deps: AppDeps,
  actor: Actor,
  tripId: string,
  event: Extract<TripEvent, 'arrive' | 'pickup' | 'start' | 'complete'>,
): Promise<Trip> {
  requireRole(actor, 'driver');
  const trip = await loadAssignedTrip(deps, actor, tripId);
  const now = deps.clock.now();
  let next: Trip = {
    ...trip,
    status: nextStatus(trip.status, event),
    completedAt: event === 'complete' ? now : trip.completedAt,
    updatedAt: now,
  };
  await saveTransition(deps, next, trip, event);

  if (event === 'complete') {
    await deps.drivers.markOnline(actor.id);
    if (trip.paymentMethod === 'vixanh') {
      next = await collectPayment(deps, next, next.fare, `trip:${trip.id}:fare`, 'Cước chuyến');
    }
  }
  return next;
}

/** Tài xế huỷ: chuyến quay lại `searching`, khách không mất phí (SPEC R-STATE-04, R-CANCEL-05). */
export async function cancelByDriver(deps: AppDeps, actor: Actor, tripId: string): Promise<Trip> {
  requireRole(actor, 'driver');
  const trip = await loadAssignedTrip(deps, actor, tripId);
  const now = deps.clock.now();
  const reopened: Trip = {
    ...trip,
    status: nextStatus(trip.status, 'driver_cancel'),
    driverId: null,
    acceptedAt: null,
    etaMinutes: null,
    searchingSince: now,
    updatedAt: now,
  };
  await saveTransition(deps, reopened, trip, 'driver_cancel');
  await deps.drivers.markOnline(actor.id);
  return reopened;
}

// ---------------------------------------------------------------- hệ thống

/** Huỷ chuyến tìm tài xế quá 5 phút, lý do `no_driver` (SPEC R-STATE-05). Trả số chuyến đã huỷ. */
export async function expireStaleSearches(deps: AppDeps): Promise<number> {
  const now = deps.clock.now();
  const cutoff = addMinutes(now, -deps.config.searchTimeoutMinutes);
  let expired = 0;
  for (const trip of await deps.trips.listSearchingSince(cutoff)) {
    const cancelled: Trip = {
      ...trip,
      status: nextStatus(trip.status, 'system_cancel'),
      cancelledAt: now,
      cancelledBy: 'system',
      cancelReason: 'no_driver',
      cancellationFee: 0,
      paymentStatus: 'none',
      updatedAt: now,
    };
    if (await deps.trips.updateIfStatus(cancelled, 'searching')) expired++;
  }
  return expired;
}

// ---------------------------------------------------------------- hàm phụ

function isEligible(driver: Driver, trip: Trip): boolean {
  return driver.cityCode === trip.cityCode && driver.vehicle === trip.vehicle;
}

function assertDriverAvailable(driver: Driver): void {
  if (driver.status === 'offline') throw new DomainError('DRIVER_OFFLINE', 'Tài xế đang ngoại tuyến.');
  if (driver.status === 'busy') throw new DomainError('DRIVER_BUSY', 'Tài xế đang có chuyến khác.');
}

/** Chuyến đã huỷ ⇒ INVALID_TRANSITION; đã có tài xế ⇒ TRIP_ALREADY_TAKEN (SPEC R-ACCEPT-02). */
function assertAcceptable(trip: Trip): void {
  if (trip.status === 'searching') return;
  if (trip.status === 'cancelled' || trip.status === 'requested') throw new InvalidTransitionError(trip.status, 'accept');
  throw tripTaken();
}

async function loadDriver(deps: AppDeps, actor: Actor): Promise<Driver> {
  const driver = await deps.drivers.findById(actor.id);
  if (!driver) throw new DomainError('UNAUTHENTICATED', 'Tài xế không tồn tại.');
  return driver;
}

async function loadAssignedTrip(deps: AppDeps, actor: Actor, tripId: string): Promise<Trip> {
  const trip = await deps.trips.findById(tripId);
  if (!trip || trip.driverId !== actor.id) throw tripNotFound();
  return trip;
}

/** Lưu chuyển trạng thái có điều kiện; ai đó đổi trước ⇒ INVALID_TRANSITION theo trạng thái mới nhất. */
async function saveTransition(deps: AppDeps, next: Trip, previous: Trip, event: TripEvent): Promise<void> {
  if (await deps.trips.updateIfStatus(next, previous.status)) return;
  const latest = await deps.trips.findById(previous.id);
  throw new InvalidTransitionError(latest?.status ?? previous.status, event);
}

/** Thu tiền qua VíXanh rồi ghi `paymentStatus`; thất bại không chặn nghiệp vụ (SPEC R-PAY-06). */
async function collectPayment(deps: AppDeps, trip: Trip, amount: number, idempotencyKey: string, description: string): Promise<Trip> {
  const outcome = await chargeWithRetry(
    deps.payments,
    { idempotencyKey, customerId: trip.customerId, amount, description },
    deps.config.payment,
  );
  const updated: Trip = { ...trip, paymentStatus: outcome.status, updatedAt: deps.clock.now() };
  await deps.trips.updateIfStatus(updated, trip.status);
  return updated;
}
