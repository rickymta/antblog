/** Kiểu dùng chung của miền nghiệp vụ Tới Nơi (SPEC R-GEN). */

export const CITY_CODES = ['SG', 'HN', 'DN', 'HP', 'CT', 'NT'] as const;
export type CityCode = (typeof CITY_CODES)[number];

export const TRIP_KINDS = ['ride', 'parcel', 'food'] as const;
export type TripKind = (typeof TRIP_KINDS)[number];

export const VEHICLES = ['bike', 'car'] as const;
export type Vehicle = (typeof VEHICLES)[number];

export function isCityCode(value: string): value is CityCode {
  return (CITY_CODES as readonly string[]).includes(value);
}
