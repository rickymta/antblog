import { createHash } from 'node:crypto';
import type { Request } from 'express';
import type { IdempotencyStore } from '../app/ports';
import type { Clock } from '../domain/time';
import { ApiError, toErrorResponse, type JsonResponse } from './http-errors';

export const IDEMPOTENCY_HEADER = 'idempotency-key';
const KEY_PATTERN = /^[A-Za-z0-9_.:-]{8,128}$/;

/** JSON chuẩn tắc (khoá sắp xếp) ⇒ cùng nội dung khác thứ tự khoá vẫn cùng vân tay. */
export function canonicalJson(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(',')}]`;
  if (value !== null && typeof value === 'object') {
    const entries = Object.entries(value as Record<string, unknown>).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
    return `{${entries.map(([k, v]) => `${JSON.stringify(k)}:${canonicalJson(v)}`).join(',')}}`;
  }
  return JSON.stringify(value) ?? 'null';
}

export function fingerprint(body: unknown): string {
  return createHash('sha256').update(canonicalJson(body ?? {})).digest('hex');
}

/**
 * Chạy `handler` đúng một lần cho mỗi (scope, Idempotency-Key) — SPEC R-IDEM:
 * cùng khoá + cùng thân ⇒ trả lại phản hồi đã lưu; cùng khoá + khác thân ⇒ 422;
 * đang chạy ⇒ 409; phản hồi 5xx không được lưu (client thử lại được).
 */
export async function runIdempotent(
  req: Request,
  deps: { store: IdempotencyStore; clock: Clock },
  scope: string,
  handler: () => Promise<JsonResponse>,
): Promise<JsonResponse & { replayed: boolean }> {
  const key = req.get(IDEMPOTENCY_HEADER);
  if (!key || !KEY_PATTERN.test(key)) {
    throw new ApiError('IDEMPOTENCY_KEY_REQUIRED', 'Thiếu header Idempotency-Key hợp lệ (8–128 ký tự A–Z, a–z, 0–9, _ . : -).');
  }
  const begin = await deps.store.begin(scope, key, fingerprint(req.body), deps.clock.now());
  if (begin.state === 'mismatch') {
    throw new ApiError('IDEMPOTENCY_KEY_REUSED', 'Idempotency-Key này đã được dùng cho một yêu cầu có nội dung khác.');
  }
  if (begin.state === 'in_progress') {
    throw new ApiError('IDEMPOTENCY_IN_PROGRESS', 'Yêu cầu với Idempotency-Key này đang được xử lý.');
  }
  if (begin.state === 'replay') return { ...begin.response, replayed: true };

  let response: JsonResponse;
  try {
    response = await handler();
  } catch (error) {
    response = toErrorResponse(error);
    if (response.status >= 500) {
      await deps.store.release(scope, key);
      throw error;
    }
  }
  // Lưu dạng JSON thuần để lần trả lại giống hệt lần đầu.
  response = { status: response.status, body: JSON.parse(JSON.stringify(response.body)) };
  await deps.store.complete(scope, key, response);
  return { ...response, replayed: false };
}
