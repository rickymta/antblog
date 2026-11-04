/**
 * Kho dữ liệu trong bộ nhớ — nhanh, tất định, dùng cho unit/API test và máy chủ lab mặc định.
 * Mọi giá trị trả ra là BẢN SAO (structuredClone) để test không vô tình sửa dữ liệu trong kho.
 */
import type {
  CustomerRepository,
  DriverRepository,
  IdempotencyBegin,
  IdempotencyStore,
  QuoteRepository,
  StoredResponse,
  TripRepository,
  VoucherRepository,
} from '../../app/ports';
import type { StoredQuote } from '../../app/quote';
import type { Customer, Driver, Trip, TripEventRecord } from '../../domain/trip';
import type { TripStatus } from '../../domain/trip-state';
import type { CityCode, Vehicle } from '../../domain/types';
import type { Voucher } from '../../domain/voucher';

const clone = <T>(value: T): T => structuredClone(value);

export class MemoryQuoteRepository implements QuoteRepository {
  private readonly quotes = new Map<string, StoredQuote>();

  async save(quote: StoredQuote): Promise<void> {
    this.quotes.set(quote.id, clone(quote));
  }

  async findById(id: string): Promise<StoredQuote | null> {
    const quote = this.quotes.get(id);
    return quote ? clone(quote) : null;
  }

  async markUsed(quoteId: string, tripId: string): Promise<boolean> {
    const quote = this.quotes.get(quoteId);
    if (!quote || quote.usedByTripId !== null) return false;
    quote.usedByTripId = tripId;
    return true;
  }

  clear(): void {
    this.quotes.clear();
  }
}

export class MemoryTripRepository implements TripRepository {
  private readonly trips = new Map<string, Trip>();
  private readonly events: TripEventRecord[] = [];

  async create(trip: Trip, events: TripEventRecord[]): Promise<void> {
    if (this.trips.has(trip.id)) throw new Error(`Trùng id chuyến ${trip.id}`);
    this.trips.set(trip.id, clone(trip));
    this.events.push(...clone(events));
  }

  async findById(id: string): Promise<Trip | null> {
    const trip = this.trips.get(id);
    return trip ? clone(trip) : null;
  }

  async updateIfStatus(trip: Trip, expected: TripStatus): Promise<boolean> {
    const current = this.trips.get(trip.id);
    if (!current || current.status !== expected) return false;
    this.trips.set(trip.id, clone(trip));
    if (current.status !== trip.status) this.events.push({ tripId: trip.id, status: trip.status, occurredAt: trip.updatedAt });
    return true;
  }

  async acceptIfSearching(input: { tripId: string; driverId: string; acceptedAt: Date; etaMinutes: number }): Promise<Trip | null> {
    const current = this.trips.get(input.tripId);
    if (!current || current.status !== 'searching') return null;
    const accepted: Trip = {
      ...current,
      status: 'accepted',
      driverId: input.driverId,
      acceptedAt: input.acceptedAt,
      etaMinutes: input.etaMinutes,
      updatedAt: input.acceptedAt,
    };
    this.trips.set(accepted.id, clone(accepted));
    this.events.push({ tripId: accepted.id, status: 'accepted', occurredAt: input.acceptedAt });
    return clone(accepted);
  }

  async listSearching(filter: { cityCode: CityCode; vehicle: Vehicle }): Promise<Trip[]> {
    return [...this.trips.values()]
      .filter((t) => t.status === 'searching' && t.cityCode === filter.cityCode && t.vehicle === filter.vehicle)
      .sort((a, b) => a.requestedAt.getTime() - b.requestedAt.getTime() || a.id.localeCompare(b.id))
      .map(clone);
  }

  async listSearchingSince(cutoff: Date): Promise<Trip[]> {
    return [...this.trips.values()]
      .filter((t) => t.status === 'searching' && t.searchingSince !== null && t.searchingSince <= cutoff)
      .map(clone);
  }

  async countVoucherUses(input: { customerId: string; voucherCode: string; from: Date; to: Date }): Promise<number> {
    return [...this.trips.values()].filter(
      (t) =>
        t.customerId === input.customerId &&
        t.voucherCode === input.voucherCode &&
        t.status !== 'cancelled' &&
        t.requestedAt >= input.from &&
        t.requestedAt < input.to,
    ).length;
  }

  async listEvents(tripId: string): Promise<TripEventRecord[]> {
    return clone(this.events.filter((e) => e.tripId === tripId));
  }

  clear(): void {
    this.trips.clear();
    this.events.length = 0;
  }
}

export class MemoryDriverRepository implements DriverRepository {
  private readonly drivers = new Map<string, Driver>();

  constructor(drivers: Driver[] = []) {
    drivers.forEach((d) => this.add(d));
  }

  add(driver: Driver): void {
    this.drivers.set(driver.id, clone(driver));
  }

  async findById(id: string): Promise<Driver | null> {
    const driver = this.drivers.get(id);
    return driver ? clone(driver) : null;
  }

  async tryMarkBusy(id: string): Promise<boolean> {
    const driver = this.drivers.get(id);
    if (!driver || driver.status !== 'online') return false;
    driver.status = 'busy';
    return true;
  }

  async markOnline(id: string): Promise<void> {
    const driver = this.drivers.get(id);
    if (driver && driver.status === 'busy') driver.status = 'online';
  }

  clear(): void {
    this.drivers.clear();
  }
}

export class MemoryCustomerRepository implements CustomerRepository {
  private readonly customers = new Map<string, Customer>();

  constructor(customers: Customer[] = []) {
    customers.forEach((c) => this.add(c));
  }

  add(customer: Customer): void {
    this.customers.set(customer.id, clone(customer));
  }

  async findById(id: string): Promise<Customer | null> {
    const customer = this.customers.get(id);
    return customer ? clone(customer) : null;
  }

  clear(): void {
    this.customers.clear();
  }
}

export class MemoryVoucherRepository implements VoucherRepository {
  private readonly vouchers = new Map<string, Voucher>();

  constructor(vouchers: Voucher[] = []) {
    vouchers.forEach((v) => this.add(v));
  }

  add(voucher: Voucher): void {
    this.vouchers.set(voucher.code, clone(voucher));
  }

  async findByCode(code: string): Promise<Voucher | null> {
    const voucher = this.vouchers.get(code);
    return voucher ? clone(voucher) : null;
  }

  clear(): void {
    this.vouchers.clear();
  }
}

export class MemoryIdempotencyStore implements IdempotencyStore {
  private readonly records = new Map<string, { fingerprint: string; response: StoredResponse | null }>();

  async begin(scope: string, key: string, fingerprint: string): Promise<IdempotencyBegin> {
    const id = `${scope}|${key}`;
    const existing = this.records.get(id);
    if (!existing) {
      this.records.set(id, { fingerprint, response: null });
      return { state: 'new' };
    }
    if (existing.fingerprint !== fingerprint) return { state: 'mismatch' };
    if (existing.response === null) return { state: 'in_progress' };
    return { state: 'replay', response: clone(existing.response) };
  }

  async complete(scope: string, key: string, response: StoredResponse): Promise<void> {
    const record = this.records.get(`${scope}|${key}`);
    if (record) record.response = clone(response);
  }

  async release(scope: string, key: string): Promise<void> {
    this.records.delete(`${scope}|${key}`);
  }

  clear(): void {
    this.records.clear();
  }
}
