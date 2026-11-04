import { DomainError } from '../domain/errors';
import type { FareBreakdown } from '../domain/fare';
import type { FoodItem } from '../domain/food';
import { normalizePriceRequest, PRICING_VERSION, priceRequest, type PriceRequest } from '../domain/pricing';
import { addMinutes, isPeakTime, vnDayRange } from '../domain/time';
import type { CityCode, TripKind, Vehicle } from '../domain/types';
import {
  assertUsageAvailable,
  assertVoucherEligible,
  normalizeVoucherCode,
  type Voucher,
} from '../domain/voucher';
import type { Actor, AppDeps } from './deps';

export type QuoteInput = PriceRequest & { voucherCode?: string | null };

/** Nội dung giá của một báo giá (SPEC R-QUOTE-02) — chỉ gồm dữ liệu JSON thuần. */
export interface QuoteDetails {
  kind: TripKind;
  cityCode: CityCode;
  vehicle: Vehicle;
  distanceKm: number;
  parcelWeightKg: number | null;
  items: FoodItem[] | null;
  peak: boolean;
  raining: boolean;
  voucherCode: string | null;
  fare: FareBreakdown;
  itemsSubtotal: number | null;
  itemsDiscount: number | null;
  deliveryFee: number | null;
  discount: number;
  total: number;
}

/** Báo giá đã lưu, có hạn dùng (SPEC R-QUOTE-01). */
export interface StoredQuote {
  id: string;
  customerId: string;
  pricingVersion: string;
  createdAt: Date;
  expiresAt: Date;
  usedByTripId: string | null;
  details: QuoteDetails;
}

export function requireRole(actor: Actor, role: Actor['role']): void {
  if (actor.role !== role) {
    throw new DomainError('FORBIDDEN', role === 'customer' ? 'Chỉ khách hàng được dùng chức năng này.' : 'Chỉ tài xế được dùng chức năng này.');
  }
}

/** Báo giá tại `clock.now()`, lưu lại với hạn dùng `quoteTtlMinutes` (SPEC R-QUOTE-01). */
export async function createQuote(deps: AppDeps, actor: Actor, input: QuoteInput): Promise<StoredQuote> {
  requireRole(actor, 'customer');
  const now = deps.clock.now();
  const details = await priceQuote(deps, actor.id, input, now);
  const stored: StoredQuote = {
    id: deps.ids.next(),
    customerId: actor.id,
    pricingVersion: PRICING_VERSION,
    createdAt: now,
    expiresAt: addMinutes(now, deps.config.quoteTtlMinutes),
    usedByTripId: null,
    details,
  };
  await deps.quotes.save(stored);
  return stored;
}

/**
 * Tính giá theo đúng thứ tự kiểm tra của SPEC R-VALID-01 và R-VOUCHER-08.
 * Hàm không ghi gì — tách riêng để test lõi định giá mà không cần kho báo giá.
 */
export async function priceQuote(deps: AppDeps, customerId: string, input: QuoteInput, at: Date): Promise<QuoteDetails> {
  const request = normalizePriceRequest(input);

  const voucher = await loadEligibleVoucher(deps, input.voucherCode, { kind: request.kind, cityCode: request.cityCode, at });
  const peak = isPeakTime(at);
  const raining = await deps.weather.isRaining(request.cityCode, at);
  const price = priceRequest(request, { peak, raining, voucher }); // có thể ném VOUCHER_MIN_AMOUNT
  if (voucher) await assertVoucherQuota(deps, customerId, voucher, at);

  return {
    kind: request.kind,
    cityCode: request.cityCode,
    vehicle: request.vehicle,
    distanceKm: request.distanceKm,
    parcelWeightKg: request.parcelWeightKg,
    items: request.items,
    peak,
    raining,
    voucherCode: voucher?.code ?? null,
    ...price,
  };
}

/** Lượt dùng voucher trong ngày giờ Việt Nam (SPEC R-VOUCHER-07). */
export async function assertVoucherQuota(deps: AppDeps, customerId: string, voucher: Voucher, at: Date): Promise<void> {
  const { start, end } = vnDayRange(at);
  const usedToday = await deps.trips.countVoucherUses({ customerId, voucherCode: voucher.code, from: start, to: end });
  assertUsageAvailable(voucher, usedToday);
}

async function loadEligibleVoucher(
  deps: AppDeps,
  rawCode: string | null | undefined,
  context: { kind: TripKind; cityCode: CityCode; at: Date },
): Promise<Voucher | null> {
  const code = normalizeVoucherCode(rawCode);
  if (code === null) return null;
  const voucher = await deps.vouchers.findByCode(code);
  if (!voucher) throw new DomainError('VOUCHER_NOT_FOUND', `Không tìm thấy voucher ${code}.`);
  assertVoucherEligible(voucher, context);
  return voucher;
}
