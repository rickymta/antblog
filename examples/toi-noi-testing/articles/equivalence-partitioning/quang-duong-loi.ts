/**
 * TEST-031 · Hai bản cài đặt CÓ LỖI CỐ Ý của normalizeDistanceKm (bản đúng: src/domain/fare.ts).
 * Chỉ để chứng minh EP bắt được lỗi nào, bỏ lọt lỗi nào. KHÔNG dùng ở nơi khác.
 */
import { roundToTenth } from '../../src/domain/money';

export type Normalize = (raw: number) => number;

function outOfRange(): never {
  throw new Error('DISTANCE_OUT_OF_RANGE');
}

/** Lỗi A — quên cận trên 100 km (phân vùng D3 bị nuốt vào D2). */
export const thieuCanTren: Normalize = (raw) => {
  const km = roundToTenth(raw);
  if (!Number.isFinite(km) || km < 0.5) outOfRange();
  return km;
};

/** Lỗi B — so sánh TRƯỚC khi làm tròn (sai thứ tự R-FARE-02): 0,45 bị từ chối, 100,04 bị từ chối. */
export const kiemTruocLamTron: Normalize = (raw) => {
  if (!Number.isFinite(raw) || raw < 0.5 || raw > 100) outOfRange();
  return roundToTenth(raw);
};
