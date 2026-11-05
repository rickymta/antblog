/**
 * TEST-031 · EP ở mức system qua API: POST /api/quotes (dịch vụ ride), 09:00 giờ VN, không mưa.
 * Bảng ca và phân vùng ở ep-model.ts — một bảng, hai việc: chạy test và đo coverage.
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { API_CASES } from './ep-model';

const app = createApp(createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) }).deps, { logErrors: false });

function quote(body: Record<string, unknown>) {
  return request(app).post('/api/quotes').set('x-user', 'customer:C001').send(body);
}

describe('Each Choice cho POST /api/quotes: mỗi phân vùng không hợp lệ một ca riêng', () => {
  it.each(API_CASES)('$id phủ $covers ⇒ HTTP $status', async (c) => {
    const res = await quote(c.body).expect(c.status);
    if (c.total !== undefined) expect(res.body.quote.total).toBe(c.total);
    if (c.code !== undefined) expect(res.body.error.code).toBe(c.code);
    if (c.field !== undefined) {
      // Lỗi định dạng phải chỉ đúng trường đang kiểm — không phải trường khác vô tình sai
      const paths = (res.body.error.details as { path: string }[]).map((d) => d.path);
      expect(paths).toEqual([c.field]);
    }
  });
});

describe('Cùng phân vùng ⇒ cùng cách xử lý (giả định của EP, kiểm vài đại diện khác)', () => {
  const base = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 };
  it.each([
    { partition: 'D1', label: 'distanceKm = 0', body: { ...base, distanceKm: 0 }, code: 'DISTANCE_OUT_OF_RANGE' },
    { partition: 'D1', label: 'distanceKm = -5', body: { ...base, distanceKm: -5 }, code: 'DISTANCE_OUT_OF_RANGE' },
    { partition: 'D3', label: 'distanceKm = 1.000.000', body: { ...base, distanceKm: 1_000_000 }, code: 'DISTANCE_OUT_OF_RANGE' },
    { partition: 'D4', label: 'distanceKm = null', body: { ...base, distanceKm: null }, code: 'VALIDATION_ERROR' },
    { partition: 'D4', label: 'thiếu distanceKm', body: { kind: 'ride', cityCode: 'SG', vehicle: 'bike' }, code: 'VALIDATION_ERROR' },
    // Chữ thường: cài đặt hiện xếp vào C2; SPEC R-GEN-03 không nói có chuẩn hoá hay không
    { partition: 'C2', label: 'cityCode = "sg"', body: { ...base, cityCode: 'sg' }, code: 'CITY_NOT_SUPPORTED' },
    { partition: 'C2', label: 'cityCode = "HCM"', body: { ...base, cityCode: 'HCM' }, code: 'CITY_NOT_SUPPORTED' },
    { partition: 'C3', label: 'cityCode 11 ký tự', body: { ...base, cityCode: 'THANHPHOHCM' }, code: 'VALIDATION_ERROR' },
    { partition: 'C3', label: 'cityCode = 79 (số)', body: { ...base, cityCode: 79 }, code: 'VALIDATION_ERROR' },
  ])('$partition: $label ⇒ $code', async ({ body, code }) => {
    const res = await quote(body);
    expect(res.body.error.code).toBe(code);
  });
});

describe('Defect masking: hai phân vùng không hợp lệ trong cùng một ca', () => {
  it('EP-X thành phố DL + quãng đường 0,2 km ⇒ chỉ thấy CITY_NOT_SUPPORTED (R-VALID-01), D1 KHÔNG được kiểm', async () => {
    const res = await quote({ kind: 'ride', cityCode: 'DL', vehicle: 'bike', distanceKm: 0.2 }).expect(422);
    expect(res.body.error.code).toBe('CITY_NOT_SUPPORTED');
  });
});
