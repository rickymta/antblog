/**
 * TCD-05 · Test data builder cho từng test case.
 *
 * Mỗi test case tự khai báo dữ liệu tiền điều kiện của mình (khách, voucher, thời tiết, các chuyến
 * đã tiêu lượt voucher) rồi gọi `build()` để có một hệ thống MỚI, không dùng chung với ca nào khác.
 * Dữ liệu đều là dữ liệu giả lập: tên «Khách Thử nn», số điện thoại dải 0900000xxx.
 */
import type { Actor, AppDeps } from '../../src/app/deps';
import { createQuote, type QuoteInput } from '../../src/app/quote';
import { cancelByCustomer, createTrip } from '../../src/app/trips';
import { vnTime } from '../../src/domain/time';
import type { Customer, Trip } from '../../src/domain/trip';
import type { CityCode } from '../../src/domain/types';
import type { Voucher } from '../../src/domain/voucher';
import { FixedClock } from '../../src/infra/clock';
import { createMemorySystem, type LabSystem } from '../../src/infra/compose';
import { labSeed, voucher } from '../../src/infra/seed';

/** Khách giả lập: mã C9nn, tên «Khách Thử nn», số điện thoại 0900000(900+n). */
export function aCustomer(n = 1): Customer {
  const nn = String(n).padStart(2, '0');
  return { id: `C9${nn}`, phone: `0900000${900 + n}`, fullName: `Khách Thử ${nn}` };
}

/** Voucher giả lập, mỗi lần gọi trả một object MỚI. Mặc định: THU20, 20%, trần 30.000 đ, 1 lượt/ngày. */
export function aVoucher(overrides: Partial<Voucher> = {}): Voucher {
  return voucher({ code: 'THU20', percent: 20, maxDiscount: 30_000, minAmount: 0, ...overrides });
}

/** Yêu cầu báo giá đặt xe mặc định: SG, xe máy, 7,4 km, không voucher. */
export function aRideRequest(overrides: Partial<Extract<QuoteInput, { kind: 'ride' }>> = {}): QuoteInput {
  return { kind: 'ride', cityCode: 'SG', vehicle: 'bike', distanceKm: 7.4, ...overrides };
}

interface PriorTrip {
  at: Date;
  customerId: string;
  voucherCode: string;
  cancelled: boolean;
}

export interface World {
  system: LabSystem;
  deps: AppDeps;
  clock: FixedClock;
  /** Khách chính của test case (khách đầu tiên được khai báo). */
  customer: Actor;
  /** Các chuyến đã dựng làm dữ liệu tiền điều kiện. */
  priorTrips: Trip[];
}

export class WorldBuilder {
  private now = vnTime(2026, 11, 16, 9, 0, 0); // 09:00 giờ VN — ngoài cao điểm
  private readonly customers: Customer[] = [aCustomer(1)];
  private readonly vouchers: Voucher[] = [aVoucher()];
  private raining: CityCode[] = [];
  private readonly priorTrips: PriorTrip[] = [];

  /** Thời điểm «bây giờ» của test case, sau khi đã dựng xong dữ liệu tiền điều kiện. */
  at(at: Date): this {
    this.now = at;
    return this;
  }

  withCustomer(customer: Customer): this {
    this.customers.push(customer);
    return this;
  }

  /** Thêm hoặc thay voucher cùng mã. */
  withVoucher(v: Voucher): this {
    const i = this.vouchers.findIndex((x) => x.code === v.code);
    if (i >= 0) this.vouchers[i] = v;
    else this.vouchers.push(v);
    return this;
  }

  withRain(...cities: CityCode[]): this {
    this.raining = cities;
    return this;
  }

  /**
   * Một chuyến đã tạo bằng voucher tại thời điểm `at` (dữ liệu tiền điều kiện của R-VOUCHER-07).
   * Chuyến được dựng qua đúng use case báo giá → tạo chuyến (→ huỷ nếu cần), không chèn thẳng vào kho.
   */
  withVoucherTrip(at: Date, opts: { customerId?: string; voucherCode?: string; cancelled?: boolean } = {}): this {
    this.priorTrips.push({
      at,
      customerId: opts.customerId ?? this.customers[0]!.id,
      voucherCode: opts.voucherCode ?? this.vouchers[0]!.code,
      cancelled: opts.cancelled ?? false,
    });
    return this;
  }

  async build(): Promise<World> {
    const ordered = [...this.priorTrips].sort((a, b) => a.at.getTime() - b.at.getTime());
    const clock = new FixedClock(ordered[0]?.at ?? this.now);
    const system = createMemorySystem({
      clock,
      seed: { customers: [...this.customers], drivers: labSeed().drivers, vouchers: [...this.vouchers] },
      rainingCities: this.raining,
    });
    const trips: Trip[] = [];
    for (const p of ordered) {
      clock.set(p.at);
      const actor: Actor = { role: 'customer', id: p.customerId };
      const quote = await createQuote(system.deps, actor, aRideRequest({ voucherCode: p.voucherCode }));
      const trip = await createTrip(system.deps, actor, { quoteId: quote.id });
      trips.push(p.cancelled ? (await cancelByCustomer(system.deps, actor, trip.id)).trip : trip);
    }
    clock.set(this.now);
    return {
      system,
      deps: system.deps,
      clock,
      customer: { role: 'customer', id: this.customers[0]!.id },
      priorTrips: trips,
    };
  }
}

export const aWorld = () => new WorldBuilder();
