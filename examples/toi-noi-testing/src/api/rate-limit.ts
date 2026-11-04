import type { Clock } from '../domain/time';

export type RateDecision = { allowed: true } | { allowed: false; retryAfterSeconds: number };

/**
 * Giới hạn tần suất cửa sổ trượt, giữ trong bộ nhớ của một tiến trình (SPEC R-API-13).
 * Yêu cầu bị từ chối (429) không được đếm. `max = 0` ⇒ tắt.
 */
export function createRateLimiter(options: { max: number; windowMs: number; clock: Clock }) {
  const hits = new Map<string, number[]>();

  return {
    check(key: string): RateDecision {
      if (options.max <= 0) return { allowed: true };
      const now = options.clock.now().getTime();
      const recent = (hits.get(key) ?? []).filter((t) => t > now - options.windowMs);
      if (recent.length >= options.max) {
        hits.set(key, recent);
        const oldest = recent[0] ?? now;
        return { allowed: false, retryAfterSeconds: Math.max(1, Math.ceil((oldest + options.windowMs - now) / 1000)) };
      }
      recent.push(now);
      hits.set(key, recent);
      return { allowed: true };
    },
    reset(): void {
      hits.clear();
    },
  };
}

export type RateLimiter = ReturnType<typeof createRateLimiter>;
