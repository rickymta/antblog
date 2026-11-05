/**
 * TEST-018 · Non-functional checks dựng TỪ test chức năng (CTFL v4.0.1 mục 2.2.2):
 * cùng một lời gọi API, thêm một ràng buộc «làm tốt tới mức nào».
 * Đây là kiểm tra sớm trong lab (một tiến trình, một máy) — KHÔNG phải performance test
 * và không dùng để kết luận R-NFR-02/03 đạt hay không trên hệ thống thật.
 */
import type { Server } from 'node:http';
import request from 'supertest';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';

const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
let server: Server;
beforeAll(() => {
  server = createApp(system.deps, { logErrors: false }).listen(0);
});
afterAll(() => {
  server.close();
});

const QUOTE = { kind: 'parcel', cityCode: 'SG', vehicle: 'car', distanceKm: 3, parcelWeightKg: 6.1 };

function percentile(sortedMs: number[], p: number): number {
  const idx = Math.min(sortedMs.length - 1, Math.ceil((p / 100) * sortedMs.length) - 1);
  return sortedMs[idx] ?? Number.NaN;
}

describe('Từ test chức năng FC-04 (6,1 kg ⇒ 62.000 đ) tới kiểm tra phi chức năng', () => {
  it('NF-01 time behaviour (R-NFR-03, kiểm sớm): 200 báo giá liên tiếp, mọi kết quả đúng và p95 < 500 ms', async () => {
    const durations: number[] = [];
    for (let i = 0; i < 200; i++) {
      const t0 = performance.now();
      const res = await request(server).post('/api/quotes').set('x-user', 'customer:C001').send(QUOTE);
      durations.push(performance.now() - t0);
      expect(res.status).toBe(200); // ràng buộc phi chức năng không thay cho kết quả chức năng
      expect(res.body.quote.total).toBe(62_000);
    }
    durations.sort((a, b) => a - b);
    const p50 = percentile(durations, 50);
    const p95 = percentile(durations, 95);
    const max = durations[durations.length - 1] ?? Number.NaN;
    console.log(`NF-01: n=200 p50=${p50.toFixed(1)} ms p95=${p95.toFixed(1)} ms max=${max.toFixed(1)} ms`);
    expect(p95).toBeLessThan(500);
  });

  it('NF-02 confidentiality (R-NFR-06, R-API-05): chuyến của C001 với C002 trả y hệt chuyến không tồn tại', async () => {
    const q = await request(server).post('/api/quotes').set('x-user', 'customer:C001').send(QUOTE).expect(200);
    const trip = await request(server)
      .post('/api/trips')
      .set('x-user', 'customer:C001')
      .set('Idempotency-Key', 'nf-02-dat-chuyen')
      .send({ quoteId: q.body.quote.quoteId })
      .expect(201);
    const other = await request(server).get(`/api/trips/${trip.body.trip.id}`).set('x-user', 'customer:C002');
    const missing = await request(server).get('/api/trips/khong-ton-tai').set('x-user', 'customer:C002');
    expect(other.status).toBe(404);
    expect(other.body).toEqual(missing.body); // không lộ việc chuyến có tồn tại
  });

  it('NF-03 resistance (R-NFR-06, R-API-01): thân yêu cầu > 10 KB ⇒ 413 PAYLOAD_TOO_LARGE, không xử lý', async () => {
    const res = await request(server)
      .post('/api/quotes')
      .set('x-user', 'customer:C001')
      .send({ ...QUOTE, voucherCode: 'X'.repeat(11 * 1024) });
    expect(res.status).toBe(413);
    expect(res.body.error.code).toBe('PAYLOAD_TOO_LARGE');
  });
});
