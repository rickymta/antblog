/**
 * TEST-039 · Completeness testing (kiểm TĨNH — không chạy hệ thống): với từng thực thể,
 * thao tác C/R/U/D nào không có chức năng nào trong ma trận CRUD thực hiện?
 * Mỗi ô trống là một ANOMALY cần điều tra (chưa chắc là defect) — CTAL-TA v4.0 mục 3.2.1.
 */
import { describe, expect, it } from 'vitest';
import { ENTITIES, matrixCells, missingOps, SPEC_MATRIX } from './crud-matrix';

describe('TEST-039 · completeness của ma trận CRUD theo SPEC', () => {
  const missing = missingOps(SPEC_MATRIX);

  it('ma trận có 10 chức năng × 6 thực thể, 48 ô thao tác', () => {
    expect(SPEC_MATRIX).toHaveLength(10);
    expect(ENTITIES).toHaveLength(6);
    expect(matrixCells(SPEC_MATRIX)).toHaveLength(48);
  });

  it.each([
    ['trips', 'D', 'không xoá cứng chuyến — cancelled là trạng thái kết thúc'],
    ['trip_events', 'RUD', 'nhật ký chỉ ghi thêm; KHÔNG chức năng nào đọc nó'],
    ['quotes', 'D', 'báo giá hết hạn không bao giờ được dọn'],
    ['drivers', 'CD', 'đăng ký / xoá tài xế ngoài phạm vi lab'],
    ['vouchers', 'CUD', 'quản trị voucher ngoài phạm vi lab'],
    ['idempotency_keys', '', 'đủ C, R, U, D'],
  ] as const)('%s thiếu [%s] — %s', (entity, ops) => {
    expect(missing[entity].join('')).toBe(ops);
  });
});
