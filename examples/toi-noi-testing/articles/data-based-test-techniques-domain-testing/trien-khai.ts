/**
 * TEST-040 · Bản cài đặt được kiểm:
 * - `srcImpl`: mã THẬT của lab (`normalizePriceRequest` + `parcelSurcharge`).
 * - `makeImpl(cfg)`: bản chép lại logic phân vùng của src, có tham số để gài đúng loại defect mà domain
 *   testing nhắm tới (sai toán tử, sai hằng số, sai làm tròn, sai thứ tự). `makeImpl(REFERENCE)` phải khớp src.
 * Không sửa `src/`.
 */
import { DomainError } from '../../src/domain/errors';
import { parcelSurcharge } from '../../src/domain/parcel';
import { normalizePriceRequest } from '../../src/domain/pricing';
import type { Code, Outcome, Vehicle } from './mien-buu-kien';

export type Impl = (d: number, w: number, v: Vehicle) => Outcome;

export const srcImpl: Impl = (d, w, v) => {
  try {
    const req = normalizePriceRequest({ kind: 'parcel', cityCode: 'SG', vehicle: v, distanceKm: d, parcelWeightKg: w });
    return { code: 'OK', surcharge: parcelSurcharge(req.parcelWeightKg!) };
  } catch (error) {
    if (error instanceof DomainError) return { code: error.code as Code };
    throw error;
  }
};

export interface Cfg {
  round: 'half-up' | 'truncate';
  dMinStrict: boolean; // true ⇒ d > 0,5 (sai) thay vì d ≥ 0,5
  dMaxStrict: boolean; // true ⇒ d < 100
  wMinInclusive: boolean; // true ⇒ w ≥ 0 (sai) thay vì w > 0
  wMaxStrict: boolean; // true ⇒ w < 30
  bikeInclusive: boolean; // true ⇒ w ≥ 20 bắt ô tô (sai) thay vì w > 20
  bikeMax: number;
  bikeOnly: boolean; // false ⇒ áp giới hạn xe máy cho cả ô tô
  surchargeOnRaw: boolean; // true ⇒ phụ phí tính trên khối lượng chưa làm tròn
  absWeight: boolean; // true ⇒ «sửa dấu» khối lượng âm bằng Math.abs
  weightFirst: boolean; // true ⇒ kiểm khối lượng trước quãng đường (sai R-VALID-01)
}

export const REFERENCE: Cfg = {
  round: 'half-up', dMinStrict: false, dMaxStrict: false, wMinInclusive: false, wMaxStrict: false,
  bikeInclusive: false, bikeMax: 20, bikeOnly: true, surchargeOnRaw: false, absWeight: false, weightFirst: false,
};

export const MUTANTS: Record<string, { cfg: Partial<Cfg>; moTa: string }> = {
  M1: { cfg: { dMinStrict: true }, moTa: 'd > 0,5 thay vì d ≥ 0,5' },
  M2: { cfg: { dMaxStrict: true }, moTa: 'd < 100 thay vì d ≤ 100' },
  M3: { cfg: { wMinInclusive: true }, moTa: 'w ≥ 0 thay vì w > 0' },
  M4: { cfg: { wMaxStrict: true }, moTa: 'w < 30 thay vì w ≤ 30' },
  M5: { cfg: { bikeInclusive: true }, moTa: 'xe máy bị chặn từ w ≥ 20 (đúng: w > 20)' },
  M6: { cfg: { bikeMax: 25 }, moTa: 'hằng số giới hạn xe máy 25 thay vì 20' },
  M7: { cfg: { bikeOnly: false }, moTa: 'giới hạn 20 kg áp cho cả ô tô' },
  M8: { cfg: { round: 'truncate' }, moTa: 'cắt cụt thay vì làm tròn half-up' },
  M9: { cfg: { surchargeOnRaw: true }, moTa: 'phụ phí tính trên khối lượng chưa làm tròn' },
  M10: { cfg: { absWeight: true }, moTa: 'khối lượng âm bị «sửa» bằng Math.abs' },
  M11: { cfg: { weightFirst: true }, moTa: 'kiểm khối lượng trước quãng đường' },
};

export function makeImpl(overrides: Partial<Cfg> = {}): Impl {
  const cfg = { ...REFERENCE, ...overrides };
  const r = (x: number) => {
    const scaled = Number((x * 10).toPrecision(12));
    return (cfg.round === 'half-up' ? Math.round(scaled) : Math.trunc(scaled)) / 10;
  };
  return (rawD, rawW, v) => {
    const d = r(rawD);
    const w = r(cfg.absWeight ? Math.abs(rawW) : rawW);
    const distanceBad = (cfg.dMinStrict ? d <= 0.5 : d < 0.5) || (cfg.dMaxStrict ? d >= 100 : d > 100);
    const weightBad = (cfg.wMinInclusive ? w < 0 : w <= 0) || (cfg.wMaxStrict ? w >= 30 : w > 30);
    const checks: [boolean, Code][] = [[distanceBad, 'DISTANCE_OUT_OF_RANGE'], [weightBad, 'WEIGHT_OUT_OF_RANGE']];
    for (const [bad, code] of cfg.weightFirst ? checks.reverse() : checks) if (bad) return { code };
    const tooHeavy = cfg.bikeInclusive ? w >= cfg.bikeMax : w > cfg.bikeMax;
    if (tooHeavy && (v === 'bike' || !cfg.bikeOnly)) return { code: 'VEHICLE_TOO_SMALL' };
    const base = cfg.surchargeOnRaw ? rawW : w;
    const extraTenths = Math.round(base * 10) - 50;
    const extra = cfg.surchargeOnRaw ? base - 5 : extraTenths / 10;
    return { code: 'OK', surcharge: extra <= 0 ? 0 : Math.ceil(extra - 1e-9) * 2_000 };
  };
}
