/**
 * TEST-032 · Đếm coverage item của BVA 2-value và 3-value cho từng miền (dùng để lập bảng so sánh trong bài).
 */
import { describe, expect, it } from 'vitest';
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
  twoValueItems,
  type OrderedDomain,
} from './bva-model';

const show = (d: OrderedDomain, items: number[]) => items.map((s) => toNumber(d, s));

describe('Số coverage item 2-value vs 3-value', () => {
  it.each([
    { d: DISTANCE_RAW, two: [0.44, 0.45, 100.04, 100.05], three: [0.43, 0.44, 0.45, 0.46, 100.03, 100.04, 100.05, 100.06] },
    { d: DISTANCE_ROUNDED, two: [0.4, 0.5, 100, 100.1], three: [0.3, 0.4, 0.5, 0.6, 99.9, 100, 100.1, 100.2] },
    { d: WEIGHT_RAW, two: [0.04, 0.05, 30.04, 30.05], three: [0.03, 0.04, 0.05, 0.06, 30.03, 30.04, 30.05, 30.06] },
    { d: WEIGHT_ROUNDED, two: [0, 0.1, 30, 30.1], three: [-0.1, 0, 0.1, 0.2, 29.9, 30, 30.1, 30.2] },
    { d: BIKE_LIMIT, two: [20, 20.1], three: [19.9, 20, 20.1, 20.2] },
  ])('$d.name', ({ d, two, three }) => {
    expect(show(d, twoValueItems(d))).toEqual(two);
    expect(show(d, threeValueItems(d))).toEqual(three);
  });

  it('giờ đặt: 8 item (2-value) và 16 item (3-value)', () => {
    expect(twoValueItems(PEAK_TIME).map(fmtTime)).toEqual([
      '10:59:59', '11:00:00', '12:59:59', '13:00:00', '16:59:59', '17:00:00', '18:59:59', '19:00:00',
    ]);
    expect(threeValueItems(PEAK_TIME)).toHaveLength(16);
    expect(threeValueItems(PEAK_TIME).map(fmtTime).slice(0, 4)).toEqual(['10:59:58', '10:59:59', '11:00:00', '11:00:01']);
  });
});
