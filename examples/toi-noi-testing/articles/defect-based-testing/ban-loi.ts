/**
 * TEST-054 — tái tạo các defect LỊCH SỬ (lich-su-defect.csv) thành các bản cài đặt có lỗi, để chứng minh
 * bộ test thiết kế theo mô hình lỗi bắt được khi chúng TÁI DIỄN. Không sửa src/: mỗi bản bọc hoặc chép lại hàm thật
 * và chỉ đổi đúng chỗ defect cũ từng sai.
 */
import { calculateFare, type FareInput } from '../../src/domain/fare';
import { TRANSITIONS, type TripEvent, type TripStatus } from '../../src/domain/trip-state';

// ---------------------------------------------------------------- cụm lam-tron

/** D-01 / D-22: làm tròn 0,1 bằng toFixed — 7,35 thành 7,3 vì 7,35 lưu trong máy là 7,3499999… */
export const normalizeDistanceToFixed = (raw: number): number => Number(raw.toFixed(1));

/** D-04: bước cuối làm tròn tới 1.000 đ GẦN NHẤT thay vì làm tròn LÊN (R-FARE-08). */
export function fareTotalRoundNearest(input: FareInput): number {
  const b = calculateFare(input);
  return Math.round(Math.max(b.afterDiscount, b.minimumFare) / 1000) * 1000;
}

/** D-06 / D-09: tiền giảm voucher làm tròn tới 1.000 đ gần nhất TRƯỚC khi trừ (R-VOUCHER-06 không làm tròn tiền giảm). */
export function fareTotalRoundDiscount(input: FareInput): number {
  const b = calculateFare(input);
  const discount = Math.round(b.discount / 1000) * 1000;
  return Math.ceil(Math.max(b.beforeDiscount - discount, b.minimumFare) / 1000) * 1000;
}

/**
 * Biến thể «làm tròn về đồng ngay sau ×1,25» (17.812,5 ⇒ 17.813) — trái chữ của R-GEN-01 nhưng, như bộ test cho thấy,
 * KHÔNG đổi tổng tiền nào trong miền hợp lệ: phần lẻ chỉ là 0,5 đ và bước cuối luôn làm tròn LÊN 1.000 đ. Biến thể tương đương.
 */
export function fareTotalRoundAfterPeak(input: FareInput): number {
  const b = calculateFare(input);
  const before = Math.round(b.afterPeak) + b.rainSurcharge;
  const v = input.voucher;
  const discount = v ? Math.min((before * v.percent) / 100, v.maxDiscount) : 0;
  return Math.ceil(Math.max(before - discount, b.minimumFare) / 1000) * 1000;
}

// ---------------------------------------------------------------- cụm gio-vn

/** D-02 / D-29: giờ cao điểm lấy theo giờ của máy chủ — máy chủ chạy UTC (getUTCHours mô phỏng đúng điều đó). */
export function isPeakServerUtc(at: Date): boolean {
  const h = at.getUTCHours();
  return (h >= 11 && h < 13) || (h >= 17 && h < 19);
}

/** D-24: đầu cuối của khung cao điểm bị tính vào (<= thay vì <). */
export function isPeakEndInclusive(at: Date): boolean {
  const vn = new Date(at.getTime() + 7 * 3_600_000);
  const minutes = vn.getUTCHours() * 60 + vn.getUTCMinutes() + vn.getUTCSeconds() / 60;
  return (minutes >= 660 && minutes <= 780) || (minutes >= 1020 && minutes <= 1140);
}

/** D-05 / D-11: «ngày» của voucher lấy từ toISOString() — tức ngày UTC, lệch ngày VN suốt 7 tiếng. */
export function voucherValidIsoDate(at: Date, validFrom: string, validTo: string): boolean {
  const day = at.toISOString().slice(0, 10);
  return day >= validFrom && day <= validTo;
}

// ---------------------------------------------------------------- cụm trang-thai

type Next = (from: TripStatus, event: TripEvent) => TripStatus | 'INVALID';
const real: Next = (from, event) => TRANSITIONS[from][event] ?? 'INVALID';

/** D-08: khách huỷ được cả khi đã đón (picked_up) và đang chở (in_transit). */
export const nextCustomerCancelTooLate: Next = (from, event) =>
  event === 'customer_cancel' && (from === 'picked_up' || from === 'in_transit') ? 'cancelled' : real(from, event);

/** D-18: «Huỷ nhận» bấm muộn đưa chuyến completed về searching. */
export const nextDriverCancelAfterComplete: Next = (from, event) =>
  event === 'driver_cancel' && from === 'completed' ? 'searching' : real(from, event);
