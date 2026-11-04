import type { ZodError } from 'zod';
import { DomainError, type ErrorCode } from '../domain/errors';

/** Mã lỗi chỉ có ở lớp HTTP (không phải quy tắc nghiệp vụ). */
export type HttpOnlyCode =
  | 'INVALID_JSON'
  | 'PAYLOAD_TOO_LARGE'
  | 'NOT_FOUND'
  | 'RATE_LIMITED'
  | 'IDEMPOTENCY_KEY_REQUIRED'
  | 'IDEMPOTENCY_KEY_REUSED'
  | 'IDEMPOTENCY_IN_PROGRESS'
  | 'INTERNAL_ERROR';

export type ApiErrorCode = ErrorCode | HttpOnlyCode;

/** Bảng mã lỗi → HTTP status (SPEC R-API-06). */
export const STATUS_BY_CODE: Record<ApiErrorCode, number> = {
  VALIDATION_ERROR: 400,
  INVALID_JSON: 400,
  IDEMPOTENCY_KEY_REQUIRED: 400,
  UNAUTHENTICATED: 401,
  FORBIDDEN: 403,
  TRIP_NOT_FOUND: 404,
  NOT_FOUND: 404,
  INVALID_TRANSITION: 409,
  TRIP_ALREADY_TAKEN: 409,
  DRIVER_OFFLINE: 409,
  DRIVER_BUSY: 409,
  QUOTE_ALREADY_USED: 409,
  IDEMPOTENCY_IN_PROGRESS: 409,
  PAYLOAD_TOO_LARGE: 413,
  CITY_NOT_SUPPORTED: 422,
  DISTANCE_OUT_OF_RANGE: 422,
  WEIGHT_OUT_OF_RANGE: 422,
  VEHICLE_TOO_SMALL: 422,
  SUBTOTAL_TOO_LOW: 422,
  CITY_MISMATCH: 422,
  VEHICLE_MISMATCH: 422,
  VOUCHER_NOT_FOUND: 422,
  VOUCHER_EXPIRED: 422,
  VOUCHER_NOT_APPLICABLE: 422,
  VOUCHER_MIN_AMOUNT: 422,
  VOUCHER_USAGE_LIMIT: 422,
  QUOTE_NOT_FOUND: 422,
  QUOTE_EXPIRED: 422,
  IDEMPOTENCY_KEY_REUSED: 422,
  RATE_LIMITED: 429,
  INTERNAL_ERROR: 500,
};

export class ApiError extends Error {
  readonly code: ApiErrorCode;
  readonly details?: unknown;

  constructor(code: ApiErrorCode, message: string, details?: unknown) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
    this.details = details;
  }
}

export interface JsonResponse {
  status: number;
  body: unknown;
}

export function validationError(error: ZodError): ApiError {
  const details = error.issues.map((issue) => ({ path: issue.path.join('.'), message: issue.message }));
  return new ApiError('VALIDATION_ERROR', 'Dữ liệu gửi lên không hợp lệ.', details);
}

/** Đổi mọi lỗi thành phản hồi JSON `{ error: { code, message } }` (SPEC R-API-02). */
export function toErrorResponse(error: unknown): JsonResponse {
  if (error instanceof DomainError || error instanceof ApiError) {
    const body: { code: string; message: string; details?: unknown } = { code: error.code, message: error.message };
    if (error instanceof ApiError && error.details !== undefined) body.details = error.details;
    return { status: STATUS_BY_CODE[error.code], body: { error: body } };
  }
  const bodyParser = error as { type?: string };
  if (bodyParser?.type === 'entity.parse.failed') {
    return { status: 400, body: { error: { code: 'INVALID_JSON', message: 'Thân yêu cầu không phải JSON hợp lệ.' } } };
  }
  if (bodyParser?.type === 'entity.too.large') {
    return { status: 413, body: { error: { code: 'PAYLOAD_TOO_LARGE', message: 'Thân yêu cầu vượt quá 10 KB.' } } };
  }
  return { status: 500, body: { error: { code: 'INTERNAL_ERROR', message: 'Lỗi hệ thống, vui lòng thử lại sau.' } } };
}
