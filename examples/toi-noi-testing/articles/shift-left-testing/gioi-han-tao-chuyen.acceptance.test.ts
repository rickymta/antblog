/**
 * TEST-030 · Shift-left Testing — tiêu chí chấp nhận (AC) của story «Giới hạn tạo chuyến» (SPEC R-API-13)
 * chốt trong buổi refinement, viết thành test TRƯỚC khi code (ATDD). Chạy ở mức API bằng supertest,
 * đồng hồ giả FixedClock để điều khiển cửa sổ 60 giây.
 *
 * Chạy: npx vitest run articles/shift-left-testing
 */
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';

const C001 = 'customer:C001';
const C002 = 'customer:C002';
const ride = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4 };
const T0 = vnTime(2026, 11, 4, 9, 0);

let clock: FixedClock;
let app: ReturnType<typeof createApp>;
let key = 0;

beforeEach(() => {
  clock = new FixedClock(T0);
  app = createApp(createMemorySystem({ clock }).deps, { logErrors: false });
});

const at = (seconds: number) => clock.set(new Date(T0.getTime() + seconds * 1000));
async function quoteId(user = C001): Promise<string> {
  const res = await request(app).post('/api/quotes').set('x-user', user).send(ride).expect(200);
  return res.body.quote.quoteId as string;
}
function createTrip(body: object, user = C001, idemKey = `dat-chuyen-${String(++key).padStart(4, "0")}`) {
  return request(app).post('/api/trips').set('x-user', user).set('Idempotency-Key', idemKey).send(body);
}
async function validTrip(user = C001) {
  return createTrip({ quoteId: await quoteId(user) }, user);
}

describe('Story: Giới hạn tạo chuyến 5 yêu cầu / 60 giây trượt (R-API-13)', () => {
  it('AC-1 · Given khách đã tạo 5 chuyến trong 60 giây, When gửi yêu cầu thứ 6, Then 429 RATE_LIMITED kèm Retry-After', async () => {
    for (let i = 0; i < 5; i++) expect((await validTrip()).status).toBe(201);
    const sixth = await validTrip();
    expect(sixth.status).toBe(429);
    expect(sixth.body.error.code).toBe('RATE_LIMITED');
    expect(sixth.headers['retry-after']).toBe('60');
  });

  it('AC-2 · Given 5 yêu cầu bị từ chối vì dữ liệu sai (400, 422), When gửi yêu cầu hợp lệ, Then vẫn 429 — yêu cầu lỗi cũng bị đếm', async () => {
    for (let i = 0; i < 3; i++) expect((await createTrip({})).status).toBe(400);
    for (let i = 0; i < 2; i++) expect((await createTrip({ quoteId: 'khong-co' })).status).toBe(422);
    expect((await validTrip()).status).toBe(429);
  });

  it('AC-3 · Given 1 yêu cầu tạo chuyến + 4 lần gửi lại cùng Idempotency-Key, When gửi yêu cầu mới, Then 429', async () => {
    const body = { quoteId: await quoteId() };
    for (let i = 0; i < 5; i++) expect((await createTrip(body, C001, 'cung-mot-khoa')).status).toBe(201);
    expect((await validTrip()).status).toBe(429);
  });

  it('AC-4 · Retry-After = số giây tới khi yêu cầu cũ nhất rời cửa sổ, làm tròn LÊN, tối thiểu 1', async () => {
    for (const s of [0, 10, 20, 30, 40]) { at(s); await createTrip({}); }
    at(40.5);
    expect((await createTrip({})).headers['retry-after']).toBe('20'); // 60 − 40,5 = 19,5 ⇒ 20
    at(59.9);
    expect((await createTrip({})).headers['retry-after']).toBe('1'); // 0,1 ⇒ 1
  });

  it('AC-5 · Yêu cầu bị 429 KHÔNG bị đếm; AC-6 · cửa sổ trượt: đúng 60 giây sau yêu cầu cũ nhất là gửi lại được', async () => {
    for (const s of [0, 10, 20, 30, 40]) { at(s); await createTrip({}); }
    for (const s of [41, 45, 50, 55]) { at(s); expect((await createTrip({})).status).toBe(429); }
    at(60); // yêu cầu lúc 0 s rời cửa sổ; còn 4 yêu cầu (10–40 s) — bốn lần 429 không tính
    expect((await createTrip({})).status).toBe(400);
    expect((await createTrip({})).status).toBe(429); // lại đủ 5
  });

  it('AC-7 · Giới hạn tính riêng từng khách: C001 hết lượt không ảnh hưởng C002', async () => {
    for (let i = 0; i < 5; i++) await createTrip({});
    expect((await createTrip({})).status).toBe(429);
    expect((await validTrip(C002)).status).toBe(201);
  });

  it('AC-8 · Yêu cầu chưa qua xác thực (401) hoặc sai vai trò (403) không bị đếm', async () => {
    for (let i = 0; i < 3; i++) expect((await request(app).post('/api/trips').send({})).status).toBe(401);
    for (let i = 0; i < 3; i++) expect((await createTrip({}, 'driver:D101')).status).toBe(403);
    for (let i = 0; i < 5; i++) expect((await createTrip({})).status).toBe(400);
    expect((await createTrip({})).status).toBe(429);
  });
});
