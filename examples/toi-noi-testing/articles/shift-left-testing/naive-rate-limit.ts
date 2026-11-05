import type { Clock } from '../../src/domain/time';
import type { RateDecision } from '../../src/api/rate-limit';

/**
 * TEST-030 · Bản cài đặt «hiểu theo bản nháp story» — KHÔNG dùng ở đâu khác, không sửa src/.
 * Bản nháp chỉ ghi «tối đa 5 chuyến mỗi phút, vượt thì báo thử lại sau». Không có refinement,
 * dev hiểu hợp lý theo cách của mình:
 *   - «mỗi phút» = phút đồng hồ (cửa sổ cố định :00–:59), không phải 60 giây trượt;
 *   - đếm mọi lần gọi, kể cả lần bị chặn;
 *   - «thử lại sau» = số giây tới phút kế tiếp, làm tròn xuống.
 */
export function createNaiveRateLimiter(options: { max: number; clock: Clock }) {
  const counts = new Map<string, { minute: number; n: number }>();
  return {
    check(key: string): RateDecision {
      const nowMs = options.clock.now().getTime();
      const minute = Math.floor(nowMs / 60_000);
      const prev = counts.get(key);
      const n = prev && prev.minute === minute ? prev.n + 1 : 1;
      counts.set(key, { minute, n });
      if (n > options.max) {
        return { allowed: false, retryAfterSeconds: Math.floor(((minute + 1) * 60_000 - nowMs) / 1000) };
      }
      return { allowed: true };
    },
  };
}
