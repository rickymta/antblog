import type { FareBreakdown } from './fare';
import type { FoodItem } from './food';
import type { TripStatus } from './trip-state';
import type { CityCode, TripKind, Vehicle } from './types';

export type PaymentMethod = 'cash' | 'vixanh';
/** SPEC R-PAY-02. */
export type PaymentStatus = 'cash' | 'pending' | 'paid' | 'failed' | 'none';
export type CancelledBy = 'customer' | 'system';
export type DriverStatus = 'offline' | 'online' | 'busy';

export interface Customer {
  id: string;
  phone: string;
  fullName: string;
}

export interface Driver {
  id: string;
  fullName: string;
  cityCode: CityCode;
  vehicle: Vehicle;
  status: DriverStatus;
}

/** Một chuyến — tương ứng một dòng bảng `trips`. Tiền tính bằng đồng. */
export interface Trip {
  id: string;
  quoteId: string;
  pricingVersion: string;
  kind: TripKind;
  cityCode: CityCode;
  customerId: string;
  driverId: string | null;
  vehicle: Vehicle;
  status: TripStatus;
  distanceKm: number;
  parcelWeightKg: number | null;
  items: FoodItem[] | null;
  itemsSubtotal: number | null;
  deliveryFee: number | null;
  peak: boolean;
  raining: boolean;
  voucherCode: string | null;
  discount: number;
  fare: number; // tổng khách phải trả = quote.total (không đổi hồi tố)
  fareBreakdown: FareBreakdown; // ảnh chụp bảng chi tiết của báo giá
  paymentMethod: PaymentMethod;
  paymentStatus: PaymentStatus;
  requestedAt: Date;
  searchingSince: Date | null;
  acceptedAt: Date | null;
  etaMinutes: number | null;
  completedAt: Date | null;
  cancelledAt: Date | null;
  cancelledBy: CancelledBy | null;
  cancelReason: string | null;
  cancellationFee: number;
  updatedAt: Date;
}

export interface TripEventRecord {
  tripId: string;
  status: TripStatus;
  occurredAt: Date;
}
