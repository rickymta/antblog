import { randomUUID } from 'node:crypto';
import type { NextFunction, Request, Response } from 'express';
import { getConfig } from '../config.js';
import { AppError } from '../lib/errors.js';

export const VISITOR_COOKIE = 'tkh_vid';
const VISITOR_MAX_AGE_MS = 400 * 24 * 3600 * 1000;
const UUID_V4 = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

/** Đặt cookie khách `tkh_vid` nếu request chưa có (hoặc có giá trị không hợp lệ). */
export function visitorCookie(req: Request, res: Response, next: NextFunction): void {
  const existing: unknown = req.cookies?.[VISITOR_COOKIE];
  if (typeof existing === 'string' && UUID_V4.test(existing)) {
    req.visitorId = existing.toLowerCase();
  } else {
    req.visitorId = randomUUID();
    res.cookie(VISITOR_COOKIE, req.visitorId, {
      httpOnly: true,
      sameSite: 'lax',
      path: '/',
      secure: getConfig().isProduction,
      maxAge: VISITOR_MAX_AGE_MS,
    });
  }
  next();
}

const WRITE_METHODS = new Set(['POST', 'PUT', 'PATCH', 'DELETE']);

/** Request ghi phải có `Content-Type: application/json` (kể cả thân rỗng) — chống CSRF dạng form. */
export function requireJsonContentType(req: Request, _res: Response, next: NextFunction): void {
  if (!WRITE_METHODS.has(req.method)) return next();
  const type = (req.headers['content-type'] ?? '').split(';')[0]!.trim().toLowerCase();
  if (type !== 'application/json') {
    return next(new AppError(415, 'UNSUPPORTED_MEDIA_TYPE', 'Request ghi phải gửi kèm Content-Type: application/json.'));
  }
  next();
}

/** Express 5 để `req.body` là undefined khi không có thân — chuẩn hoá về `{}`. */
export function defaultBody(req: Request, _res: Response, next: NextFunction): void {
  if (req.body === undefined) req.body = {};
  next();
}

export function notFoundHandler(_req: Request, _res: Response, next: NextFunction): void {
  next(new AppError(404, 'NOT_FOUND', 'Không tìm thấy đường dẫn API.'));
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function errorHandler(err: unknown, _req: Request, res: Response, _next: NextFunction): void {
  let appErr: AppError;
  if (err instanceof AppError) {
    appErr = err;
  } else if (isBodyParserError(err, 'entity.parse.failed')) {
    appErr = new AppError(400, 'VALIDATION', 'Thân request không phải JSON hợp lệ.', { fields: {} });
  } else if (isBodyParserError(err, 'entity.too.large')) {
    appErr = new AppError(413, 'PAYLOAD_TOO_LARGE', 'Dữ liệu gửi lên quá lớn.');
  } else if (isBodyParserError(err, 'encoding.unsupported') || isBodyParserError(err, 'charset.unsupported')) {
    appErr = new AppError(415, 'UNSUPPORTED_MEDIA_TYPE', 'Bảng mã của request không được hỗ trợ.');
  } else {
    console.error('[api] Lỗi không xử lý được:', err);
    appErr = new AppError(500, 'INTERNAL', 'Đã có lỗi xảy ra ở máy chủ. Vui lòng thử lại sau.');
  }
  if (res.headersSent) return;
  const body: { error: { code: string; message: string; details?: Record<string, unknown> } } = {
    error: { code: appErr.code, message: appErr.message },
  };
  if (appErr.details) body.error.details = appErr.details;
  res.status(appErr.status).json(body);
}

function isBodyParserError(err: unknown, type: string): boolean {
  return typeof err === 'object' && err !== null && (err as { type?: unknown }).type === type;
}
