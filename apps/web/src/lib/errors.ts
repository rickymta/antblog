import { ApiError } from '@tkh/shared';

export const isNotFound = (e: unknown): boolean => e instanceof ApiError && e.status === 404;
export const isUnauthenticated = (e: unknown): boolean => e instanceof ApiError && e.status === 401;

/** Lỗi do máy khách (4xx) — thử lại không giúp gì. */
export const isClientError = (e: unknown): boolean => e instanceof ApiError && e.status >= 400 && e.status < 500;

/** Lỗi trường từ API (400 VALIDATION). */
export function fieldErrors(e: unknown): Record<string, string> {
  return e instanceof ApiError && e.fields ? e.fields : {};
}

/** Câu lỗi từ API (đã là tiếng Việt cho người đọc) — chỉ dùng cho lỗi 4xx có thân JSON; còn lại trả undefined để UI dùng câu chung. */
export function apiMessage(e: unknown): string | undefined {
  return e instanceof ApiError && e.status >= 400 && e.status < 500 && e.code !== 'UNKNOWN' ? e.message : undefined;
}
