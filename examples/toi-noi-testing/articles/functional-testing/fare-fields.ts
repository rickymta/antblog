/**
 * Ánh xạ tên bước trong SPEC R-FARE-10 sang khoá JSON của bảng chi tiết cước (`quote.fare`).
 * Dùng cho kiểm tra functional completeness: «mọi bước đặc tả đòi hỏi đều có mặt trong kết quả».
 */
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

export const FARE_FIELD_BY_LABEL: Record<string, string> = {
  'phí mở cửa': 'openingFee',
  'tiền theo km': 'distanceCharge',
  'phụ phí bưu kiện': 'parcelSurcharge',
  'cước cơ bản': 'base',
  'sau cao điểm': 'afterPeak',
  'phụ phí mưa': 'rainSurcharge',
  'trước giảm': 'beforeDiscount',
  'tiền giảm': 'discount',
  'sau giảm': 'afterDiscount',
  'cước tối thiểu': 'minimumFare',
  'tổng': 'total',
};

/** Đọc danh sách bước ngay trong SPEC.md, để test tự đỏ nếu đặc tả thêm/bớt bước. */
export function specFareLabels(): string[] {
  const specPath = fileURLToPath(new URL('../../SPEC.md', import.meta.url));
  const line = readFileSync(specPath, 'utf8')
    .split(/\r?\n/)
    .find((l) => l.includes('R-FARE-10 — Bảng chi tiết cước'));
  if (!line) throw new Error('Không tìm thấy R-FARE-10 trong SPEC.md');
  const list = line.split('từng bước:')[1]?.split('. Các')[0];
  if (!list) throw new Error('Không tách được danh sách bước của R-FARE-10');
  return list.split(',').map((s) => s.trim());
}

/** Trả về tên các bước (theo SPEC) bị thiếu hoặc không phải số hữu hạn trong một bảng chi tiết cước. */
export function missingFareFields(fare: Record<string, unknown>): string[] {
  return Object.entries(FARE_FIELD_BY_LABEL)
    .filter(([, key]) => typeof fare[key] !== 'number' || !Number.isFinite(fare[key] as number))
    .map(([label]) => label);
}
