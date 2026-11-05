/**
 * TEST-043 · Bộ ca phủ câu lệnh chạy trên mã THẬT của lab (src/domain/pricing.ts).
 * Đo từng bước bằng bộ lọc tên ca (-t), xem README.md.
 */
import { describe, it } from 'vitest';
import { normalizePriceRequest, priceRequest } from '../../src/domain/pricing';
import { boPhuCauLenh, tc05BienKhoiLuong } from './bo-phu-cau-lenh';

const thuc = { normalizePriceRequest, priceRequest };

describe('pricing.ts (mã lab)', () => {
  boPhuCauLenh(thuc);
  it('TC-05 · bưu kiện 20,04 kg đi xe máy ⇒ hợp lệ, khối lượng 20,0 kg (R-PARCEL-01/02)', () => tc05BienKhoiLuong(thuc));
});
