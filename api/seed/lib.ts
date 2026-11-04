import { existsSync, readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import bcrypt from 'bcryptjs';
import matter from 'gray-matter';
import { Types } from 'mongoose';
import { z } from 'zod';
import { zodFields } from '../src/lib/validate.js';
import { computeReadingMinutes, SLUG_PATTERN } from '../src/lib/text.js';
import { Article, Category, CATEGORY_ICONS, LearningPath, LEVELS, PATH_STATUSES, User, type IUser } from '../src/models/index.js';
import { syncLearningPathArticles } from '../src/services/articles.js';
import { recountCategories } from '../src/services/counters.js';

export interface SeedOptions {
  /** Thư mục chứa `categories.json`, `learning-paths.json`, `content/*.md`. */
  dataDir: string;
  admin: { email?: string; password?: string; name?: string };
  log?: (msg: string) => void;
  /** `--force-content`: ghi đè cả bài đã sửa trong CMS (`editedInCms`). Mặc định false (hợp đồng §8.4). */
  forceContent?: boolean;
}

export interface SeedResult {
  adminCreated: boolean;
  categories: number;
  /** `keptCms`: bài có sẵn đã sửa trong CMS ⇒ giữ nguyên, không ghi đè từ tệp .md. */
  articles: { created: number; updated: number; skipped: number; keptCms: number };
  learningPaths: number;
}

const slug = z.string().trim().regex(SLUG_PATTERN, { message: 'slug không hợp lệ (a-z0-9-)' });
const localized = z.object({ vi: z.string().default(''), en: z.string().default('') });

const categoryFile = z.array(
  z.object({
    slug,
    name: localized,
    description: localized.default({ vi: '', en: '' }),
    icon: z.enum(CATEGORY_ICONS),
    color: z.string().regex(/^#[0-9a-fA-F]{6}$/),
    order: z.number().int().default(0),
  }),
);

const pathFile = z.array(
  z.object({
    slug,
    title: z.string().min(1),
    description: z.string().default(''),
    level: z.enum(LEVELS),
    category: z.string().optional(),
    articles: z.array(z.string()).default([]),
    status: z.enum(PATH_STATUSES).default('published'),
  }),
);

const frontmatter = z.object({
  slug,
  title: z.string().trim().min(1),
  summary: z.string().trim().default(''),
  category: z.string().trim().min(1),
  level: z.enum(LEVELS),
  tags: z
    .array(z.coerce.string())
    .default([])
    .transform((t) => [...new Set(t.map((x) => x.trim().toLowerCase()).filter(Boolean))].slice(0, 10)),
  language: z.enum(['vi', 'en']).default('vi'),
  coverImageUrl: z
    .string()
    .trim()
    .refine((v) => /^https?:\/\/\S+$/i.test(v) || /^\/uploads\/(?!.*\.\.)[A-Za-z0-9._\/-]+$/.test(v), {
      message: 'coverImageUrl phải là URL http(s) hoặc /uploads/...',
    })
    .optional(),
  publishedAt: z.coerce.date().optional(),
});

function readJson(file: string, log: (m: string) => void): unknown | undefined {
  if (!existsSync(file)) {
    log(`[seed] Không có ${path.basename(file)} — bỏ qua.`);
    return undefined;
  }
  return JSON.parse(readFileSync(file, 'utf8')) as unknown;
}

function describe(err: z.ZodError): string {
  return Object.entries(zodFields(err))
    .map(([k, v]) => `${k}: ${v}`)
    .join('; ');
}

async function ensureAdmin(opts: SeedOptions['admin'], log: (m: string) => void): Promise<{ user: IUser; created: boolean }> {
  const email = opts.email?.trim().toLowerCase();
  if (!email) throw new Error('Thiếu SEED_ADMIN_EMAIL');
  const existing = await User.findOne({ email }).lean<IUser>();
  if (existing) {
    // Không bao giờ đặt lại mật khẩu của tài khoản đã có.
    if (existing.role !== 'admin' || existing.status !== 'active') {
      await User.updateOne({ _id: existing._id }, { $set: { role: 'admin', status: 'active' } });
      log(`[seed] Đã nâng ${email} lên admin.`);
    }
    return { user: { ...existing, role: 'admin', status: 'active' }, created: false };
  }
  if (!opts.password || opts.password.length < 8) throw new Error('Thiếu SEED_ADMIN_PASSWORD (≥ 8 ký tự) để tạo admin');
  const doc = await User.create({
    email,
    passwordHash: await bcrypt.hash(opts.password, 12),
    displayName: opts.name?.trim() || 'Quản trị viên',
    role: 'admin',
    status: 'active',
  });
  log(`[seed] Đã tạo tài khoản admin ${email}.`);
  return { user: doc.toObject(), created: true };
}

export async function runSeed(opts: SeedOptions): Promise<SeedResult> {
  const log = opts.log ?? ((m: string) => console.log(m));
  const result: SeedResult = { adminCreated: false, categories: 0, articles: { created: 0, updated: 0, skipped: 0, keptCms: 0 }, learningPaths: 0 };

  const { user: admin, created } = await ensureAdmin(opts.admin, log);
  result.adminCreated = created;

  // ---- Chủ đề ----
  const rawCats = readJson(path.join(opts.dataDir, 'categories.json'), log);
  if (rawCats !== undefined) {
    const parsed = categoryFile.safeParse(rawCats);
    if (!parsed.success) throw new Error(`categories.json không hợp lệ: ${describe(parsed.error)}`);
    for (const c of parsed.data) {
      await Category.updateOne(
        { slug: c.slug },
        { $set: { name: c.name, description: c.description, icon: c.icon, color: c.color.toUpperCase(), order: c.order }, $setOnInsert: { articleCount: 0 } },
        { upsert: true },
      );
    }
    result.categories = parsed.data.length;
    log(`[seed] Chủ đề: ${parsed.data.length}.`);
  }
  const catIdBySlug = new Map((await Category.find({}).select({ slug: 1 }).lean()).map((c) => [c.slug, c._id]));

  // ---- Bài viết ----
  const contentDir = path.join(opts.dataDir, 'content');
  const files = existsSync(contentDir) ? readdirSync(contentDir).filter((f) => f.endsWith('.md')).sort() : [];
  if (!files.length) log(`[seed] Không có bài viết trong ${contentDir} — bỏ qua.`);
  const seenSlugs = new Set<string>();
  for (const file of files) {
    const raw = readFileSync(path.join(contentDir, file), 'utf8');
    const { data, content } = matter(raw);
    const fm = frontmatter.safeParse(data);
    if (!fm.success) {
      log(`[seed] Bỏ qua ${file}: frontmatter không hợp lệ (${describe(fm.error)}).`);
      result.articles.skipped++;
      continue;
    }
    const meta = fm.data;
    if (seenSlugs.has(meta.slug)) {
      log(`[seed] Bỏ qua ${file}: trùng slug "${meta.slug}".`);
      result.articles.skipped++;
      continue;
    }
    seenSlugs.add(meta.slug);
    const categoryId = catIdBySlug.get(meta.category);
    if (!categoryId) {
      log(`[seed] Bỏ qua ${file}: chủ đề "${meta.category}" không tồn tại.`);
      result.articles.skipped++;
      continue;
    }
    const body = content.replace(/^\s*\n/, '');
    const fields = {
      title: meta.title,
      summary: meta.summary.slice(0, 400),
      content: body,
      language: meta.language,
      categoryId,
      level: meta.level,
      tags: meta.tags,
      readingMinutes: computeReadingMinutes(body),
      status: 'published' as const,
      ...(meta.coverImageUrl ? { coverImageUrl: meta.coverImageUrl } : {}),
    };
    const existing = await Article.findOne({ slug: meta.slug }).select({ publishedAt: 1, editedInCms: 1 }).lean();
    if (existing?.editedInCms === true && !opts.forceContent) {
      // Bài đã sửa trong CMS: không đụng nội dung/tiêu đề/trạng thái — CMS là nguồn sự thật của bài này.
      log(`[seed] ${meta.slug}: bỏ qua (đã sửa trong CMS). Dùng --force-content để ghi đè.`);
      result.articles.keptCms++;
    } else if (existing) {
      // Giữ `publishedAt` cũ khi chạy lại. Ép ghi đè ⇒ bài trở lại khớp tệp .md, bỏ cờ đã sửa trong CMS.
      if (existing.editedInCms === true) log(`[seed] ${meta.slug}: ghi đè bản đã sửa trong CMS (--force-content).`);
      await Article.updateOne(
        { _id: existing._id },
        {
          $set: { ...fields, publishedAt: existing.publishedAt ?? meta.publishedAt ?? new Date() },
          $unset: { editedInCms: 1 },
        },
      );
      result.articles.updated++;
    } else {
      await Article.create({ ...fields, slug: meta.slug, authorId: admin._id, publishedAt: meta.publishedAt ?? new Date() });
      result.articles.created++;
    }
  }
  if (files.length) {
    log(
      `[seed] Bài viết: tạo ${result.articles.created}, cập nhật ${result.articles.updated}, ` +
        `giữ bản CMS ${result.articles.keptCms}, bỏ qua ${result.articles.skipped}.`,
    );
  }

  // ---- Lộ trình học ----
  const rawPaths = readJson(path.join(opts.dataDir, 'learning-paths.json'), log);
  if (rawPaths !== undefined) {
    const parsed = pathFile.safeParse(rawPaths);
    if (!parsed.success) throw new Error(`learning-paths.json không hợp lệ: ${describe(parsed.error)}`);
    for (const p of parsed.data) {
      const arts = await Article.find({ slug: { $in: p.articles } }).select({ slug: 1 }).lean();
      const idBySlug = new Map(arts.map((a) => [a.slug, a._id]));
      const missing = p.articles.filter((s) => !idBySlug.has(s));
      if (missing.length) log(`[seed] Lộ trình "${p.slug}": bỏ qua bài không tồn tại: ${missing.join(', ')}.`);
      const articleIds = [...new Set(p.articles.filter((s) => idBySlug.has(s)))].map((s) => idBySlug.get(s)!) as Types.ObjectId[];
      let categoryId: Types.ObjectId | undefined;
      if (p.category) {
        categoryId = catIdBySlug.get(p.category);
        if (!categoryId) log(`[seed] Lộ trình "${p.slug}": chủ đề "${p.category}" không tồn tại — bỏ liên kết chủ đề.`);
      }
      const existing = await LearningPath.findOne({ slug: p.slug }).lean();
      const set = { title: p.title, description: p.description, level: p.level, articleIds, status: p.status };
      let pathId: Types.ObjectId;
      if (existing) {
        await LearningPath.updateOne(
          { _id: existing._id },
          categoryId ? { $set: { ...set, categoryId } } : { $set: set, $unset: { categoryId: 1 } },
        );
        pathId = existing._id;
      } else {
        pathId = (await LearningPath.create({ ...set, slug: p.slug, categoryId }))._id;
      }
      await syncLearningPathArticles(pathId, articleIds, existing?.articleIds ?? []);
    }
    result.learningPaths = parsed.data.length;
    log(`[seed] Lộ trình học: ${parsed.data.length}.`);
  }

  await recountCategories();
  return result;
}
