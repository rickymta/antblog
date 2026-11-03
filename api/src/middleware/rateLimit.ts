import type { NextFunction, Request, RequestHandler, Response } from 'express';
import { rateLimit } from 'express-rate-limit';
import { AppError } from '../lib/errors.js';

function handler(req: Request, _res: Response, next: NextFunction): void {
  next(new AppError(429, 'RATE_LIMITED', 'Bạn thao tác quá nhanh. Vui lòng thử lại sau ít phút.'));
}

/** `/auth/login` + `/auth/register`: mặc định 10 lần / 15 phút / IP (dùng chung một bộ đếm). */
export function authLimiter(max: number): RequestHandler {
  return rateLimit({
    windowMs: 15 * 60 * 1000,
    limit: max,
    standardHeaders: 'draft-8',
    legacyHeaders: false,
    handler,
  });
}

/** Viết bình luận: mặc định 10 / phút / người dùng (đặt SAU requireAuth). */
export function commentLimiter(max: number): RequestHandler {
  return rateLimit({
    windowMs: 60 * 1000,
    limit: max,
    standardHeaders: 'draft-8',
    legacyHeaders: false,
    keyGenerator: (req) => `user:${req.user?._id.toString() ?? 'anon'}`,
    handler,
  });
}
