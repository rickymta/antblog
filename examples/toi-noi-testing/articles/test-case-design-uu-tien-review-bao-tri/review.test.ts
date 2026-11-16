/**
 * TCD-09 · Review bộ test case: phần máy kiểm được của checklist CL-TC trên bản nháp và bản sửa.
 */
import { describe, expect, it } from 'vitest';
import { BAN_NHAP, BAN_SUA } from './ban-nhap';
import { reviewMachineCheckable, specCodes } from './review-tu-dong';

describe('CL-TC — phần máy kiểm được', () => {
  it('mã R-… đọc từ SPEC.md: có R-PAY-04 và R-IDEM-05, không có R-PAY-07', () => {
    const codes = specCodes();
    expect(codes.has('R-PAY-04')).toBe(true);
    expect(codes.has('R-IDEM-05')).toBe(true);
    expect(codes.has('R-PAY-07')).toBe(false);
  });

  it('bản nháp 9 ca ⇒ 9 phát hiện, đúng một anti-pattern cài sẵn ở mỗi chỗ', () => {
    const found = reviewMachineCheckable(BAN_NHAP).map((f) => `${f.tc} ${f.item}`);
    expect(found).toEqual([
      'TC-PAY-003 CL-TC-02', // kết quả mong đợi mơ hồ
      'TC-PAY-004 CL-TC-01', // ba mục tiêu trong một ca
      'TC-IDEM-007 CL-TC-03', // phụ thuộc thứ tự ẩn
      'TC-PAY-006 CL-TC-04', // giờ không múi giờ, chờ thời gian thật
      'TC-PAY-005 CL-TC-05', // chép từ TC-PAY-002
      'TC-IDEM-002 CL-TC-06', // kiểm chi tiết cài đặt
      'TC-PAY-009 CL-TC-02', // «thanh toán nhanh»
      'TC-PAY-009 CL-TC-07', // không truy vết
      'TC-IDEM-006 CL-TC-02', // «không phải replay» — kết quả mong đợi thiếu
    ]);
  });

  it('bản sửa (8 ca, PAY-009 bị loại) ⇒ 0 phát hiện máy kiểm được', () => {
    expect(BAN_SUA).toHaveLength(8);
    expect(reviewMachineCheckable(BAN_SUA)).toEqual([]);
  });

  it('giới hạn: một ca sai đặc tả nhưng viết «đẹp» vẫn lọt máy — cần người review', () => {
    const wrongButTidy = { ...BAN_SUA[0]!, expected: ['HTTP 200, `paymentStatus` = `failed`, ví nhận đúng 2 lần gọi'] };
    expect(reviewMachineCheckable([wrongButTidy])).toEqual([]); // SPEC R-PAY-04: phải là 3 lần gọi
  });
});
