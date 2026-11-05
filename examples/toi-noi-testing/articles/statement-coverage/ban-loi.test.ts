/**
 * TEST-043 · Cùng bộ ca chạy trên BẢN CÓ LỖI pricing-loi.ts.
 * TC-01…04 vẫn xanh và vẫn phủ 100% câu lệnh của bản lỗi; chỉ ca biên dữ liệu TC-05 lộ failure
 * (khai báo `it.fails` để `npm test` của lab vẫn xanh — vitest báo đỏ nếu ca này bỗng ĐẠT).
 * Đặt SHOW_RED=1 để chạy TC-05 như ca thường và thấy failure thật.
 */
import { describe, it } from 'vitest';
import { boPhuCauLenh, tc05BienKhoiLuong } from './bo-phu-cau-lenh';
import { normalizePriceRequest, priceRequest } from './pricing-loi';

const loi = { normalizePriceRequest, priceRequest };
const itRed = process.env.SHOW_RED === '1' ? it : it.fails;

describe('pricing-loi.ts (bản có lỗi)', () => {
  boPhuCauLenh(loi);
  itRed('TC-05 · bưu kiện 20,04 kg đi xe máy ⇒ phải hợp lệ (R-PARCEL-01/02)', () => tc05BienKhoiLuong(loi));
});
