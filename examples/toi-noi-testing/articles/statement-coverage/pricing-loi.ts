/**
 * TEST-043 · BẢN CÓ LỖI của src/domain/pricing.ts — chỉ để minh hoạ, KHÔNG phải mã của lab.
 * Khác bản gốc đúng MỘT dòng (case 'parcel'): kiểm loại xe bằng khối lượng khách gửi (chưa làm tròn
 * 0,1 kg) thay vì khối lượng đã chuẩn hoá. Câu lệnh đó chạy với MỌI đơn bưu kiện, nhưng chỉ sai khi
 * khối lượng thô nằm trong (20; 20,05) kg — một defect phụ thuộc dữ liệu.
 */
import { DomainError } from '../../src/domain/errors';
import { calculateFare, normalizeDistanceKm, type FareBreakdown } from '../../src/domain/fare';
import { assertSubtotalEnough, itemsSubtotal, priceFood, type FoodItem } from '../../src/domain/food';
import { assertVehicleFits, normalizeWeightKg, parcelSurcharge } from '../../src/domain/parcel';
import { isCityCode, type CityCode, type TripKind, type Vehicle } from '../../src/domain/types';
import type { VoucherTerms } from '../../src/domain/voucher';

/** Phiên bản bộ quy tắc giá; lưu vào báo giá để biết giá được tính theo luật nào (SPEC R-QUOTE-02). */
export const PRICING_VERSION = 'fare-2026.1';

/** Yêu cầu báo giá đã qua kiểm tra định dạng (zod) nhưng chưa qua quy tắc nghiệp vụ. */
export type PriceRequest =
  | { kind: 'ride'; cityCode: string; vehicle: Vehicle; distanceKm: number }
  | { kind: 'parcel'; cityCode: string; vehicle: Vehicle; distanceKm: number; parcelWeightKg: number }
  | { kind: 'food'; cityCode: string; distanceKm: number; items: FoodItem[] };

/** Yêu cầu đã chuẩn hoá: quãng đường/khối lượng đã làm tròn, mọi quy tắc đầu vào đã đạt. */
export interface NormalizedRequest {
  kind: TripKind;
  cityCode: CityCode;
  vehicle: Vehicle;
  distanceKm: number;
  parcelWeightKg: number | null;
  items: FoodItem[] | null;
}

export interface PriceResult {
  fare: FareBreakdown; // với đồ ăn: bảng chi tiết của phí giao
  itemsSubtotal: number | null;
  itemsDiscount: number | null;
  deliveryFee: number | null;
  discount: number;
  total: number;
}

/**
 * Kiểm quy tắc đầu vào theo đúng thứ tự SPEC R-VALID-01:
 * thành phố → quãng đường → khối lượng → loại xe theo khối lượng → tiền món.
 */
export function normalizePriceRequest(request: PriceRequest): NormalizedRequest {
  if (!isCityCode(request.cityCode)) {
    throw new DomainError('CITY_NOT_SUPPORTED', `Tới Nơi chưa phục vụ thành phố "${request.cityCode}".`);
  }
  const distanceKm = normalizeDistanceKm(request.distanceKm);

  switch (request.kind) {
    case 'ride':
      return { kind: 'ride', cityCode: request.cityCode, vehicle: request.vehicle, distanceKm, parcelWeightKg: null, items: null };
    case 'parcel': {
      const weight = normalizeWeightKg(request.parcelWeightKg);
      assertVehicleFits(request.parcelWeightKg, request.vehicle); // DEFECT: dùng khối lượng THÔ, chưa làm tròn
      return { kind: 'parcel', cityCode: request.cityCode, vehicle: request.vehicle, distanceKm, parcelWeightKg: weight, items: null };
    }
    case 'food':
      assertSubtotalEnough(itemsSubtotal(request.items));
      return { kind: 'food', cityCode: request.cityCode, vehicle: 'bike', distanceKm, parcelWeightKg: null, items: request.items };
  }
}

/** Tính giá cho yêu cầu đã chuẩn hoá (voucher đã qua kiểm thời hạn/phạm vi). */
export function priceRequest(
  request: NormalizedRequest,
  conditions: { peak: boolean; raining: boolean; voucher?: VoucherTerms | null },
): PriceResult {
  if (request.kind === 'food') {
    const food = priceFood({ distanceKm: request.distanceKm, items: request.items ?? [], ...conditions });
    return {
      fare: food.deliveryFee,
      itemsSubtotal: food.itemsSubtotal,
      itemsDiscount: food.itemsDiscount,
      deliveryFee: food.deliveryFee.total,
      discount: food.itemsDiscount,
      total: food.total,
    };
  }
  const fare = calculateFare({
    vehicle: request.vehicle,
    distanceKm: request.distanceKm,
    parcelSurcharge: request.parcelWeightKg === null ? 0 : parcelSurcharge(request.parcelWeightKg),
    ...conditions,
  });
  return { fare, itemsSubtotal: null, itemsDiscount: null, deliveryFee: null, discount: fare.discount, total: fare.total };
}
