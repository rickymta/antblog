/**
 * TEST-049 · Loop testing — vòng cộng tiền món của đơn đồ ăn (SPEC R-FOOD-02…04).
 * Lệnh: npx vitest run articles/loop-testing
 */
import request from 'supertest';
import { beforeAll, beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { DomainError } from '../../src/domain/errors';
import { itemsSubtotal, priceFood } from '../../src/domain/food';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, createPgliteSystem, type LabSystem } from '../../src/infra/compose';
import { expectedSubtotal, lines, LOOP_COUNTS, maxLines } from './fixtures';
import { capOffByOne, resetInsideLoop, silentTruncation, skipFirstLine, type SubtotalFn } from './loop-variants';

const DELIVERY_FEE_2KM = 21_000; // xe máy 2,0 km ngoài cao điểm, không mưa: 12.000 + 4.500 × 2 (R-FOOD-04)
const food = (n: number) => ({ kind: 'food', cityCode: 'SG', distanceKm: 2, items: lines(n) });
const itRed = process.env.SHOW_RED ? it : it.fails; // ca ghi nhận defect: xanh khi defect còn, SHOW_RED=1 để thấy failure thật

describe('LT-A · vòng đơn ở tầng domain: itemsSubtotal / priceFood', () => {
  it.each(LOOP_COUNTS)('n = %i dòng ⇒ itemsSubtotal = oracle dạng đóng', (n) => {
    expect(itemsSubtotal(lines(n))).toBe(expectedSubtotal(n));
  });

  it('n = 0 (bỏ qua vòng) ⇒ priceFood ném SUBTOTAL_TOO_LOW, KHÔNG phải VALIDATION_ERROR', () => {
    expect(() => priceFood({ distanceKm: 2, items: [], peak: false, raining: false })).toThrowError(
      expect.objectContaining({ code: 'SUBTOTAL_TOO_LOW' }) as unknown as DomainError,
    );
  });

  it('n = 21 (max + 1) ⇒ tầng domain vẫn tính đủ 21 dòng: chặn trên KHÔNG nằm ở đây', () => {
    const price = priceFood({ distanceKm: 2, items: lines(21), peak: false, raining: false });
    expect(price.itemsSubtotal).toBe(840_000);
    expect(price.total).toBe(840_000 + DELIVERY_FEE_2KM);
  });
});

describe('LT-B · ma trận phát hiện: mỗi số lần lặp bắt được lỗi vòng nào', () => {
  const variants: Record<string, SubtotalFn> = {
    'L1 bỏ dòng đầu': skipFirstLine,
    'L2 khởi tạo trong vòng': resetInsideLoop,
    'L3 chặn trên lệch một': capOffByOne,
    'L4 lặng lẽ cắt bớt': silentTruncation,
    'src (bản thật)': itemsSubtotal,
  };

  it('bảng phát hiện khớp phân tích ở mục 4 của bài', () => {
    const matrix = Object.fromEntries(
      Object.entries(variants).map(([name, fn]) => [
        name,
        LOOP_COUNTS.map((n) => (fn(lines(n)) === expectedSubtotal(n) ? '·' : 'X')).join(' '),
      ]),
    );
    console.log(`n:                       ${LOOP_COUNTS.join(' ')}\n${Object.entries(matrix).map(([k, v]) => `${k.padEnd(25)}${v}`).join('\n')}`);
    //                                       0 1 2 5 19 20 21
    expect(matrix).toEqual({
      'L1 bỏ dòng đầu': '· X X X X X X',
      'L2 khởi tạo trong vòng': '· · X X X X X',
      'L3 chặn trên lệch một': '· · · · · X X',
      'L4 lặng lẽ cắt bớt': '· · · · · · X',
      'src (bản thật)': '· · · · · · ·',
    });
  });
});

describe('LT-C · vòng đơn nhìn từ API: POST /api/quotes (R-FOOD-02, R-API-02)', () => {
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;

  beforeEach(() => {
    system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
    app = createApp(system.deps, { logErrors: false });
  });

  it.each([0, 21])('n = %i dòng ⇒ 400 VALIDATION_ERROR, details chỉ ra trường items', async (n) => {
    const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send(food(n));
    expect(res.status).toBe(400);
    expect(res.body.error.code).toBe('VALIDATION_ERROR');
    expect(res.body.error.details.map((d: { path: string }) => d.path)).toContain('items');
  });

  it.each([1, 2, 5, 19, 20])('n = %i dòng ⇒ 200, itemsSubtotal + phí giao 21.000 đ = total', async (n) => {
    const res = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send(food(n));
    expect(res.status).toBe(200);
    expect(res.body.quote).toMatchObject({
      itemsSubtotal: expectedSubtotal(n),
      deliveryFee: DELIVERY_FEE_2KM,
      total: expectedSubtotal(n) + DELIVERY_FEE_2KM,
    });
  });
});

describe('LT-D · biến cộng dồn ở số lần lặp lớn × giá trị trần: báo giá rồi đặt chuyến', () => {
  const clock = new FixedClock(vnTime(2026, 11, 4, 9, 0));
  let pg: LabSystem;

  beforeAll(async () => {
    pg = await createPgliteSystem({ clock });
  });
  beforeEach(async () => {
    await pg.reset();
  });

  async function quoteThenBook(system: LabSystem, n: number) {
    const app = createApp(system.deps, { logErrors: false });
    const body = { kind: 'food', cityCode: 'SG', distanceKm: 2, items: maxLines(n) };
    const quote = await request(app).post('/api/quotes').set('x-user', 'customer:C001').send(body);
    const trip = await request(app)
      .post('/api/trips')
      .set('x-user', 'customer:C001')
      .set('Idempotency-Key', `loop-max-${n}-lines`)
      .send({ quoteId: quote.body.quote?.quoteId });
    return { quote, trip };
  }

  it('20 dòng × 5.000.000 đ × 50 ⇒ báo giá hợp lệ, tổng 5.000.021.000 đ (kho bộ nhớ đặt được chuyến)', async () => {
    const { quote, trip } = await quoteThenBook(createMemorySystem({ clock }), 20);
    expect(quote.status).toBe(200);
    expect(quote.body.quote.total).toBe(5_000_021_000);
    expect(trip.status).toBe(201);
  });

  it('PostgreSQL: 8 dòng giá trần ⇒ tổng 2.000.021.000 đ, đặt chuyến 201', async () => {
    const { quote, trip } = await quoteThenBook(pg, 8);
    expect(quote.body.quote.total).toBe(2_000_021_000);
    expect(trip.status).toBe(201);
  });

  itRed('DEFECT · PostgreSQL: 9 dòng giá trần ⇒ báo giá 200 nhưng đặt chuyến phải 201 (thực tế 500)', async () => {
    const { quote, trip } = await quoteThenBook(pg, 9);
    expect(quote.status).toBe(200);
    expect(quote.body.quote.total).toBe(2_250_021_000);
    expect({ status: trip.status, body: trip.body }).toMatchObject({ status: 201 });
  });
});
