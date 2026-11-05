/**
 * Mô hình cấu hình của ứng dụng khách Tới Nơi (TEST-037) và các lỗi tương tác GIẢ LẬP.
 *
 * Lab không có app Android/iOS thật: các lỗi F1…F5 dưới đây là defect hư cấu, gài để đo xem
 * một bộ test có "chạm" tới tổ hợp gây lỗi hay không. Chúng không phải lỗi của src/.
 */
import { label, type Model, type Row } from './pairwise';

/**
 * Giá trị phiên bản đã qua phân vùng tương đương (EP) theo từng nền tảng:
 * "cũ nhất" = phiên bản thấp nhất còn hỗ trợ, "phổ biến" = nhiều người dùng nhất, "mới nhất" = bản phát hành cuối.
 * Khi chạy thật mới đổi sang số cụ thể (vd Android 10 / 14 / 16, iOS 16 / 18 / 26, Chrome cũ / phổ biến / mới).
 */
export const APP_MODEL: Model = [
  { name: 'nenTang', values: ['Android', 'iOS', 'Web'] },
  { name: 'phienBan', values: ['cũ nhất', 'phổ biến', 'mới nhất'] },
  { name: 'thanhToan', values: ['cash', 'vixanh'] },
  { name: 'ngonNgu', values: ['vi', 'en'] },
  { name: 'mang', values: ['Wi-Fi', '4G', 'mạng yếu'] },
];

export interface SimulatedFault {
  id: string;
  /** Số tham số phải cùng nhận đúng giá trị thì lỗi mới lộ (bậc tương tác). */
  strength: number;
  when: Record<string, string>;
  story: string;
}

export const FAULTS: SimulatedFault[] = [
  { id: 'F1', strength: 1, when: { ngonNgu: 'en' }, story: 'Màn huỷ chuyến thiếu bản dịch phí huỷ (R-CANCEL-03)' },
  { id: 'F2', strength: 2, when: { nenTang: 'iOS', thanhToan: 'vixanh' }, story: 'Quay lại từ app VíXanh không về màn chuyến' },
  { id: 'F3', strength: 2, when: { nenTang: 'Web', mang: 'mạng yếu' }, story: 'Bấm «Đặt chuyến» lần hai khi chờ lâu sinh khoá idempotency mới' },
  { id: 'F4', strength: 3, when: { nenTang: 'Android', phienBan: 'cũ nhất', thanhToan: 'vixanh' }, story: 'SDK VíXanh gọi API trình duyệt không có trên Android cũ nhất' },
  {
    id: 'F5',
    strength: 4,
    when: { nenTang: 'iOS', phienBan: 'mới nhất', ngonNgu: 'en', mang: 'mạng yếu' },
    story: 'Thông báo hết giờ tiếng Anh bị cắt chữ trên bố cục mới',
  },
];

/** Hàng có kích hoạt lỗi không: mọi điều kiện của lỗi đều khớp. */
export function triggers(fault: SimulatedFault, row: Row): boolean {
  const values = label(APP_MODEL, row);
  return Object.entries(fault.when).every(([param, value]) => values[param] === value);
}

/** Danh sách lỗi mà bộ test chạm tới. */
export function faultsFound(suite: readonly Row[]): string[] {
  return FAULTS.filter((f) => suite.some((row) => triggers(f, row))).map((f) => f.id);
}

/**
 * Bộ 9 hàng dựng tay: ba tham số 3 giá trị (nền tảng, phiên bản, mạng) lấy theo mảng trực giao L9,
 * hai cột 2 giá trị (thanh toán, ngôn ngữ) điền sao cho mọi cặp còn lại đều xuất hiện.
 * 9 = 3 × 3 là cận dưới: hai tham số 3 giá trị bất kỳ đã có 9 cặp, mỗi hàng chỉ phủ được một.
 */
export const L9_SUITE: number[][] = [
  [0, 0, 1, 1, 0],
  [0, 1, 0, 0, 1],
  [0, 2, 0, 0, 2],
  [1, 0, 0, 0, 1],
  [1, 1, 1, 1, 2],
  [1, 2, 0, 0, 0],
  [2, 0, 1, 0, 2],
  [2, 1, 0, 1, 0],
  [2, 2, 1, 1, 1],
];
