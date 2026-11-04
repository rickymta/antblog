import bcrypt from 'bcryptjs';
import { Router, type RequestHandler } from 'express';
import { z } from 'zod';
import { AppError, conflict, isDuplicateKeyError } from '../lib/errors.js';
import { parse } from '../lib/validate.js';
import { clearSessionCookie, currentUser, requireAuth, setSessionCookie } from '../middleware/auth.js';
import { User, type IUser } from '../models/index.js';
import { toUser } from '../services/serializers.js';

export const BCRYPT_COST = 12;
// Hash giả để so sánh khi email không tồn tại (giảm chênh lệch thời gian phản hồi).
const DUMMY_HASH = bcrypt.hashSync('tkh-dummy-password', 4);

const emailSchema = z.string().trim().toLowerCase().max(254).pipe(z.email());

const registerSchema = z.object({
  email: emailSchema,
  password: z.string().min(8).max(72),
  displayName: z.string().trim().min(2).max(60),
});

const loginSchema = z.object({
  email: z.string().trim().toLowerCase().min(1).max(254),
  password: z.string().min(1).max(200),
});

const httpUrl = z
  .string()
  .trim()
  .max(500)
  .refine((v) => /^https?:\/\/\S+$/i.test(v), { message: 'Phải là đường dẫn http(s) hợp lệ' });

const updateMeSchema = z.object({
  displayName: z.string().trim().min(2).max(60).optional(),
  // Chuỗi rỗng ⇒ xoá trường.
  avatarUrl: z.union([z.literal(''), httpUrl]).optional(),
  bio: z.string().trim().max(300).optional(),
});

export function authRouter(limiter: RequestHandler): Router {
  const r = Router();

  r.post('/register', limiter, async (req, res) => {
    const input = parse(registerSchema, req.body);
    if (await User.exists({ email: input.email })) throw conflict('EMAIL_TAKEN', 'Email này đã được đăng ký.');
    let user: IUser;
    try {
      const doc = await User.create({
        email: input.email,
        passwordHash: await bcrypt.hash(input.password, BCRYPT_COST),
        displayName: input.displayName,
        role: 'reader',
        status: 'active',
        lastLoginAt: new Date(),
      });
      user = doc.toObject();
    } catch (err) {
      if (isDuplicateKeyError(err)) throw conflict('EMAIL_TAKEN', 'Email này đã được đăng ký.');
      throw err;
    }
    setSessionCookie(res, user);
    res.status(201).json({ user: toUser(user) });
  });

  r.post('/login', limiter, async (req, res) => {
    const input = parse(loginSchema, req.body);
    const user = await User.findOne({ email: input.email }).lean<IUser>();
    const ok = await bcrypt.compare(input.password, user?.passwordHash ?? DUMMY_HASH);
    if (!user || !ok) throw new AppError(401, 'INVALID_CREDENTIALS', 'Email hoặc mật khẩu không đúng.');
    if (user.status === 'locked') {
      clearSessionCookie(res);
      throw new AppError(403, 'ACCOUNT_LOCKED', 'Tài khoản đã bị khoá. Vui lòng liên hệ quản trị viên.');
    }
    await User.updateOne({ _id: user._id }, { $set: { lastLoginAt: new Date() } });
    setSessionCookie(res, user);
    res.json({ user: toUser(user) });
  });

  r.post('/logout', (_req, res) => {
    clearSessionCookie(res);
    res.status(204).end();
  });

  r.get('/me', requireAuth, (req, res) => {
    res.json({ user: toUser(currentUser(req)) });
  });

  // Trang đọc dùng endpoint này để biết đã đăng nhập chưa: LUÔN 200, chưa đăng nhập ⇒ { user: null }.
  // (/me trả 401 khi chưa đăng nhập ⇒ trình duyệt ghi lỗi console ở mọi lượt xem của khách — hại điểm chất lượng trang.)
  r.get('/session', (req, res) => {
    res.json({ user: req.user ? toUser(currentUser(req)) : null });
  });

  r.patch('/me', requireAuth, async (req, res) => {
    const input = parse(updateMeSchema, req.body);
    const set: Record<string, string> = {};
    const unset: Record<string, 1> = {};
    if (input.displayName !== undefined) set.displayName = input.displayName;
    for (const key of ['avatarUrl', 'bio'] as const) {
      const v = input[key];
      if (v === undefined) continue;
      if (v === '') unset[key] = 1;
      else set[key] = v;
    }
    const update: Record<string, unknown> = {};
    if (Object.keys(set).length) update.$set = set;
    if (Object.keys(unset).length) update.$unset = unset;
    const me = currentUser(req);
    const user = Object.keys(update).length
      ? await User.findByIdAndUpdate(me._id, update, { new: true }).lean<IUser>()
      : me;
    res.json({ user: toUser(user ?? me) });
  });

  return r;
}
