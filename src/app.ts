import cookieParser from 'cookie-parser';
import express, { type Express } from 'express';
import helmet from 'helmet';
import './lib/validate.js';
import { loadUser } from './middleware/auth.js';
import {
  defaultBody,
  errorHandler,
  notFoundHandler,
  requireJsonContentType,
  visitorCookie,
} from './middleware/common.js';
import { authLimiter, commentLimiter } from './middleware/rateLimit.js';
import { adminRouter } from './routes/admin.js';
import { articlesRouter } from './routes/articles.js';
import { authRouter } from './routes/auth.js';
import { commentsRouter } from './routes/comments.js';
import { publicRouter } from './routes/public.js';

export interface AppOptions {
  /** Số lần đăng nhập/đăng ký mỗi IP trong 15 phút (mặc định 10). */
  authRateLimit?: number;
  /** Số bình luận mỗi người trong 1 phút (mặc định 10). */
  commentRateLimit?: number;
}

export function createApp(options: AppOptions = {}): Express {
  const app = express();
  // API chỉ nghe trên 127.0.0.1, request đi qua proxy Vite cục bộ ⇒ tin X-Forwarded-For từ loopback.
  app.set('trust proxy', 'loopback');
  app.set('etag', false);
  app.use(helmet());

  const api = express.Router();
  api.use((_req, res, next) => {
    res.set('Cache-Control', 'no-store');
    next();
  });
  api.use(requireJsonContentType);
  api.use(express.json({ limit: '2mb', strict: true }));
  api.use(defaultBody);
  api.use(cookieParser());
  api.use(visitorCookie);
  api.use(loadUser);

  api.use('/auth', authRouter(authLimiter(options.authRateLimit ?? 10)));
  api.use(publicRouter());
  api.use(articlesRouter());
  api.use(commentsRouter(commentLimiter(options.commentRateLimit ?? 10)));
  api.use('/admin', adminRouter());

  app.use('/api/v1', api);
  app.use(notFoundHandler);
  app.use(errorHandler);
  return app;
}
