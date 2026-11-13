/**
 * SYS-16 · Tương thích schema khi rollback. Mã hiện tại trong src/ đóng vai "bản cũ" (bản sẽ quay về khi rollback);
 * mỗi test áp một migration lên database PGlite riêng rồi xem bản cũ còn đọc/ghi được không.
 * Không sửa src/. PGlite một kết nối: không mô phỏng migration chạy song song với traffic thật hay khoá bảng.
 */
import request from 'supertest';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createPgliteSystem, type LabSystem } from '../../src/infra/compose';

describe('SYS-16 · migration và khả năng rollback mã ứng dụng', () => {
  let system: LabSystem;
  let app: ReturnType<typeof createApp>;
  let seq = 0;

  beforeEach(async () => {
    system = await createPgliteSystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
    await system.reset();
    app = createApp(system.deps, { logErrors: false });
  });
  afterEach(async () => { await system?.db?.close(); });

  const sql = (text: string) => system.db!.exec(text);

  async function book() {
    const quote = await request(app).post('/api/quotes').set('x-user', 'customer:C001')
      .send({ kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 }).expect(200);
    return request(app).post('/api/trips').set('x-user', 'customer:C001')
      .set('Idempotency-Key', `sys16-mig-${++seq}`)
      .send({ quoteId: quote.body.quote.quoteId, paymentMethod: 'cash' });
  }

  it('MG-01 · expand: thêm cột nullable — bản cũ vẫn ghi/đọc, dữ liệu bản mới không mất khi quay về', async () => {
    const before = await book();
    expect(before.status).toBe(201);
    const oldTripId: string = before.body.trip.id;

    await sql('ALTER TABLE trips ADD COLUMN pickup_note text');
    // Bản mới (giả lập bằng SQL) ghi giá trị cho cột mới.
    await system.db!.query('UPDATE trips SET pickup_note = $1 WHERE id = $2', ['Cổng số 2', oldTripId]);

    // Quay về bản cũ: tạo, chuyển trạng thái và đọc lại đều chạy.
    const created = await book();
    expect(created.status).toBe(201);
    const id: string = created.body.trip.id;
    await request(app).post(`/api/trips/${oldTripId}/accept`).set('x-user', 'driver:D101').send({ etaMinutes: 5 }).expect(200);
    const read = await request(app).get(`/api/trips/${oldTripId}`).set('x-user', 'customer:C001').expect(200);
    expect(read.body.trip).toMatchObject({ status: 'accepted', driverId: 'D101' });

    const { rows } = await system.db!.query<{ id: string; pickup_note: string | null }>(
      'SELECT id, pickup_note FROM trips ORDER BY requested_at, id');
    expect(rows).toEqual([
      { id: oldTripId, pickup_note: 'Cổng số 2' }, // bản cũ UPDATE cả dòng nhưng không chạm cột nó không biết
      { id, pickup_note: null },                    // dòng do bản cũ tạo: bản mới phải chịu được NULL
    ]);
  });

  it('MG-02 · expand sai: cột mới NOT NULL không default — bản cũ không tạo được chuyến (500, không lộ chi tiết)', async () => {
    await sql(`ALTER TABLE trips ADD COLUMN channel text;
               UPDATE trips SET channel = 'app';
               ALTER TABLE trips ALTER COLUMN channel SET NOT NULL;`);
    const res = await book();
    expect(res.status).toBe(500);
    expect(res.body).toEqual({ error: { code: 'INTERNAL_ERROR', message: 'Lỗi hệ thống, vui lòng thử lại sau.' } });
    const { rows } = await system.db!.query<{ n: number }>('SELECT count(*)::int AS n FROM trips');
    expect(rows).toEqual([{ n: 0 }]);
  });

  it('MG-03 · contract quá sớm: đổi tên cột — bản cũ ghi lỗi, còn đọc thì 200 nhưng âm thầm mất trường', async () => {
    const created = await book();
    const id: string = created.body.trip.id;
    const beforeRead = await request(app).get(`/api/trips/${id}`).set('x-user', 'customer:C001').expect(200);
    expect(beforeRead.body.trip).toHaveProperty('cancelReason', null);

    await sql('ALTER TABLE trips RENAME COLUMN cancel_reason TO cancellation_reason');

    const write = await book();
    expect(write.status).toBe(500);
    expect(write.body.error.code).toBe('INTERNAL_ERROR');

    const afterRead = await request(app).get(`/api/trips/${id}`).set('x-user', 'customer:C001').expect(200);
    expect(afterRead.body.trip).not.toHaveProperty('cancelReason');
  });
});
