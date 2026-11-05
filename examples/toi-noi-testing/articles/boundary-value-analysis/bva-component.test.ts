/**
 * TEST-032 · BVA 3-value ở mức component: normalizeDistanceKm, normalizeWeightKg, assertVehicleFits, isPeakTime.
 * Kết quả mong đợi suy từ phân vùng chứa giá trị (bva-model.ts), không chép từ đầu ra của code.
 */
import { describe, expect, it } from 'vitest';
import { normalizeDistanceKm } from '../../src/domain/fare';
import { assertVehicleFits, normalizeWeightKg } from '../../src/domain/parcel';
import { isPeakTime, vnTime } from '../../src/domain/time';
import {
  BIKE_LIMIT,
  DISTANCE_RAW,
  DISTANCE_ROUNDED,
  PEAK_TIME,
  WEIGHT_RAW,
  WEIGHT_ROUNDED,
  fmtTime,
  threeValueItems,
  toNumber,
  type OrderedDomain,
} from './bva-model';

function accepts(fn: () => unknown): boolean {
  try {
    fn();
    return true;
  } catch {
    return false;
  }
}

/** Giá trị thuộc phân vùng hợp lệ (phân vùng giữa) hay không. */
function inValidPartition(d: OrderedDomain, step: number): boolean {
  const valid = d.partitions[1]!;
  return step >= (valid.min ?? -Infinity) && step <= (valid.max ?? Infinity);
}

const rows = (d: OrderedDomain) => threeValueItems(d).map((s) => ({ value: toNumber(d, s), valid: inValidPartition(d, s) }));

describe('Quãng đường R-FARE-02 — 3-value trên giá trị thô và sau làm tròn', () => {
  it.each([...rows(DISTANCE_RAW), ...rows(DISTANCE_ROUNDED)])('$value km ⇒ hợp lệ: $valid', ({ value, valid }) => {
    expect(accepts(() => normalizeDistanceKm(value))).toBe(valid);
  });
});

describe('Khối lượng R-PARCEL-01 — 3-value trên giá trị thô và sau làm tròn', () => {
  it.each([...rows(WEIGHT_RAW), ...rows(WEIGHT_ROUNDED)])('$value kg ⇒ hợp lệ: $valid', ({ value, valid }) => {
    expect(accepts(() => normalizeWeightKg(value))).toBe(valid);
  });
});

describe('Ngưỡng xe máy R-PARCEL-02 — 3-value quanh 20,0 | 20,1 kg', () => {
  it.each(threeValueItems(BIKE_LIMIT).map((s) => ({ kg: toNumber(BIKE_LIMIT, s), bikeOk: s <= 200 })))(
    '$kg kg xe máy ⇒ được: $bikeOk',
    ({ kg, bikeOk }) => {
      expect(accepts(() => assertVehicleFits(normalizeWeightKg(kg), 'bike'))).toBe(bikeOk);
      expect(accepts(() => assertVehicleFits(normalizeWeightKg(kg), 'car'))).toBe(true);
    },
  );
});

describe('Giờ cao điểm R-FARE-04 — 3-value, độ chính xác 1 giây, giờ Việt Nam', () => {
  const peakLabels = new Set(['cao điểm trưa', 'cao điểm tối']);
  const cases = threeValueItems(PEAK_TIME).map((sec) => {
    const p = PEAK_TIME.partitions.find((x) => (x.min ?? -1) <= sec && sec <= (x.max ?? 1e9))!;
    return { time: fmtTime(sec), sec, peak: peakLabels.has(p.label) };
  });
  it.each(cases)('$time ⇒ cao điểm: $peak', ({ sec, peak }) => {
    const at = vnTime(2026, 11, 4, Math.floor(sec / 3600), Math.floor((sec % 3600) / 60), sec % 60);
    expect(isPeakTime(at)).toBe(peak);
  });
});
