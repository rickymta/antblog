/**
 * TEST-039 · Ghi lại thao tác CRUD THỰC SỰ xảy ra khi chạy test, để đo CRUD coverage.
 * Bọc các repository của lab bằng Proxy: mỗi lời gọi được quy về (thực thể, thao tác)
 * và gán cho chức năng đang chạy (`tracer.as('F05', …)`). Không sửa `src/`.
 * Có thêm một móc gây lỗi (`failOnce`) để mô phỏng database lỗi giữa chừng.
 */
import type { AppDeps } from '../../src/app/deps';
import type { Entity, Op } from './crud-matrix';

type Hit = [Entity, Op];
type Classifier = (args: unknown[], result: unknown) => Hit[];

/** Lời gọi repository ⇒ thao tác CRUD. Cập nhật có điều kiện mà thua ⇒ chỉ là một lần đọc. */
const RULES: Record<string, Classifier> = {
  'quotes.save': () => [['quotes', 'C']],
  'quotes.findById': () => [['quotes', 'R']],
  'quotes.markUsed': (_a, ok) => [['quotes', ok ? 'U' : 'R']],
  'trips.create': () => [['trips', 'C'], ['trip_events', 'C']],
  'trips.findById': () => [['trips', 'R']],
  'trips.listSearching': () => [['trips', 'R']],
  'trips.listSearchingSince': () => [['trips', 'R']],
  'trips.countVoucherUses': () => [['trips', 'R']],
  'trips.listEvents': () => [['trip_events', 'R']],
  'trips.updateIfStatus': (args, ok) => {
    if (!ok) return [['trips', 'R']];
    const [trip, expected] = args as [{ status: string }, string];
    return trip.status !== expected ? [['trips', 'U'], ['trip_events', 'C']] : [['trips', 'U']];
  },
  'trips.acceptIfSearching': (_a, trip) => (trip ? [['trips', 'U'], ['trip_events', 'C']] : [['trips', 'R']]),
  'drivers.findById': () => [['drivers', 'R']],
  'drivers.tryMarkBusy': (_a, ok) => [['drivers', ok ? 'U' : 'R']],
  'drivers.markOnline': () => [['drivers', 'U']],
  'vouchers.findByCode': () => [['vouchers', 'R']],
  'idempotency.begin': (_a, r) => [['idempotency_keys', (r as { state: string }).state === 'new' ? 'C' : 'R']],
  'idempotency.complete': () => [['idempotency_keys', 'U']],
  'idempotency.release': () => [['idempotency_keys', 'D']],
};

export class CrudTracer {
  private current: string | null = null;
  private readonly faults = new Set<string>();
  readonly hits = new Set<string>(); // "F05|trips|U"

  /** Chạy `work` và gán mọi thao tác CRUD trong lúc đó cho chức năng `fnId`. */
  async as<T>(fnId: string, work: () => PromiseLike<T>): Promise<T> {
    this.current = fnId;
    try {
      return await work();
    } finally {
      this.current = null;
    }
  }

  /** Lần gọi tới `method` (vd 'trips.create') kế tiếp sẽ ném lỗi — giả lập database hỏng. */
  failOnce(method: string): void {
    this.faults.add(method);
  }

  wrap<T extends object>(name: string, target: T): T {
    return new Proxy(target, {
      get: (obj, prop, receiver) => {
        const value = Reflect.get(obj, prop, receiver);
        const method = `${name}.${String(prop)}`;
        const rule = RULES[method];
        if (typeof value !== 'function' || !rule) return value;
        return async (...args: unknown[]) => {
          if (this.faults.delete(method)) throw new Error(`Lỗi giả lập tại ${method}`);
          const result = await (value as (...a: unknown[]) => Promise<unknown>).apply(obj, args);
          if (this.current) for (const [entity, op] of rule(args, result)) this.hits.add(`${this.current}|${entity}|${op}`);
          return result;
        };
      },
    });
  }

  deps(base: AppDeps): AppDeps {
    return {
      ...base,
      quotes: this.wrap('quotes', base.quotes),
      trips: this.wrap('trips', base.trips),
      drivers: this.wrap('drivers', base.drivers),
      vouchers: this.wrap('vouchers', base.vouchers),
      idempotency: this.wrap('idempotency', base.idempotency),
    };
  }
}
