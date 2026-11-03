import type { CookieOptions, NextFunction, Request, Response } from 'express';
import jwt from 'jsonwebtoken';
import { getConfig } from '../config.js';
import { forbidden, unauthenticated } from '../lib/errors.js';
import { isObjectId } from '../lib/validate.js';
import { User, type IUser, type Role } from '../models/index.js';

declare global {
  // eslint-disable-next-line @typescript-eslint/no-namespace
  namespace Express {
    interface Request {
      /** Người dùng đăng nhập (đã tải lại từ DB, chắc chắn `active`). */
      user?: IUser;
      /** Id khách từ cookie `tkh_vid`. */
      visitorId?: string;
    }
  }
}

export const SESSION_COOKIE = 'tkh_session';
const SESSION_MAX_AGE_MS = 7 * 24 * 3600 * 1000;

function baseCookie(): CookieOptions {
  return { httpOnly: true, sameSite: 'lax', path: '/', secure: getConfig().cookieSecure };
}

export function setSessionCookie(res: Response, user: IUser): void {
  const token = jwt.sign({ sub: user._id.toString(), role: user.role }, getConfig().jwtSecret, {
    algorithm: 'HS256',
    expiresIn: Math.floor(SESSION_MAX_AGE_MS / 1000),
  });
  res.cookie(SESSION_COOKIE, token, { ...baseCookie(), maxAge: SESSION_MAX_AGE_MS });
}

export function clearSessionCookie(res: Response): void {
  res.clearCookie(SESSION_COOKIE, baseCookie());
}

/** Đọc cookie phiên, tải lại user từ DB. Token hỏng / user không còn / bị khoá ⇒ coi như chưa đăng nhập + xoá cookie. */
export async function loadUser(req: Request, res: Response, next: NextFunction): Promise<void> {
  const token: unknown = req.cookies?.[SESSION_COOKIE];
  if (typeof token !== 'string' || !token) return next();
  let sub: unknown;
  try {
    const payload = jwt.verify(token, getConfig().jwtSecret, { algorithms: ['HS256'] });
    sub = typeof payload === 'object' ? payload.sub : undefined;
  } catch {
    clearSessionCookie(res);
    return next();
  }
  if (!isObjectId(sub)) {
    clearSessionCookie(res);
    return next();
  }
  const user = await User.findById(sub).lean<IUser>();
  if (!user || user.status !== 'active') {
    clearSessionCookie(res);
    return next();
  }
  req.user = user;
  next();
}

export function requireAuth(req: Request, _res: Response, next: NextFunction): void {
  if (!req.user) return next(unauthenticated());
  next();
}

export function requireRole(...roles: Role[]) {
  return (req: Request, _res: Response, next: NextFunction): void => {
    if (!req.user) return next(unauthenticated());
    if (!roles.includes(req.user.role)) return next(forbidden());
    next();
  };
}

/** Lấy user đã đăng nhập (sau requireAuth). */
export function currentUser(req: Request): IUser {
  if (!req.user) throw unauthenticated();
  return req.user;
}
