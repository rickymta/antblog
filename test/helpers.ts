import bcrypt from 'bcryptjs';
import jwt from 'jsonwebtoken';
import mongoose, { type Types } from 'mongoose';
import { afterAll, beforeAll } from 'vitest';
import { createApp, type AppOptions } from '../src/app.js';
import { getConfig } from '../src/config.js';
import { connectDb, ensureIndexes } from '../src/db.js';
import { Article, Category, User, type IArticle, type ICategory, type IUser, type Role } from '../src/models/index.js';
import { computeReadingMinutes } from '../src/lib/text.js';

export const TEST_PASSWORD = 'mat-khau-test-123';
const TEST_HASH = bcrypt.hashSync(TEST_PASSWORD, 4);

/** Kết nối DB test sạch cho một file test, dọn sạch sau cùng. */
export function useTestDb(): void {
  beforeAll(async () => {
    const uri = getConfig().mongoUri;
    if (!/_test(\?|$)/.test(uri)) throw new Error(`Từ chối chạy test trên DB không phải *_test: ${uri}`);
    if (mongoose.connection.readyState !== 1) await connectDb(uri);
    await mongoose.connection.db!.dropDatabase();
    await ensureIndexes();
  });
  afterAll(async () => {
    await mongoose.connection.db?.dropDatabase();
    await mongoose.disconnect();
  });
}

export async function resetDb(): Promise<void> {
  await Promise.all(Object.values(mongoose.connection.collections).map((c) => c.deleteMany({})));
}

export function makeApp(options: AppOptions = {}) {
  return createApp({ authRateLimit: 10_000, commentRateLimit: 10_000, ...options });
}

let seq = 0;
export async function createUser(role: Role = 'reader', extra: Partial<IUser> = {}): Promise<IUser> {
  seq++;
  const doc = await User.create({
    email: `user${seq}-${Date.now()}@test.local`,
    passwordHash: TEST_HASH,
    displayName: `Người dùng ${seq}`,
    role,
    status: 'active',
    ...extra,
  });
  return doc.toObject();
}

/** Header Cookie phiên đăng nhập hợp lệ cho user (không qua /auth/login để tránh giới hạn tần suất). */
export function sessionCookie(user: IUser): string {
  const token = jwt.sign({ sub: user._id.toString(), role: user.role }, getConfig().jwtSecret, { algorithm: 'HS256', expiresIn: 3600 });
  return `tkh_session=${token}`;
}

export function visitorCookie(id: string): string {
  return `tkh_vid=${id}`;
}

export async function createCategory(extra: Partial<ICategory> = {}): Promise<ICategory> {
  seq++;
  const doc = await Category.create({
    slug: `chu-de-${seq}`,
    name: { vi: `Chủ đề ${seq}`, en: `Category ${seq}` },
    description: { vi: 'Mô tả', en: 'Description' },
    icon: 'database',
    color: '#336699',
    order: seq,
    ...extra,
  });
  return doc.toObject();
}

export async function createArticle(
  category: ICategory,
  author: IUser,
  extra: Partial<Omit<IArticle, 'stats'>> & { stats?: Partial<IArticle['stats']> } = {},
): Promise<IArticle> {
  seq++;
  const content = extra.content ?? `Nội dung bài viết số ${seq}. `.repeat(20);
  const doc = await Article.create({
    slug: `bai-viet-${seq}`,
    title: `Bài viết số ${seq}`,
    summary: `Tóm tắt bài ${seq}`,
    language: 'vi',
    categoryId: category._id,
    level: 'beginner',
    tags: [],
    authorId: author._id,
    status: 'published',
    publishedAt: new Date(Date.now() - (1000 - seq) * 1000),
    readingMinutes: computeReadingMinutes(content),
    ...extra,
    content,
  });
  return doc.toObject();
}

export const idStr = (id: Types.ObjectId) => id.toString();

/** Kiểm tra đệ quy không có `_id` / `__v` / `null` trong JSON trả về. */
export function assertCleanJson(value: unknown, path = '$'): void {
  if (value === null) throw new Error(`Giá trị null ở ${path}`);
  if (Array.isArray(value)) return value.forEach((v, i) => assertCleanJson(v, `${path}[${i}]`));
  if (typeof value === 'object' && value) {
    for (const [k, v] of Object.entries(value)) {
      if (k === '_id' || k === '__v') throw new Error(`Khoá ${k} ở ${path}`);
      assertCleanJson(v, `${path}.${k}`);
    }
  }
}
