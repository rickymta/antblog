/**
 * TEST-030 · Cùng hai tiêu chí chấp nhận, chạy trên bộ giới hạn thật (src/api/rate-limit.ts — đạt) và trên
 * bản «hiểu theo bản nháp» (naive-rate-limit.ts — `it.fails`: PHẢI thất bại). Minh hoạ: nếu AC không được
 * chốt trước khi code, defect chỉ lộ ra ở system test hoặc production.
 *
 * Chạy: npx vitest run articles/shift-left-testing
 */
import { describe, expect, it } from 'vitest';
import { createRateLimiter } from '../../src/api/rate-limit';
import { vnTime } from '../../src/domain/time';
import { FixedClock } from '../../src/infra/clock';
import { createNaiveRateLimiter } from './naive-rate-limit';

const T0 = vnTime(2026, 11, 4, 9, 0); // đúng đầu phút 09:00:00
type Limiter = { check(key: string): { allowed: boolean; retryAfterSeconds?: number } };
type Make = (clock: FixedClock) => Limiter;

const thật: Make = (clock) => createRateLimiter({ max: 5, windowMs: 60_000, clock });
const bảnNháp: Make = (clock) => createNaiveRateLimiter({ max: 5, clock });

/** Gọi `check` tại các mốc giây (tính từ T0), trả số lần được phép. */
function allowedCount(make: Make, seconds: number[]): number {
  const clock = new FixedClock(T0);
  const limiter = make(clock);
  return seconds.filter((s) => {
    clock.set(new Date(T0.getTime() + s * 1000));
    return limiter.check('C001').allowed;
  }).length;
}

function retryAfterAt59_9(make: Make): number | undefined {
  const clock = new FixedClock(T0);
  const limiter = make(clock);
  for (const s of [0, 10, 20, 30, 40]) { clock.set(new Date(T0.getTime() + s * 1000)); limiter.check('C001'); }
  clock.set(new Date(T0.getTime() + 59_900));
  const d = limiter.check('C001');
  return d.allowed ? undefined : d.retryAfterSeconds;
}

// 5 yêu cầu ở giây 58,0–58,4 và 5 yêu cầu ở giây 60,0–60,4: 10 yêu cầu trong 2,4 giây
const burstQuaRanhPhut = [58, 58.1, 58.2, 58.3, 58.4, 60, 60.1, 60.2, 60.3, 60.4];

describe('AC-6 · cửa sổ 60 giây trượt — tối đa 5 yêu cầu trong mọi khoảng 60 giây', () => {
  it('bộ giới hạn thật: chỉ 5/10 yêu cầu qua', () => expect(allowedCount(thật, burstQuaRanhPhut)).toBe(5));
  it.fails('bản nháp: phải chỉ 5/10 (bản nháp cho 10/10 vì đếm lại từ đầu phút 09:01)', () =>
    expect(allowedCount(bảnNháp, burstQuaRanhPhut)).toBe(5));
});

describe('AC-4 · Retry-After làm tròn lên, tối thiểu 1', () => {
  it('bộ giới hạn thật: lúc 59,9 giây ⇒ Retry-After 1', () => expect(retryAfterAt59_9(thật)).toBe(1));
  it.fails('bản nháp: phải là 1 (bản nháp trả 0 — client thử lại ngay, vòng lặp 429)', () =>
    expect(retryAfterAt59_9(bảnNháp)).toBe(1));
});
