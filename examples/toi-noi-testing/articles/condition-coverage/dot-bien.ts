/**
 * TEST-045 — ba bản cài đặt SAI của quyết định «voucher áp được» (lỗi gài, không có trong src/).
 * Mỗi bản nhận cùng bốn giá trị điều kiện và trả kết quả quyết định.
 */
import type { GiaTriDieuKien } from './dieu-kien-voucher';

export const DOT_BIEN: Record<string, { moTa: string; quyetDinh: (g: GiaTriDieuKien) => boolean }> = {
  M1: {
    moTa: 'quên điều kiện thành phố',
    quyetDinh: (g) => g.trongHan && g.dungDichVu && g.duTien,
  },
  M2: {
    moTa: 'dịch vụ HOẶC thành phố (|| thay &&)',
    quyetDinh: (g) => g.trongHan && (g.dungDichVu || g.dungThanhPho) && g.duTien,
  },
  M3: {
    moTa: 'bỏ kiểm tiền tối thiểu',
    quyetDinh: (g) => g.trongHan && g.dungDichVu && g.dungThanhPho,
  },
};
