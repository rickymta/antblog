/**
 * Bài TEST-025 «Technical Review: review API contract» — bằng chứng cho các phát hiện của buổi review.
 *
 * Technical review là static testing: phát hiện tìm ra bằng cách ĐỌC openapi-trips.v0.yaml và
 * pricing-quote.v2-draft.proto, đối chiếu SPEC và kiến trúc. Lab là bản cài đặt tham chiếu của SPEC, nên
 * reviewer dùng nó để kiểm nhanh «hợp đồng nói X, hệ thống đang làm Y» trước khi chốt mức nghiêm trọng —
 * đúng ý CTAL-TTA 5.1: kiểm work product nhất quán với sản phẩm đã có. Tên test = mã phát hiện → mã yêu cầu.
 * Lược đồ zod `tripResponse` dưới đây là hợp đồng ĐÃ SỬA của phản hồi 201, kiểm trên phản hồi thật.
 *
 * Chạy: npx vitest run articles/technical-review-api-contract --reporter=verbose
 */
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { z } from 'zod';
import { createApp } from '../../src/api/app';
import { calculateFare } from '../../src/domain/fare';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';

const C001 = 'customer:C001';
const C002 = 'customer:C002';
const ride = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 };

/** Hợp đồng đã sửa sau review (phần đủ để kiểm): tiền là số nguyên đồng, có lớp bọc { trip }. */
const dong = z.number().int().nonnegative();
const tripResponse = z.object({
  trip: z.object({
    id: z.string(),
    quoteId: z.string(),
    status: z.enum(['requested', 'searching', 'accepted', 'arriving', 'picked_up', 'in_transit', 'completed', 'cancelled']),
    fare: dong,
    discount: dong,
    paymentMethod: z.enum(['cash', 'vixanh']),
    paymentStatus: z.enum(['cash', 'pending', 'paid', 'failed', 'none']),
  }),
});
const errorBody = z.object({ error: z.object({ code: z.string(), message: z.string(), details: z.unknown().optional() }) });

describe('TEST-025 · Bằng chứng cho phát hiện của technical review', () => {
  let clock: FixedClock;
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;

  beforeEach(() => {
    clock = new FixedClock(vnTime(2026, 11, 4, 9, 0)); // 09:00 giờ VN — ngoài cao điểm
    system = createMemorySystem({ clock });
    app = createApp(system.deps, { logErrors: false });
  });

  async function quoteId(user = C001, body: object = ride): Promise<string> {
    const res = await request(app).post('/api/quotes').set('x-user', user).send(body).expect(200);
    return res.body.quote.quoteId;
  }
  const book = (body: object, key?: string, user = C001) => {
    const req = request(app).post('/api/trips').set('x-user', user);
    return key ? req.set('Idempotency-Key', key).send(body) : req.send(body);
  };

  it('TR-04/TR-05/TR-08 · R-API-08, R-GEN-01 · tạo chuyến trả 201, bọc trong { trip }, tiền là số nguyên đồng', async () => {
    const res = await book({ quoteId: await quoteId() }, 'tr-04-key-0001').expect(201);
    expect(tripResponse.safeParse(res.body).success).toBe(true);
  });

  it('TR-01 · R-IDEM-01 · thiếu Idempotency-Key ⇒ 400 IDEMPOTENCY_KEY_REQUIRED', async () => {
    const res = await book({ quoteId: await quoteId() }).expect(400);
    expect(res.body.error.code).toBe('IDEMPOTENCY_KEY_REQUIRED');
  });

  it('TR-02 · R-QUOTE-04, R-API-14 · giá client gửi kèm bị bỏ qua, chuyến lấy giá của báo giá', async () => {
    const q = await request(app).post('/api/quotes').set('x-user', C001).send(ride).expect(200);
    const res = await book({ quoteId: q.body.quote.quoteId, fare: 1_000 }, 'tr-02-key-0001').expect(201);
    expect(res.body.trip.fare).toBe(q.body.quote.total);
    expect(res.body.trip.fare).toBe(46_000); // 12.000 + 4.500 × 7,4 = 45.300 ⇒ làm tròn lên
  });

  it('TR-03 · R-PAY-01 · paymentMethod "card" không được hỗ trợ ⇒ 400 VALIDATION_ERROR có details', async () => {
    const res = await book({ quoteId: await quoteId(), paymentMethod: 'card' }, 'tr-03-key-0001').expect(400);
    expect(errorBody.safeParse(res.body).success).toBe(true);
    expect(res.body.error.code).toBe('VALIDATION_ERROR');
    expect(res.body.error.details[0].path).toBe('paymentMethod');
  });

  it('TR-06 · R-QUOTE-03, R-API-05 · báo giá của khách khác ⇒ 422 QUOTE_NOT_FOUND (không phải 404)', async () => {
    const res = await book({ quoteId: await quoteId(C002) }, 'tr-06-key-0001').expect(422);
    expect(res.body.error.code).toBe('QUOTE_NOT_FOUND');
  });

  it('TR-07 · R-QUOTE-03, R-IDEM-02 · gửi lại cùng khoá ⇒ 201 replay; cùng báo giá khoá khác ⇒ 409', async () => {
    const id = await quoteId();
    const first = await book({ quoteId: id }, 'tr-07-key-0001').expect(201);
    const replay = await book({ quoteId: id }, 'tr-07-key-0001').expect(201);
    expect(replay.headers['idempotent-replayed']).toBe('true');
    expect(replay.body.trip.id).toBe(first.body.trip.id);
    const other = await book({ quoteId: id }, 'tr-07-key-0002').expect(409);
    expect(other.body.error.code).toBe('QUOTE_ALREADY_USED');
  });

  it('TR-07 · R-API-13 · yêu cầu thứ 6 trong 60 giây ⇒ 429 RATE_LIMITED kèm Retry-After', async () => {
    for (let i = 1; i <= 5; i++) await book({ quoteId: 'khong-co' }, `tr-07-rate-000${i}`).expect(422);
    const res = await book({ quoteId: 'khong-co' }, 'tr-07-rate-0006').expect(429);
    expect(res.body.error.code).toBe('RATE_LIMITED');
    expect(Number(res.headers['retry-after'])).toBeGreaterThanOrEqual(1);
  });

  it('PQ-04 · R-FARE-02 · quãng đường đi qua float 32 bit làm đổi cước và đổi kết quả kiểm biên', () => {
    const gia = (km: number) => calculateFare({ vehicle: 'bike', distanceKm: km, peak: false, raining: false }).total;
    expect(Math.fround(7.35)).toBe(7.349999904632568);
    expect(gia(7.35)).toBe(46_000); // làm tròn thành 7,4 km
    expect(gia(Math.fround(7.35))).toBe(45_000); // thành 7,3 km — khách bị tính thiếu 1.000 đ
    expect(() => gia(0.45)).not.toThrow(); // SPEC: 0,45 ⇒ 0,5 km, hợp lệ
    expect(() => gia(Math.fround(0.45))).toThrow(/Quãng đường/); // float: 0,4499… ⇒ 0,4 km ⇒ DISTANCE_OUT_OF_RANGE
  });
});
