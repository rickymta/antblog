/**
 * Cổng (port) mà các use case phụ thuộc. Hạ tầng (in-memory, PGlite, giả lập) cài đặt chúng;
 * test thay bằng bản giả để cô lập từng lớp.
 */
import type { Clock } from '../domain/time';
import type { Customer, Driver, Trip, TripEventRecord } from '../domain/trip';
import type { TripStatus } from '../domain/trip-state';
import type { CityCode, Vehicle } from '../domain/types';
import type { Voucher } from '../domain/voucher';
import type { StoredQuote } from './quote';

export type { Clock };

export interface QuoteRepository {
  save(quote: StoredQuote): Promise<void>;
  findById(id: string): Promise<StoredQuote | null>;
  /** Đánh dấu báo giá đã dùng cho một chuyến — có điều kiện `used_by_trip_id IS NULL`. Trả false nếu đã dùng. */
  markUsed(quoteId: string, tripId: string): Promise<boolean>;
}

export interface TripRepository {
  /** Thêm chuyến mới cùng các dòng nhật ký trạng thái ban đầu. */
  create(trip: Trip, events: TripEventRecord[]): Promise<void>;
  findById(id: string): Promise<Trip | null>;
  /**
   * Ghi đè toàn bộ chuyến NẾU trạng thái trong kho vẫn là `expected` (khoá lạc quan).
   * Trạng thái đổi ⇒ ghi thêm một dòng nhật ký. Trả false nếu ai đó đã đổi chuyến trước.
   */
  updateIfStatus(trip: Trip, expected: TripStatus): Promise<boolean>;
  /**
   * Nhận chuyến có điều kiện: chỉ thành công khi chuyến còn `searching`
   * (`UPDATE … WHERE id = $1 AND status = 'searching' RETURNING …`). Trả null nếu thua.
   */
  acceptIfSearching(input: { tripId: string; driverId: string; acceptedAt: Date; etaMinutes: number }): Promise<Trip | null>;
  /** Chuyến `searching` theo thành phố + loại xe, cũ nhất trước. */
  listSearching(filter: { cityCode: CityCode; vehicle: Vehicle }): Promise<Trip[]>;
  /** Chuyến `searching` có `searchingSince` ≤ `cutoff`. */
  listSearchingSince(cutoff: Date): Promise<Trip[]>;
  /** Số chuyến CHƯA huỷ của khách dùng mã voucher, `requestedAt` trong [from, to). */
  countVoucherUses(input: { customerId: string; voucherCode: string; from: Date; to: Date }): Promise<number>;
  listEvents(tripId: string): Promise<TripEventRecord[]>;
}

export interface DriverRepository {
  findById(id: string): Promise<Driver | null>;
  /** online → busy có điều kiện. Trả false nếu tài xế không còn `online`. */
  tryMarkBusy(id: string): Promise<boolean>;
  /** busy → online (giải phóng tài xế). */
  markOnline(id: string): Promise<void>;
}

export interface CustomerRepository {
  findById(id: string): Promise<Customer | null>;
}

export interface VoucherRepository {
  findByCode(code: string): Promise<Voucher | null>;
}

/** Dịch vụ thời tiết (SPEC R-GEN-05). */
export interface WeatherService {
  isRaining(cityCode: CityCode, at: Date): Promise<boolean>;
}

export interface IdGenerator {
  next(): string;
}

// ---- Idempotency của API (SPEC R-IDEM) ----

/** Phản hồi HTTP đã lưu để trả lại khi client gửi lại cùng khoá. */
export interface StoredResponse {
  status: number;
  body: unknown;
}

export type IdempotencyBegin =
  | { state: 'new' } // đã giữ chỗ khoá — được phép xử lý
  | { state: 'replay'; response: StoredResponse } // đã xử lý xong trước đó
  | { state: 'mismatch' } // cùng khoá, khác nội dung
  | { state: 'in_progress' }; // yêu cầu cùng khoá đang chạy

export interface IdempotencyStore {
  /** Giữ chỗ (scope, key) một cách nguyên tử nếu chưa có; ngược lại cho biết trạng thái bản ghi cũ. */
  begin(scope: string, key: string, fingerprint: string, now: Date): Promise<IdempotencyBegin>;
  complete(scope: string, key: string, response: StoredResponse): Promise<void>;
  /** Bỏ giữ chỗ (lỗi 5xx) để client được thử lại với cùng khoá. */
  release(scope: string, key: string): Promise<void>;
}

// ---- Cổng thanh toán VíXanh (SPEC R-PAY) ----

export interface ChargeRequest {
  idempotencyKey: string;
  customerId: string;
  amount: number; // đồng
  description: string;
}

export interface ChargeResult {
  status: 'succeeded' | 'declined';
  chargeId: string | null;
}

export interface RefundRequest {
  idempotencyKey: string;
  chargeId: string;
  amount: number;
}

export interface RefundResult {
  status: 'succeeded' | 'rejected';
}

/** Lỗi tạm thời: cổng không trả lời được (nên thử lại). */
export class PaymentUnavailableError extends Error {
  constructor(message = 'Cổng thanh toán tạm thời không phản hồi') {
    super(message);
    this.name = 'PaymentUnavailableError';
  }
}

/** Cùng `idempotencyKey` ⇒ cổng chỉ trừ/hoàn tiền một lần. */
export interface PaymentGateway {
  charge(request: ChargeRequest): Promise<ChargeResult>;
  refund(request: RefundRequest): Promise<RefundResult>;
}
