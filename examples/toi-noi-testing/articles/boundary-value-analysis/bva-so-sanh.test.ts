/**
 * TEST-032 · Bốn bộ ca BVA (thô/sau làm tròn × 2-value/3-value) trên ba bản cài đặt có lỗi cố ý (bien-loi.ts).
 * Câu hỏi: bộ nào bắt được lỗi nào? Ma trận kết quả được khẳng định ở cuối.
 */
import { describe, expect, it } from 'vitest';
import { normalizeDistanceKm } from '../../src/domain/fare';
import { normalizeWeightKg } from '../../src/domain/parcel';
import { khoiLuongBangKhong, quangDuongKiemTruocLamTron, quangDuongToFixed, type Accepts } from './bien-loi';
import {
  DISTANCE_RAW,
  DISTANCE_ROUNDED,
  WEIGHT_RAW,
  WEIGHT_ROUNDED,
  threeValueItems,
  toNumber,
  twoValueItems,
  type OrderedDomain,
} from './bva-model';

const accepts = (fn: () => unknown) => {
  try {
    fn();
    return true;
  } catch {
    return false;
  }
};

/** Giá trị trong bộ ca mà bản lỗi trả khác bản đúng (= ca đỏ). */
function redValues(items: (d: OrderedDomain) => number[], d: OrderedDomain, buggy: Accepts, correct: Accepts): number[] {
  return items(d)
    .map((s) => toNumber(d, s))
    .filter((v) => buggy(v) !== correct(v));
}

const distanceOk: Accepts = (v) => accepts(() => normalizeDistanceKm(v));
const weightOk: Accepts = (v) => accepts(() => normalizeWeightKg(v));

function matrix(raw: OrderedDomain, rounded: OrderedDomain, buggy: Accepts, correct: Accepts) {
  return {
    raw2: redValues(twoValueItems, raw, buggy, correct),
    raw3: redValues(threeValueItems, raw, buggy, correct),
    rounded2: redValues(twoValueItems, rounded, buggy, correct),
    rounded3: redValues(threeValueItems, rounded, buggy, correct),
  };
}

describe('Bộ ca nào bắt được lỗi nào?', () => {
  it('Lỗi B «kiểm trước khi làm tròn»: chỉ bộ trên giá trị THÔ bắt được', () => {
    expect(matrix(DISTANCE_RAW, DISTANCE_ROUNDED, quangDuongKiemTruocLamTron, distanceOk)).toEqual({
      raw2: [0.45, 100.04],
      raw3: [0.45, 0.46, 100.03, 100.04],
      rounded2: [],
      rounded3: [],
    });
  });

  it('Lỗi T «toFixed(1)»: chỉ bộ trên giá trị THÔ bắt được, qua đúng một ca 100,05', () => {
    expect(matrix(DISTANCE_RAW, DISTANCE_ROUNDED, quangDuongToFixed, distanceOk)).toEqual({
      raw2: [100.05],
      raw3: [100.05],
      rounded2: [],
      rounded3: [],
    });
  });

  it('Lỗi K «kg === 0»: chỉ 3-value trên giá trị SAU làm tròn bắt được, qua ca −0,1 kg', () => {
    expect(matrix(WEIGHT_RAW, WEIGHT_ROUNDED, khoiLuongBangKhong, weightOk)).toEqual({
      raw2: [],
      raw3: [],
      rounded2: [],
      rounded3: [-0.1],
    });
  });

  it('Bản đúng không có ca đỏ (kiểm chính hàm so sánh)', () => {
    expect(matrix(DISTANCE_RAW, DISTANCE_ROUNDED, distanceOk, distanceOk).raw3).toEqual([]);
  });
});
