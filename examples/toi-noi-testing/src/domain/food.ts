import { DomainError } from './errors';
import { calculateFare, normalizeDistanceKm, type FareBreakdown } from './fare';
import { formatDong, toDong, toMilli } from './money';
import { voucherDiscountMilli, type VoucherTerms } from './voucher';

export const MIN_ITEMS_SUBTOTAL = 30_000; // SPEC R-FOOD-03

export interface FoodItem {
  name: string;
  unitPrice: number; // đồng, bội số 1.000 (kiểm ở lớp nhập liệu — SPEC R-FOOD-02)
  quantity: number;
}

export interface FoodPrice {
  itemsSubtotal: number;
  itemsDiscount: number;
  deliveryFee: FareBreakdown; // công thức xe máy, không voucher
  total: number;
}

export function itemsSubtotal(items: readonly FoodItem[]): number {
  return items.reduce((sum, item) => sum + item.unitPrice * item.quantity, 0);
}

/** Tiền món tối thiểu 30.000 đ (SPEC R-FOOD-03). */
export function assertSubtotalEnough(subtotal: number): void {
  if (subtotal < MIN_ITEMS_SUBTOTAL) {
    throw new DomainError('SUBTOTAL_TOO_LOW', `Tiền món tối thiểu ${formatDong(MIN_ITEMS_SUBTOTAL)}.`);
  }
}

/** Giá đơn đồ ăn (SPEC R-FOOD-01, R-FOOD-04, R-FOOD-05). */
export function priceFood(input: {
  distanceKm: number;
  items: readonly FoodItem[];
  peak: boolean;
  raining: boolean;
  voucher?: VoucherTerms | null;
}): FoodPrice {
  normalizeDistanceKm(input.distanceKm); // quãng đường được kiểm trước tiền món (SPEC R-VALID-01)
  const subtotal = itemsSubtotal(input.items);
  assertSubtotalEnough(subtotal);

  const discount = input.voucher ? toDong(voucherDiscountMilli(input.voucher, toMilli(subtotal))) : 0;
  const deliveryFee = calculateFare({
    vehicle: 'bike',
    distanceKm: input.distanceKm,
    peak: input.peak,
    raining: input.raining,
  });

  return {
    itemsSubtotal: subtotal,
    itemsDiscount: discount,
    deliveryFee,
    total: subtotal - discount + deliveryFee.total,
  };
}
