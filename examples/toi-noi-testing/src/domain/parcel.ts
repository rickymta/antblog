import { DomainError } from './errors';
import { roundToTenth } from './money';
import type { Vehicle } from './types';

export const MAX_PARCEL_KG = 30;
export const BIKE_MAX_KG = 20; // > 20 kg bắt buộc ô tô
export const FREE_KG = 5;
export const SURCHARGE_PER_STARTED_KG = 2_000;

/** Làm tròn 0,1 kg half-up rồi kiểm 0 < w ≤ 30 (SPEC R-PARCEL-01). */
export function normalizeWeightKg(raw: number): number {
  const kg = roundToTenth(raw);
  if (!Number.isFinite(kg) || kg <= 0 || kg > MAX_PARCEL_KG) {
    throw new DomainError('WEIGHT_OUT_OF_RANGE', `Khối lượng phải lớn hơn 0 và tối đa ${MAX_PARCEL_KG} kg.`);
  }
  return kg;
}

/** w > 20 kg phải đi ô tô (SPEC R-PARCEL-02). */
export function assertVehicleFits(weightKg: number, vehicle: Vehicle): void {
  if (weightKg > BIKE_MAX_KG && vehicle !== 'car') {
    throw new DomainError('VEHICLE_TOO_SMALL', `Bưu kiện trên ${BIKE_MAX_KG} kg phải giao bằng ô tô.`);
  }
}

/**
 * Phụ phí 2.000 đ cho mỗi kg bắt đầu vượt 5 kg (SPEC R-PARCEL-03).
 * Tính trên số phần mười kg (nguyên) để 6,1 − 5 không thành 1,0999…
 */
export function parcelSurcharge(weightKg: number): number {
  const extraTenths = Math.round(weightKg * 10) - FREE_KG * 10;
  if (extraTenths <= 0) return 0;
  return Math.ceil(extraTenths / 10) * SURCHARGE_PER_STARTED_KG;
}
