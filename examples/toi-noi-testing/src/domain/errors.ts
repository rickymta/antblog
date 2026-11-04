/** Mã lỗi nghiệp vụ (SPEC R-API-06). Lớp API ánh xạ mã → HTTP status. */
export type ErrorCode =
  | 'VALIDATION_ERROR'
  | 'CITY_NOT_SUPPORTED'
  | 'DISTANCE_OUT_OF_RANGE'
  | 'WEIGHT_OUT_OF_RANGE'
  | 'VEHICLE_TOO_SMALL'
  | 'SUBTOTAL_TOO_LOW'
  | 'VOUCHER_NOT_FOUND'
  | 'VOUCHER_EXPIRED'
  | 'VOUCHER_NOT_APPLICABLE'
  | 'VOUCHER_MIN_AMOUNT'
  | 'VOUCHER_USAGE_LIMIT'
  | 'QUOTE_NOT_FOUND'
  | 'QUOTE_EXPIRED'
  | 'QUOTE_ALREADY_USED'
  | 'INVALID_TRANSITION'
  | 'TRIP_NOT_FOUND'
  | 'TRIP_ALREADY_TAKEN'
  | 'DRIVER_OFFLINE'
  | 'DRIVER_BUSY'
  | 'CITY_MISMATCH'
  | 'VEHICLE_MISMATCH'
  | 'UNAUTHENTICATED'
  | 'FORBIDDEN';

/** Lỗi nghiệp vụ có mã ổn định; thông điệp tiếng Việt cho người dùng. */
export class DomainError extends Error {
  readonly code: ErrorCode;

  constructor(code: ErrorCode, message: string) {
    super(message);
    this.name = 'DomainError';
    this.code = code;
  }
}
