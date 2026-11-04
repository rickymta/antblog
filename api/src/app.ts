import { mkdirSync } from 'node:fs';
import cookieParser from 'cookie-parser';
import express, { type Express } from 'express';
import helmet from 'helmet';
import { getConfig } from './config.js';
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
  // Máy dev: request đi qua proxy Vite cục bộ ⇒ tin X-Forwarded-For từ loopback (mặc định).
  // Docker: đặt TRUST_PROXY = subnet của mạng compose để tin nginx.
  app.set('trust proxy', getConfig().trustProxy);
  app.set('etag', false);
  app.use(helmet());

  const api = express.Router();
  api.use((req, res, next) => {
    // GET: `private, no-cache` — không chia sẻ, luôn hỏi lại server, nhưng không chặn back/forward cache của trình duyệt
    // như `no-store`. Ghi: `no-store`.
    res.set('Cache-Control', req.method === 'GET' || req.method === 'HEAD' ? 'private, no-cache' : 'no-store');
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
  app.use('/uploads', uploadsStatic());
  app.use(notFoundHandler);
  app.use(errorHandler);
  return app;
}

/**
 * Phục vụ ảnh thư viện media từ UPLOAD_DIR (hợp đồng §8.2). Docker: nginx đọc thẳng volume, nhánh này dành cho dev.
 * Tên tệp ngẫu nhiên + không bao giờ ghi đè ⇒ cache vĩnh viễn. Không liệt kê thư mục, không phục vụ dotfile.
 */
function uploadsStatic(): express.RequestHandler {
  const dir = getConfig().uploadDir;
  try {
    mkdirSync(dir, { recursive: true });
  } catch (err) {
    // Không tạo được thư mục (quyền) ⇒ API vẫn chạy, chỉ tải ảnh lên sẽ lỗi — báo rõ trong log.
    console.error(`[api] Không tạo được UPLOAD_DIR ${dir}:`, err instanceof Error ? err.message : err);
  }
  return express.static(dir, {
    index: false,
    redirect: false,
    dotfiles: 'deny',
    fallthrough: true,
    etag: true,
    lastModified: true,
    maxAge: '365d',
    immutable: true,
    setHeaders: (res) => {
      res.setHeader('X-Content-Type-Options', 'nosniff');
      res.setHeader('Content-Security-Policy', "default-src 'none'; sandbox");
      // Ảnh công khai, trang đọc và trang quản trị ở hai tên miền con ⇒ cho phép nhúng chéo.
      res.setHeader('Cross-Origin-Resource-Policy', 'cross-origin');
    },
  });
}
