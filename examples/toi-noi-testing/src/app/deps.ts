import type {
  Clock,
  CustomerRepository,
  DriverRepository,
  IdGenerator,
  IdempotencyStore,
  PaymentGateway,
  QuoteRepository,
  TripRepository,
  VoucherRepository,
  WeatherService,
} from './ports';

export interface AppConfig {
  payment: {
    timeoutMs: number; // SPEC R-PAY-04: 3 giây
    maxRetries: number; // SPEC R-PAY-04: tối đa 2 lần thử lại
  };
  tripRateLimit: {
    max: number; // SPEC R-API-13: 5 yêu cầu; 0 = tắt
    windowMs: number; // 60 giây trượt
  };
  searchTimeoutMinutes: number; // SPEC R-STATE-05: 5 phút
  quoteTtlMinutes: number; // SPEC R-QUOTE-01: 5 phút
}

export const DEFAULT_CONFIG: AppConfig = {
  payment: { timeoutMs: 3_000, maxRetries: 2 },
  tripRateLimit: { max: 5, windowMs: 60_000 },
  searchTimeoutMinutes: 5,
  quoteTtlMinutes: 5,
};

/** Mọi phụ thuộc của use case — tiêm qua tham số, không có biến toàn cục. */
export interface AppDeps {
  quotes: QuoteRepository;
  trips: TripRepository;
  drivers: DriverRepository;
  customers: CustomerRepository;
  vouchers: VoucherRepository;
  payments: PaymentGateway;
  weather: WeatherService;
  clock: Clock;
  ids: IdGenerator;
  idempotency: IdempotencyStore;
  config: AppConfig;
}

/** Người gọi đã xác thực (SPEC R-API-03). */
export interface Actor {
  role: 'customer' | 'driver';
  id: string;
}
