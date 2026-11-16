/**
 * TCD-05 · Tính chất của chính builder: dữ liệu giả lập, tối thiểu, độc lập giữa các lần dựng,
 * và không dựng được tiền điều kiện mà nghiệp vụ không cho phép.
 */
import { describe, expect, it } from 'vitest';
import { createQuote } from '../../src/app/quote';
import { createTrip } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import type { Voucher } from '../../src/domain/voucher';
import { aCustomer, aRideRequest, aVoucher, aWorld } from './test-data-builder';

describe('Builder', () => {
  it('chỉ chứa dữ liệu giả lập mà test case khai báo — không có khách mẫu C001 của Phụ lục A', async () => {
    const w = await aWorld().withCustomer(aCustomer(2)).build();
    for (const id of ['C901', 'C902']) {
      const c = await w.deps.customers.findById(id);
      expect(c?.phone).toMatch(/^0900000\d{3}$/);
      expect(c?.fullName).toMatch(/^Khách Thử \d{2}$/);
    }
    expect(await w.deps.customers.findById('C001')).toBeNull();
    expect(await w.deps.vouchers.findByCode('GIAM20')).toBeNull();
  });

  it('hai lần build là hai hệ thống tách biệt: chuyến tạo ở thế giới A không tiêu lượt ở thế giới B', async () => {
    const a = await aWorld().build();
    const b = await aWorld().build();
    const q = await createQuote(a.deps, a.customer, aRideRequest({ voucherCode: 'THU20' }));
    await createTrip(a.deps, a.customer, { quoteId: q.id });
    await expect(createQuote(a.deps, a.customer, aRideRequest({ voucherCode: 'THU20' }))).rejects.toMatchObject({
      code: 'VOUCHER_USAGE_LIMIT',
    });
    await expect(createQuote(b.deps, b.customer, aRideRequest({ voucherCode: 'THU20' }))).resolves.toBeDefined();
  });

  it('object dùng chung cấp module bị một ca sửa thì ca sau thấy dữ liệu đã đổi; aVoucher() thì không', () => {
    const SHARED: Voucher = aVoucher(); // giả lập `const VOUCHER = {...}` ở đầu tệp test
    const caseA = (v: Voucher) => { v.perCustomerPerDay = 2; }; // ca A «tiện tay» sửa cho kịch bản của mình
    const caseB = (v: Voucher) => v.perCustomerPerDay;
    caseA(SHARED);
    expect(caseB(SHARED)).toBe(2); // rò dữ liệu sang ca B
    caseA(aVoucher());
    expect(caseB(aVoucher())).toBe(1); // mỗi lần gọi một object mới
  });

  it('tiền điều kiện trái nghiệp vụ không dựng được: hai chuyến cùng ngày với voucher 1 lượt/ngày', async () => {
    const b = aWorld()
      .withVoucherTrip(vnTime(2026, 11, 16, 8, 0, 0))
      .withVoucherTrip(vnTime(2026, 11, 16, 8, 30, 0));
    await expect(b.build()).rejects.toMatchObject({ code: 'VOUCHER_USAGE_LIMIT' });
  });
});
