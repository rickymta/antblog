/**
 * Giờ nghiệp vụ = giờ Việt Nam UTC+7 (SPEC R-GEN-02).
 * Tính tường minh từ mốc UTC, KHÔNG phụ thuộc múi giờ của máy chạy.
 */

export const VN_OFFSET_MS = 7 * 60 * 60 * 1000;
export const MINUTE_MS = 60 * 1000;

/** Đồng hồ được tiêm vào mọi nơi cần "bây giờ" — test thay bằng đồng hồ giả. */
export interface Clock {
  now(): Date;
}

export interface VnParts {
  year: number;
  month: number; // 1–12
  day: number;
  hour: number;
  minute: number;
  second: number;
}

/** Tách một thời điểm thành các thành phần theo giờ Việt Nam. */
export function toVnParts(at: Date): VnParts {
  const shifted = new Date(at.getTime() + VN_OFFSET_MS);
  return {
    year: shifted.getUTCFullYear(),
    month: shifted.getUTCMonth() + 1,
    day: shifted.getUTCDate(),
    hour: shifted.getUTCHours(),
    minute: shifted.getUTCMinutes(),
    second: shifted.getUTCSeconds(),
  };
}

/** Ngày theo giờ Việt Nam dạng `YYYY-MM-DD` (so sánh chuỗi được). */
export function vnDate(at: Date): string {
  const p = toVnParts(at);
  return `${p.year}-${String(p.month).padStart(2, '0')}-${String(p.day).padStart(2, '0')}`;
}

/** Khoảng nửa hở [start, end) theo UTC của ngày Việt Nam chứa `at`. */
export function vnDayRange(at: Date): { start: Date; end: Date } {
  const p = toVnParts(at);
  const start = new Date(Date.UTC(p.year, p.month - 1, p.day) - VN_OFFSET_MS);
  return { start, end: new Date(start.getTime() + 24 * 60 * MINUTE_MS) };
}

/** Giờ cao điểm: [11:00, 13:00) hoặc [17:00, 19:00) giờ Việt Nam (SPEC R-FARE-04). */
export function isPeakTime(at: Date): boolean {
  const { hour } = toVnParts(at);
  return (hour >= 11 && hour < 13) || (hour >= 17 && hour < 19);
}

/** Tạo thời điểm từ giờ Việt Nam — tiện cho test: `vnTime(2026, 11, 4, 11, 30)`. */
export function vnTime(year: number, month: number, day: number, hour = 0, minute = 0, second = 0): Date {
  return new Date(Date.UTC(year, month - 1, day, hour, minute, second) - VN_OFFSET_MS);
}

export function addMinutes(at: Date, minutes: number): Date {
  return new Date(at.getTime() + minutes * MINUTE_MS);
}
