/**
 * TEST-017 · Functional completeness — bảng chi tiết cước R-FARE-10 có ĐỦ mọi bước không,
 * cho cả ba dịch vụ, qua POST /api/quotes (test level: system, qua API).
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import oldFare from './fare-ban-cu.json';
import { FARE_FIELD_BY_LABEL, missingFareFields, specFareLabels } from './fare-fields';

const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
const app = createApp(system.deps, { logErrors: false });

const BODIES = {
  ride: { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 },
  parcel: { kind: 'parcel', cityCode: 'SG', vehicle: 'car', distanceKm: 3, parcelWeightKg: 6.1 },
  food: { kind: 'food', cityCode: 'SG', distanceKm: 2.5, items: [{ name: 'Cơm tấm', unitPrice: 45000, quantity: 1 }] },
} as const;

describe('R-FARE-10 · functional completeness', () => {
  it('bảng ánh xạ của test phủ ĐÚNG danh sách 11 bước trong SPEC (không thiếu, không thừa)', () => {
    expect(Object.keys(FARE_FIELD_BY_LABEL).sort()).toEqual(specFareLabels().sort());
    expect(specFareLabels()).toHaveLength(11);
  });

  it.each(Object.entries(BODIES))('%s: quote.fare có đủ 11 bước, mỗi bước là số', async (_kind, body) => {
    const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send(body).expect(200);
    expect(missingFareFields(res.body.quote.fare)).toEqual([]);
  });

  it('bộ kiểm tra bắt được trường vắng trên một bảng chi tiết thiếu bước (dữ liệu giả định)', () => {
    expect(missingFareFields(oldFare)).toEqual(['phụ phí mưa', 'cước tối thiểu']);
  });
});
