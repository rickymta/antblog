/**
 * TCD-06 · Bản chép lớp kiểm tra yêu cầu báo giá (thứ tự R-VALID-01) có HAI lỗi cố ý. Không sửa src/.
 *  - Lỗi Q: quên cận dưới quãng đường (chỉ kiểm > 100 km).
 *  - Lỗi X: quên quy tắc R-PARCEL-02 (xe máy chở được trên 20 kg).
 */
import { quoteSchema } from '../../src/api/schemas';
import { roundToTenth } from '../../src/domain/money';
import { normalizeWeightKg } from '../../src/domain/parcel';
import { isCityCode } from '../../src/domain/types';
import { toBody, type QuoteCase } from './mo-hinh';

export function buggyOutcome(c: QuoteCase): { status: number; code?: string } {
  const parsed = quoteSchema.safeParse(toBody(c));
  if (!parsed.success) return { status: 400, code: 'VALIDATION_ERROR' };
  const req = parsed.data;
  if (!isCityCode(req.cityCode)) return { status: 422, code: 'CITY_NOT_SUPPORTED' };
  if (roundToTenth(req.distanceKm) > 100) return { status: 422, code: 'DISTANCE_OUT_OF_RANGE' }; // lỗi Q
  if (req.kind === 'parcel') {
    try {
      normalizeWeightKg(req.parcelWeightKg);
    } catch {
      return { status: 422, code: 'WEIGHT_OUT_OF_RANGE' };
    }
    // lỗi X: không có assertVehicleFits
  }
  return { status: 200 };
}
