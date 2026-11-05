/**
 * TEST-047 · Ca BỔ SUNG để voucher.ts + quote.ts đạt 100% statement và 100% branch.
 * 10 ca MC/DC chỉ phủ quyết định V; các quyết định KHÁC trong hai tệp (chuẩn hoá mã, không có voucher,
 * kiểm vai trò) cần ca riêng. Thêm hai ca theo đặc tả mà coverage không đòi: trần tiền giảm, thứ tự lỗi.
 * Chạy: npx vitest run articles/mc-dc-testing --reporter=verbose
 */
import { describe, expect, it } from 'vitest';
import { createQuote, priceQuote, requireRole } from '../../src/app/quote';
import { AT, BASE, C001, RIDE, runCase, systemFor } from './voucher-cases';

const err = (code: string) => expect.objectContaining({ code });

describe('S · bổ sung statement/branch (R-VOUCHER-02, R-API-04)', () => {
  it('S1 · không gửi voucherCode ⇒ không giảm, tổng 45.300 đ (quote.ts:93 `?? null`, :111, voucher.ts:24 `?? \'\'`)', async () => {
    const deps = await systemFor({ id: 'S1', note: '', overrides: {}, expected: { error: '' } });
    const q = await priceQuote(deps, C001.id, { ...RIDE }, AT);
    expect({ voucherCode: q.voucherCode, discount: q.discount, total: q.total }).toEqual({ voucherCode: null, discount: 0, total: 46_000 });
  });

  it('S2 · mã chỉ có khoảng trắng ⇒ coi như không nhập (voucher.ts:25 nhánh `? null`)', async () => {
    const deps = await systemFor({ id: 'S2', note: '', overrides: {}, expected: { error: '' } });
    const q = await priceQuote(deps, C001.id, { ...RIDE, voucherCode: '   ' }, AT);
    expect(q.voucherCode).toBeNull();
  });

  it('S3 · " mcdc10 " ≡ MCDC10 (chuẩn hoá trước khi tra)', async () => {
    expect(await runCase({ id: 'S3', note: '', overrides: {}, typedCode: ' mcdc10 ', expected: { error: '' } })).toEqual({
      total: 41_000,
      discount: 4_530,
    });
  });

  it('S4 · createQuote của khách lưu báo giá có hạn (quote.ts:54–68)', async () => {
    const deps = await systemFor({ id: 'S4', note: '', overrides: {}, expected: { error: '' } });
    const stored = await createQuote(deps, C001, { ...RIDE, voucherCode: BASE.code });
    expect({ voucherCode: stored.details.voucherCode, total: stored.details.total }).toEqual({ voucherCode: 'MCDC10', total: 41_000 });
  });

  it('S5 · tài xế gọi báo giá ⇒ FORBIDDEN (quote.ts:48 nhánh khách)', async () => {
    const deps = await systemFor({ id: 'S5', note: '', overrides: {}, expected: { error: '' } });
    await expect(createQuote(deps, { role: 'driver', id: 'D101' }, { ...RIDE })).rejects.toThrow(err('FORBIDDEN'));
  });

  it('S6 · requireRole(…, "driver") với khách ⇒ thông điệp nhánh tài xế (quote.ts:49 vế còn lại của `?:`)', () => {
    expect(() => requireRole(C001, 'driver')).toThrow('Chỉ tài xế được dùng chức năng này.');
  });
});

describe('Đ · theo đặc tả, coverage KHÔNG đòi (R-VOUCHER-06, R-VOUCHER-08)', () => {
  it('Đ1 · trần tiền giảm: 50% × 45.300 = 22.650 > trần 5.000 ⇒ giảm 5.000 (Math.min không phải nhánh)', async () => {
    expect(await runCase({ id: 'Đ1', note: '', overrides: { percent: 50, maxDiscount: 5_000 }, expected: { error: '' } })).toEqual({
      total: 41_000, // 45.300 − 5.000 = 40.300 ⇒ 41.000
      discount: 5_000,
    });
  });

  it('Đ2 · vừa hết hạn, vừa sai dịch vụ, vừa thiếu tiền tối thiểu ⇒ chỉ báo VOUCHER_EXPIRED', async () => {
    const run = runCase({
      id: 'Đ2',
      note: '',
      overrides: { validTo: '2026-11-03', validFrom: '2026-11-01', kinds: ['food'], minAmount: 999_999 },
      expected: { error: '' },
    });
    expect(await run).toEqual({ error: 'VOUCHER_EXPIRED' });
  });
});
