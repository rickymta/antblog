/**
 * Lắp ráp phụ thuộc cho test và máy chủ lab. Trả kèm các bản giả (`fakes`) để test điều khiển:
 * đồng hồ, thời tiết, ví VíXanh; và `reset()` đưa dữ liệu về trạng thái mẫu.
 */
import type { PGlite } from '@electric-sql/pglite';
import { DEFAULT_CONFIG, type AppConfig, type AppDeps } from '../app/deps';
import type { Clock, IdGenerator } from '../app/ports';
import type { CityCode } from '../domain/types';
import { FakePaymentGateway } from './fake-payment-gateway';
import { FakeWeather } from './fake-weather';
import { SequentialIdGenerator } from './ids';
import {
  MemoryCustomerRepository,
  MemoryDriverRepository,
  MemoryIdempotencyStore,
  MemoryQuoteRepository,
  MemoryTripRepository,
  MemoryVoucherRepository,
} from './memory/repositories';
import { createPgliteDatabase, resetPgliteData } from './pglite/database';
import {
  PgCustomerRepository,
  PgDriverRepository,
  PgIdempotencyStore,
  PgQuoteRepository,
  PgVoucherRepository,
} from './pglite/pg-repositories';
import { PgTripRepository } from './pglite/pg-trip-repository';
import { labSeed, type SeedData } from './seed';

export interface ComposeOptions {
  clock: Clock;
  ids?: IdGenerator;
  seed?: SeedData;
  config?: Partial<AppConfig>;
  rainingCities?: CityCode[];
}

export interface LabSystem {
  deps: AppDeps;
  weather: FakeWeather;
  payments: FakePaymentGateway;
  /** Xoá dữ liệu nghiệp vụ, nạp lại dữ liệu mẫu, đưa bản giả về mặc định. */
  reset(): Promise<void>;
  /** Chỉ có với PGlite — để test truy vấn SQL trực tiếp. */
  db?: PGlite;
}

function baseDeps(options: ComposeOptions) {
  const weather = new FakeWeather(options.rainingCities ?? []);
  const payments = new FakePaymentGateway();
  const config: AppConfig = { ...DEFAULT_CONFIG, ...options.config };
  return { weather, payments, config, ids: options.ids ?? new SequentialIdGenerator() };
}

/** Hệ thống với kho trong bộ nhớ — nhanh, dùng cho phần lớn test. */
export function createMemorySystem(options: ComposeOptions): LabSystem {
  const seed = options.seed ?? labSeed();
  const { weather, payments, config, ids } = baseDeps(options);
  const repos = {
    quotes: new MemoryQuoteRepository(),
    trips: new MemoryTripRepository(),
    drivers: new MemoryDriverRepository(seed.drivers),
    customers: new MemoryCustomerRepository(seed.customers),
    vouchers: new MemoryVoucherRepository(seed.vouchers),
    idempotency: new MemoryIdempotencyStore(),
  };
  const deps: AppDeps = { ...repos, payments, weather, clock: options.clock, ids, config };

  return {
    deps,
    weather,
    payments,
    async reset() {
      Object.values(repos).forEach((repo) => repo.clear());
      seed.customers.forEach((c) => repos.customers.add(c));
      seed.drivers.forEach((d) => repos.drivers.add(d));
      seed.vouchers.forEach((v) => repos.vouchers.add(v));
      weather.setRaining(options.rainingCities ?? []);
      payments.reset();
    },
  };
}

/** Hệ thống với PostgreSQL thật chạy bằng PGlite (một kết nối). */
export async function createPgliteSystem(options: ComposeOptions): Promise<LabSystem> {
  const seed = options.seed ?? labSeed();
  const db = await createPgliteDatabase();
  await resetPgliteData(db, seed);
  const { weather, payments, config, ids } = baseDeps(options);
  const deps: AppDeps = {
    quotes: new PgQuoteRepository(db),
    trips: new PgTripRepository(db),
    drivers: new PgDriverRepository(db),
    customers: new PgCustomerRepository(db),
    vouchers: new PgVoucherRepository(db),
    idempotency: new PgIdempotencyStore(db),
    payments,
    weather,
    clock: options.clock,
    ids,
    config,
  };

  return {
    deps,
    weather,
    payments,
    db,
    async reset() {
      await resetPgliteData(db, seed);
      weather.setRaining(options.rainingCities ?? []);
      payments.reset();
    },
  };
}
