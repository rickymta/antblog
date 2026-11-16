/**
 * TCD-06 · Chạy bộ ca cuối cùng qua POST /api/quotes (supertest). 09:00 giờ VN 16/11/2026, không mưa.
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';
import { MASKED_CASE, POOL, expectedOutcome, greedyMinimize, toBody, type QuoteCase } from './mo-hinh';

const app = createApp(createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 16, 9, 0)) }).deps, { logErrors: false });
const quote = (body: Record<string, unknown>) => request(app).post('/api/quotes').set('x-user', 'customer:C001').send(body);
const paths = (body: { error: { details: { path: string }[] } }) => body.error.details.map((x) => x.path).sort();

async function check(c: QuoteCase) {
  const res = await quote(toBody(c));
  const exp = expectedOutcome(c);
  expect(res.status).toBe(exp.status);
  if (exp.status === 200) expect(res.body.quote.total).toBe(exp.total);
  else expect(res.body.error.code).toBe(exp.code);
  if (exp.status === 400) expect(paths(res.body)).toEqual([...exp.fields].sort());
}

describe('Bộ ca cuối (18 ca) qua API', () => {
  it.each(greedyMinimize(POOL))('$id · $note', check);
});

describe('Đối chiếu oracle với hệ thống trên cả kho 73 ca', () => {
  it('mọi ca trong kho cho đúng status, mã lỗi, tổng tiền, details', async () => {
    for (const c of POOL) await check(c);
  });
});

describe('Ca âm gộp nhiều lỗi', () => {
  it('MASK-01 xanh qua API nhưng chỉ cho thấy CITY_NOT_SUPPORTED', async () => {
    const res = await quote(toBody(MASKED_CASE)).expect(422);
    expect(res.body.error.code).toBe('CITY_NOT_SUPPORTED');
  });

  it('lỗi định dạng KHÔNG che nhau: details liệt kê đủ hai trường', async () => {
    const res = await quote({ kind: 'parcel', cityCode: '', vehicle: 'truck', distanceKm: 7.4, parcelWeightKg: 3 }).expect(400);
    expect(paths(res.body)).toEqual(['cityCode', 'vehicle']);
  });

  it('nhưng kind sai thì che mọi lỗi định dạng khác: details chỉ có kind', async () => {
    const res = await quote({ kind: 'taxi', cityCode: '', vehicle: 'truck', distanceKm: '7.4' }).expect(400);
    expect(paths(res.body)).toEqual(['kind']);
  });
});
