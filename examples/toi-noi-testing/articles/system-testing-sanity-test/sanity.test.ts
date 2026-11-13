import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { createApp } from '../../src/api/app';
import { normalizeVoucherCode } from '../../src/domain/voucher';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem } from '../../src/infra/compose';

// Fault fixture: phiên bản giả định trước sửa, cố ý thiếu trim.
// Chỉ dùng để minh họa khả năng phát hiện lỗi, không thay src/ dùng chung.
const beforeFix = (raw: string) => raw.toUpperCase();

describe('SYS-02 · sanity cho thay đổi chuẩn hóa voucher', () => {
  let app: ReturnType<typeof createApp>;
  const ride = { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 5 };
  beforeEach(() => {
    const system = createMemorySystem({ clock: new FixedClock(vnTime(2026, 11, 4, 9, 0)) });
    app = createApp(system.deps, { logErrors: false });
  });
  const quote = (voucherCode?: string) => request(app).post('/api/quotes')
    .set('x-user', 'customer:C001').send({ ...ride, ...(voucherCode === undefined ? {} : { voucherCode }) });

  it('SN-01 · R-VOUCHER-02: oracle bắt bản thiếu trim và chấp nhận hàm hiện tại', () => {
    const input = ' giam20 ';
    expect(beforeFix(input)).not.toBe('GIAM20');
    expect(normalizeVoucherCode(input)).toBe('GIAM20');
  });
  it('SN-02 · R-VOUCHER-02/06: đầu vào lỗi gốc qua API trả mã chuẩn và 28.000 đ', async () => {
    const res = await quote(' giam20 ').expect(200);
    expect(res.body.quote).toMatchObject({ voucherCode: 'GIAM20', discount: 6_900, total: 28_000 });
  });
  it('SN-03 · mã chuẩn đang dùng không bị ảnh hưởng', async () => {
    const res = await quote('GIAM20').expect(200);
    expect(res.body.quote).toMatchObject({ voucherCode: 'GIAM20', discount: 6_900, total: 28_000 });
  });
  it.each(['', '   ', undefined])('SN-04 · mã rỗng/không nhập (%s) vẫn báo giá không giảm', async (code) => {
    const res = await quote(code).expect(200);
    expect(res.body.quote).toMatchObject({ voucherCode: null, discount: 0, total: 35_000 });
  });
  it('SN-05 · R-VOUCHER-03: mã không tồn tại không biến thành không dùng mã', async () => {
    const res = await quote(' khongco ').expect(422);
    expect(res.body.error.code).toBe('VOUCHER_NOT_FOUND');
  });
  it('SN-06 · R-VOUCHER-07/R-QUOTE-04: mã chuẩn được giữ khi đặt và tính lượt dùng', async () => {
    const first = await quote(' giam20 ').expect(200);
    const trip = await request(app).post('/api/trips').set('x-user', 'customer:C001')
      .set('Idempotency-Key', 'sys02-sanity-001').send({ quoteId: first.body.quote.quoteId }).expect(201);
    expect(trip.body.trip).toMatchObject({ fare: 28_000, voucherCode: 'GIAM20' });
    const next = await quote('GIAM20').expect(422);
    expect(next.body.error.code).toBe('VOUCHER_USAGE_LIMIT');
  });
});
