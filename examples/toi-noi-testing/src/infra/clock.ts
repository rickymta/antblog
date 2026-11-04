import type { Clock } from '../domain/time';

/** Đồng hồ đứng yên — cho unit test: chỉ đổi khi gọi `set`/`advance`. */
export class FixedClock implements Clock {
  private current: Date;

  constructor(start: Date) {
    this.current = new Date(start);
  }

  now(): Date {
    return new Date(this.current);
  }

  set(at: Date): void {
    this.current = new Date(at);
  }

  advanceMinutes(minutes: number): void {
    this.current = new Date(this.current.getTime() + minutes * 60_000);
  }

  advanceMs(ms: number): void {
    this.current = new Date(this.current.getTime() + ms);
  }
}

/**
 * Đồng hồ chạy theo thời gian thật nhưng dời được mốc — cho máy chủ lab:
 * `set()` nhảy tới một thời điểm rồi tiếp tục trôi (dùng bởi API test và TOINOI_START_AT).
 */
export class AdjustableClock implements Clock {
  private offsetMs = 0;

  constructor(start?: Date) {
    if (start) this.set(start);
  }

  now(): Date {
    return new Date(Date.now() + this.offsetMs);
  }

  set(at: Date): void {
    this.offsetMs = at.getTime() - Date.now();
  }
}
