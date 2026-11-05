import { validateCategoryParent, categorySubtreeIds } from '../services/categories.js';
import { Router, type Request } from 'express';
import { Types } from 'mongoose';
import { z } from 'zod';
import { conflict, forbidden, isDuplicateKeyError, notFound, validation } from '../lib/errors.js';
import { computeReadingMinutes, escapeRegex, SLUG_PATTERN } from '../lib/text.js';
import { isObjectId, objectIdSchema, optionalQueryString, pagingSchema, parse, queryInt } from '../lib/validate.js';
import { currentUser, requireRole } from '../middleware/auth.js';
import {
  Article,
  ARTICLE_STATUSES,
  Category,
  CATEGORY_ICONS,
  Comment,
  COMMENT_STATUSES,
  LANGUAGES,
  LearningPath,
  LEVELS,
  PATH_STATUSES,
  ROLES,
  Subscriber,
  User,
  USER_STATUSES,
  type IArticle,
  type ICategory,
  type IComment,
  type ILearningPath,
  type ISubscriber,
  type IUser,
} from '../models/index.js';
import { resolveArticleSlug, syncLearningPathArticles } from '../services/articles.js';
import { setCommentVisibility, softDeleteComment } from '../services/comments.js';
import { recountCategories, withLiveArticleCounts } from '../services/counters.js';
import { mediaRouter } from './media.js';
import {
  compact,
  toAdminArticles,
  toAdminLearningPaths,
  toCategory,
  toComments,
  toUser,
} from '../services/serializers.js';
import { statsOverview } from '../services/stats.js';

const emptyToUndefined = (v: unknown) => (v === '' || v === null ? undefined : v);

const slugSchema = z
  .string()
  .trim()
  .min(1)
  .max(120)
  .regex(SLUG_PATTERN, { message: 'Slug chỉ gồm a-z, 0-9 và dấu gạch ngang' });

/** Đường dẫn ảnh trong thư viện media cùng origin: `/uploads/...` (không `..`, không ký tự lạ). */
const UPLOAD_PATH = /^\/uploads\/(?!.*\.\.)[A-Za-z0-9._\/-]+$/;

/** URL ảnh: http(s) tuyệt đối HOẶC đường dẫn `/uploads/...` của thư viện media. */
const imageUrl = z
  .string()
  .trim()
  .max(500)
  .refine((v) => /^https?:\/\/\S+$/i.test(v) || UPLOAD_PATH.test(v), {
    message: 'Phải là đường dẫn http(s) hoặc ảnh trong thư viện (/uploads/...)',
  });

/** Chuỗi tuỳ chọn: rỗng/khoảng trắng ⇒ vắng (hợp đồng §8.1). */
const blankToUndefined = (v: unknown) => (v === null || (typeof v === 'string' && v.trim() === '') ? undefined : v);

const seoSchema = z.object({
  title: z.preprocess(blankToUndefined, z.string().trim().max(70).optional()),
  description: z.preprocess(blankToUndefined, z.string().trim().max(170).optional()),
  ogImageUrl: z.preprocess(blankToUndefined, imageUrl.optional()),
  noindex: z.preprocess((v) => (v === null ? undefined : v), z.boolean().optional()),
});

const articleInputSchema = z.object({
  title: z.string().trim().min(3).max(200),
  slug: z.preprocess(emptyToUndefined, slugSchema.optional()),
  summary: z.string().trim().max(400),
  content: z.string().max(500_000),
  coverImageUrl: z.preprocess(blankToUndefined, imageUrl.optional()),
  seo: z.preprocess((v) => (v === null ? undefined : v), seoSchema.optional()),
  language: z.enum(LANGUAGES),
  categoryId: objectIdSchema,
  level: z.enum(LEVELS),
  tags: z
    .array(z.string().trim().toLowerCase().min(1).max(40))
    .max(10)
    .transform((tags) => [...new Set(tags)]),
});
type ArticleInput = z.output<typeof articleInputSchema>;

const localizedSchema = (max: number, min = 0) => z.object({ vi: z.string().trim().min(min).max(max), en: z.string().trim().min(min).max(max) });

const categoryInputSchema = z.object({
  parentId: objectIdSchema.nullable().optional(),
  slug: slugSchema,
  name: localizedSchema(100, 1),
  description: localizedSchema(500),
  icon: z.enum(CATEGORY_ICONS),
  color: z.string().trim().regex(/^#[0-9a-fA-F]{6}$/, { message: 'Màu phải có dạng #RRGGBB' }),
  order: z.number().int().min(-100000).max(100000),
});

const learningPathInputSchema = z.object({
  slug: slugSchema,
  title: z.string().trim().min(3).max(200),
  description: z.string().trim().max(2000),
  level: z.enum(LEVELS),
  categoryId: z.preprocess(emptyToUndefined, objectIdSchema.optional()),
  articleIds: z
    .array(objectIdSchema)
    .max(200)
    .transform((ids) => [...new Set(ids)]),
  status: z.enum(PATH_STATUSES),
});

const oid = (id: string) => new Types.ObjectId(id);

function idParam(req: Request, what: string): Types.ObjectId {
  const id = String(req.params.id);
  if (!isObjectId(id)) throw notFound(`Không tìm thấy ${what}.`);
  return oid(id);
}

async function ensureCategory(categoryId: string): Promise<void> {
  if (!(await Category.exists({ _id: categoryId }))) throw validation({ categoryId: 'Chủ đề không tồn tại' });
}

export function adminRouter(): Router {
  const r = Router();
  const adminOnly = requireRole('admin');
  r.use(requireRole('admin', 'author'));

  // ---------- Thống kê ----------
  r.get('/stats/overview', adminOnly, async (req, res) => {
    const { days } = parse(z.object({ days: queryInt(7, 180, 30) }), req.query);
    res.json(await statsOverview(days));
  });

  // ---------- Bài viết (admin: mọi bài; author: chỉ bài của mình) ----------
  async function loadArticleFor(req: Request): Promise<IArticle> {
    const id = idParam(req, 'bài viết');
    const article = await Article.findById(id).lean<IArticle>();
    if (!article) throw notFound('Không tìm thấy bài viết.');
    const user = currentUser(req);
    if (user.role !== 'admin' && !article.authorId.equals(user._id)) {
      throw forbidden('Bạn chỉ quản lý được bài viết do mình là tác giả.');
    }
    return article;
  }

  async function adminArticle(id: Types.ObjectId) {
    const a = await Article.findById(id).lean<IArticle>();
    if (!a) throw notFound('Không tìm thấy bài viết.');
    const [out] = await toAdminArticles([a], true);
    return out;
  }

  /** `seo` sau khi bỏ trường rỗng (`noindex: false` = mặc định ⇒ bỏ). Không còn gì ⇒ undefined (gỡ ghi đè). */
  function seoFields(input: ArticleInput) {
    const s = input.seo;
    if (!s) return undefined;
    const out = compact({ title: s.title, description: s.description, ogImageUrl: s.ogImageUrl, noindex: s.noindex === true ? true : undefined });
    return Object.keys(out).length ? out : undefined;
  }

  function articleFields(input: ArticleInput) {
    return {
      title: input.title,
      summary: input.summary,
      content: input.content,
      language: input.language,
      categoryId: oid(input.categoryId),
      level: input.level,
      tags: input.tags,
      readingMinutes: computeReadingMinutes(input.content),
    };
  }

  r.get('/articles', async (req, res) => {
    const q = parse(
      z.object({
        status: z.preprocess(emptyToUndefined, z.enum([...ARTICLE_STATUSES, 'scheduled']).optional()),
        category: optionalQueryString(100),
        q: optionalQueryString(200),
        ...pagingSchema(12),
      }),
      req.query,
    );
    const user = currentUser(req);
    const filter: Record<string, unknown> = {};
    // Không lọc trạng thái ⇒ ẩn bài đã lưu trữ (DELETE = lưu trữ); `status=archived` để xem chúng.
    // `published` = đang công khai; `scheduled` = đã xuất bản nhưng `publishedAt` ở tương lai (lên lịch).
    const now = new Date();
    if (q.status === 'published') Object.assign(filter, { status: 'published', publishedAt: { $lte: now } });
    else if (q.status === 'scheduled') Object.assign(filter, { status: 'published', publishedAt: { $gt: now } });
    else filter.status = q.status ?? { $ne: 'archived' };
    if (user.role !== 'admin') filter.authorId = user._id;
    if (q.category) {
      const cat = await Category.findOne(isObjectId(q.category) ? { _id: q.category } : { slug: q.category })
        .select({ _id: 1 })
        .lean<ICategory>();
      if (!cat) return void res.json({ items: [], total: 0, page: q.page, pageSize: q.pageSize });
      filter.categoryId = { $in: await categorySubtreeIds(cat._id) };
    }
    if (q.q) {
      const rx = new RegExp(escapeRegex(q.q), 'i');
      filter.$or = [{ title: rx }, { slug: rx }, { tags: q.q.toLowerCase() }];
    }
    const [items, total] = await Promise.all([
      Article.find(filter)
        .select({ content: 0 })
        .sort({ updatedAt: -1, _id: -1 })
        .skip((q.page - 1) * q.pageSize)
        .limit(q.pageSize)
        .lean<IArticle[]>(),
      Article.countDocuments(filter),
    ]);
    res.json({ items: await toAdminArticles(items, false), total, page: q.page, pageSize: q.pageSize });
  });

  r.post('/articles', async (req, res) => {
    const input = parse(articleInputSchema, req.body);
    await ensureCategory(input.categoryId);
    const slug = await resolveArticleSlug({ explicit: input.slug, title: input.title });
    const user = currentUser(req);
    let created;
    try {
      created = await Article.create({
        ...articleFields(input),
        slug,
        coverImageUrl: input.coverImageUrl,
        seo: seoFields(input),
        authorId: user._id,
        status: 'draft',
        // Bài sinh ra trong CMS thuộc quyền CMS — seed không được ghi đè nếu sau này có tệp .md trùng slug.
        editedInCms: true,
      });
    } catch (err) {
      if (isDuplicateKeyError(err)) throw conflict('SLUG_TAKEN', 'Slug này đã được dùng cho bài viết khác.');
      throw err;
    }
    res.status(201).json(await adminArticle(created._id));
  });

  r.get('/articles/:id', async (req, res) => {
    const article = await loadArticleFor(req);
    res.json(await adminArticle(article._id));
  });

  r.put('/articles/:id', async (req, res) => {
    const article = await loadArticleFor(req);
    const input = parse(articleInputSchema, req.body);
    await ensureCategory(input.categoryId);
    // Slug trống khi sửa ⇒ giữ slug hiện tại (không đổi URL bài đã có).
    const slug = input.slug ? await resolveArticleSlug({ explicit: input.slug, title: input.title, excludeId: article._id }) : article.slug;
    const set: Record<string, unknown> = { ...articleFields(input), slug, editedInCms: true };
    const unset: Record<string, 1> = {};
    if (input.coverImageUrl) set.coverImageUrl = input.coverImageUrl;
    else unset.coverImageUrl = 1;
    const seo = seoFields(input);
    if (seo) set.seo = seo;
    else unset.seo = 1;
    const update: Record<string, unknown> = { $set: set };
    if (Object.keys(unset).length) update.$unset = unset;
    try {
      await Article.updateOne({ _id: article._id }, update);
    } catch (err) {
      if (isDuplicateKeyError(err)) throw conflict('SLUG_TAKEN', 'Slug này đã được dùng cho bài viết khác.');
      throw err;
    }
    if (!article.categoryId.equals(oid(input.categoryId))) await recountCategories([article.categoryId, input.categoryId]);
    res.json(await adminArticle(article._id));
  });

  r.post('/articles/:id/publish', async (req, res) => {
    const article = await loadArticleFor(req);
    const { publishAt } = parse(
      z.object({
        publishAt: z.preprocess(
          blankToUndefined,
          z.iso.datetime({ offset: true, message: 'Thời điểm phải là ISO 8601 (vd 2026-11-05T02:00:00.000Z)' }).optional(),
        ),
      }),
      req.body,
    );
    const now = new Date();
    let publishedAt: Date;
    if (publishAt) {
      publishedAt = new Date(publishAt);
      if (!(publishedAt.getTime() > now.getTime())) {
        throw validation({ publishAt: 'Thời điểm lên lịch phải ở tương lai.' });
      }
    } else {
      // Xuất bản ngay: giữ ngày xuất bản cũ (đăng lại sau khi gỡ); bài đang lên lịch ⇒ đăng NGAY bây giờ.
      publishedAt = article.publishedAt && article.publishedAt.getTime() <= now.getTime() ? article.publishedAt : now;
    }
    await Article.updateOne({ _id: article._id }, { $set: { status: 'published', publishedAt } });
    await recountCategories([article.categoryId]);
    res.json(await adminArticle(article._id));
  });

  r.post('/articles/:id/unpublish', async (req, res) => {
    const article = await loadArticleFor(req);
    await Article.updateOne({ _id: article._id }, { $set: { status: 'draft' } });
    await recountCategories([article.categoryId]);
    res.json(await adminArticle(article._id));
  });

  r.delete('/articles/:id', async (req, res) => {
    const article = await loadArticleFor(req);
    await Article.updateOne({ _id: article._id }, { $set: { status: 'archived' } });
    await recountCategories([article.categoryId]);
    res.status(204).end();
  });

  // ---------- Thư viện media (admin + author; author chỉ ảnh của mình) ----------
  r.use('/media', mediaRouter());

  // ---------- Mọi mục còn lại: chỉ admin ----------
  r.use(adminOnly);

  // ---------- Chủ đề ----------
  r.get('/categories', async (_req, res) => {
    const cats = await Category.find({}).sort({ order: 1, slug: 1 }).lean<ICategory[]>();
    res.json((await withLiveArticleCounts(cats)).map(toCategory));
  });

  r.post('/categories', async (req, res) => {
    const input = parse(categoryInputSchema, req.body);
    if (await Category.exists({ slug: input.slug })) throw conflict('SLUG_TAKEN', 'Slug chủ đề đã tồn tại.');
    await validateCategoryParent('new-category', input.parentId ?? null);
    try {
      const created = await Category.create({ ...input, articleCount: 0 });
      res.status(201).json(toCategory(created.toObject()));
    } catch (err) {
      if (isDuplicateKeyError(err)) throw conflict('SLUG_TAKEN', 'Slug chủ đề đã tồn tại.');
      throw err;
    }
  });

  r.put('/categories/:id', async (req, res) => {
    const id = idParam(req, 'chủ đề');
    const input = parse(categoryInputSchema, req.body);
    if (await Category.exists({ slug: input.slug, _id: { $ne: id } })) throw conflict('SLUG_TAKEN', 'Slug chủ đề đã tồn tại.');
    if (input.parentId !== undefined) await validateCategoryParent(String(id), input.parentId);
    let updated: ICategory | null;
    try {
      updated = await Category.findByIdAndUpdate(id, { $set: input }, { new: true }).lean<ICategory>();
    } catch (err) {
      if (isDuplicateKeyError(err)) throw conflict('SLUG_TAKEN', 'Slug chủ đề đã tồn tại.');
      throw err;
    }
    if (!updated) throw notFound('Không tìm thấy chủ đề.');
    res.json(toCategory(updated));
  });

  r.delete('/categories/:id', async (req, res) => {
    const id = idParam(req, 'chủ đề');
    if (!(await Category.exists({ _id: id }))) throw notFound('Không tìm thấy chủ đề.');
    if (await Category.exists({ parentId: id })) throw conflict('CATEGORY_HAS_CHILDREN', 'Hãy chuyển hoặc xoá danh mục con trước khi xoá danh mục này.');
    if (await Article.exists({ categoryId: id })) {
      throw conflict('CATEGORY_IN_USE', 'Chủ đề vẫn còn bài viết (kể cả bản nháp/lưu trữ) — hãy chuyển bài sang chủ đề khác trước.');
    }
    await Category.deleteOne({ _id: id });
    await LearningPath.updateMany({ categoryId: id }, { $unset: { categoryId: 1 } });
    res.status(204).end();
  });

  // ---------- Lộ trình học ----------
  async function validatePathInput(input: z.output<typeof learningPathInputSchema>, excludeId?: Types.ObjectId) {
    const dupFilter: Record<string, unknown> = { slug: input.slug };
    if (excludeId) dupFilter._id = { $ne: excludeId };
    if (await LearningPath.exists(dupFilter)) throw conflict('SLUG_TAKEN', 'Slug lộ trình đã tồn tại.');
    if (input.categoryId) await ensureCategory(input.categoryId);
    if (input.articleIds.length) {
      const n = await Article.countDocuments({ _id: { $in: input.articleIds } });
      if (n !== input.articleIds.length) throw validation({ articleIds: 'Có bài viết không tồn tại trong danh sách' });
    }
  }

  async function adminPath(id: Types.ObjectId) {
    const p = await LearningPath.findById(id).lean<ILearningPath>();
    if (!p) throw notFound('Không tìm thấy lộ trình học.');
    const [out] = await toAdminLearningPaths([p]);
    return out;
  }

  r.get('/learning-paths', async (_req, res) => {
    const paths = await LearningPath.find({}).sort({ createdAt: 1, _id: 1 }).lean<ILearningPath[]>();
    res.json(await toAdminLearningPaths(paths));
  });

  r.post('/learning-paths', async (req, res) => {
    const input = parse(learningPathInputSchema, req.body);
    await validatePathInput(input);
    const articleIds = input.articleIds.map(oid);
    let created;
    try {
      created = await LearningPath.create({
        slug: input.slug,
        title: input.title,
        description: input.description,
        level: input.level,
        categoryId: input.categoryId ? oid(input.categoryId) : undefined,
        articleIds,
        status: input.status,
      });
    } catch (err) {
      if (isDuplicateKeyError(err)) throw conflict('SLUG_TAKEN', 'Slug lộ trình đã tồn tại.');
      throw err;
    }
    await syncLearningPathArticles(created._id, articleIds, []);
    res.status(201).json(await adminPath(created._id));
  });

  r.get('/learning-paths/:id', async (req, res) => {
    res.json(await adminPath(idParam(req, 'lộ trình học')));
  });

  r.put('/learning-paths/:id', async (req, res) => {
    const id = idParam(req, 'lộ trình học');
    const existing = await LearningPath.findById(id).lean<ILearningPath>();
    if (!existing) throw notFound('Không tìm thấy lộ trình học.');
    const input = parse(learningPathInputSchema, req.body);
    await validatePathInput(input, id);
    const articleIds = input.articleIds.map(oid);
    const update: Record<string, unknown> = {
      $set: {
        slug: input.slug,
        title: input.title,
        description: input.description,
        level: input.level,
        articleIds,
        status: input.status,
        ...(input.categoryId ? { categoryId: oid(input.categoryId) } : {}),
      },
    };
    if (!input.categoryId) update.$unset = { categoryId: 1 };
    try {
      await LearningPath.updateOne({ _id: id }, update);
    } catch (err) {
      if (isDuplicateKeyError(err)) throw conflict('SLUG_TAKEN', 'Slug lộ trình đã tồn tại.');
      throw err;
    }
    await syncLearningPathArticles(id, articleIds, existing.articleIds);
    res.json(await adminPath(id));
  });

  r.delete('/learning-paths/:id', async (req, res) => {
    const id = idParam(req, 'lộ trình học');
    const existing = await LearningPath.findByIdAndDelete(id).lean<ILearningPath>();
    if (!existing) throw notFound('Không tìm thấy lộ trình học.');
    await Article.updateMany({ learningPathId: id }, { $unset: { learningPathId: 1 } });
    res.status(204).end();
  });

  // ---------- Bình luận ----------
  async function toAdminComments(comments: IComment[], viewer: IUser) {
    const out = await toComments(comments, viewer, 'admin');
    const articles = await Article.find({ _id: { $in: [...new Set(comments.map((c) => c.articleId.toString()))] } })
      .select({ slug: 1, title: 1 })
      .lean<Pick<IArticle, '_id' | 'slug' | 'title'>[]>();
    const byId = new Map(articles.map((a) => [a._id.toString(), a]));
    return out.map((c) => {
      const a = byId.get(c.articleId);
      return { ...c, article: a ? { id: a._id.toString(), slug: a.slug, title: a.title } : { id: c.articleId, slug: '', title: '' } };
    });
  }

  r.get('/comments', async (req, res) => {
    const q = parse(
      z.object({
        status: z.preprocess(emptyToUndefined, z.enum(COMMENT_STATUSES).optional()),
        articleId: z.preprocess(emptyToUndefined, objectIdSchema.optional()),
        q: optionalQueryString(200),
        ...pagingSchema(12),
      }),
      req.query,
    );
    const filter: Record<string, unknown> = {};
    if (q.status) filter.status = q.status;
    if (q.articleId) filter.articleId = oid(q.articleId);
    if (q.q) filter.body = new RegExp(escapeRegex(q.q), 'i');
    const [items, total] = await Promise.all([
      Comment.find(filter)
        .sort({ createdAt: -1, _id: -1 })
        .skip((q.page - 1) * q.pageSize)
        .limit(q.pageSize)
        .lean<IComment[]>(),
      Comment.countDocuments(filter),
    ]);
    res.json({ items: await toAdminComments(items, currentUser(req)), total, page: q.page, pageSize: q.pageSize });
  });

  r.patch('/comments/:id', async (req, res) => {
    const id = idParam(req, 'bình luận');
    const { status } = parse(z.object({ status: z.enum(['visible', 'hidden']) }), req.body);
    const comment = await Comment.findById(id).lean<IComment>();
    if (!comment) throw notFound('Không tìm thấy bình luận.');
    if (comment.status === 'deleted') throw conflict('COMMENT_DELETED', 'Bình luận đã bị xoá, không thể đổi trạng thái.');
    const updated = (await setCommentVisibility(id, status)) ?? (await Comment.findById(id).lean<IComment>());
    const [out] = await toAdminComments([updated!], currentUser(req));
    res.json(out);
  });

  r.delete('/comments/:id', async (req, res) => {
    const id = idParam(req, 'bình luận');
    if (!(await Comment.exists({ _id: id }))) throw notFound('Không tìm thấy bình luận.');
    await softDeleteComment(id);
    res.status(204).end();
  });

  // ---------- Người dùng ----------
  async function toAdminUsers(users: IUser[]) {
    const counts = await Comment.aggregate<{ _id: Types.ObjectId; n: number }>([
      { $match: { authorId: { $in: users.map((u) => u._id) }, status: 'visible' } },
      { $group: { _id: '$authorId', n: { $sum: 1 } } },
    ]);
    const byId = new Map(counts.map((c) => [c._id.toString(), c.n]));
    return users.map((u) =>
      compact({
        ...toUser(u),
        commentCount: byId.get(u._id.toString()) ?? 0,
        lastLoginAt: u.lastLoginAt ? new Date(u.lastLoginAt).toISOString() : undefined,
      }),
    );
  }

  r.get('/users', async (req, res) => {
    const q = parse(
      z.object({
        q: optionalQueryString(200),
        role: z.preprocess(emptyToUndefined, z.enum(ROLES).optional()),
        status: z.preprocess(emptyToUndefined, z.enum(USER_STATUSES).optional()),
        ...pagingSchema(12),
      }),
      req.query,
    );
    const filter: Record<string, unknown> = {};
    if (q.role) filter.role = q.role;
    if (q.status) filter.status = q.status;
    if (q.q) {
      const rx = new RegExp(escapeRegex(q.q), 'i');
      filter.$or = [{ email: rx }, { displayName: rx }];
    }
    const [users, total] = await Promise.all([
      User.find(filter)
        .sort({ createdAt: -1, _id: -1 })
        .skip((q.page - 1) * q.pageSize)
        .limit(q.pageSize)
        .lean<IUser[]>(),
      User.countDocuments(filter),
    ]);
    res.json({ items: await toAdminUsers(users), total, page: q.page, pageSize: q.pageSize });
  });

  r.patch('/users/:id', async (req, res) => {
    const id = idParam(req, 'người dùng');
    const input = parse(z.object({ role: z.enum(ROLES).optional(), status: z.enum(USER_STATUSES).optional() }), req.body);
    const me = currentUser(req);
    if (me._id.equals(id) && ((input.role && input.role !== me.role) || input.status === 'locked')) {
      throw conflict('CANNOT_MODIFY_SELF', 'Bạn không thể tự hạ vai trò hoặc tự khoá tài khoản của mình.');
    }
    const set: Record<string, string> = {};
    if (input.role) set.role = input.role;
    if (input.status) set.status = input.status;
    const user = Object.keys(set).length
      ? await User.findByIdAndUpdate(id, { $set: set }, { new: true }).lean<IUser>()
      : await User.findById(id).lean<IUser>();
    if (!user) throw notFound('Không tìm thấy người dùng.');
    const [out] = await toAdminUsers([user]);
    res.json(out);
  });

  // ---------- Đăng ký nhận tin ----------
  r.get('/subscribers', async (req, res) => {
    const q = parse(z.object({ ...pagingSchema(12) }), req.query);
    const [items, total] = await Promise.all([
      Subscriber.find({})
        .sort({ createdAt: -1, _id: -1 })
        .skip((q.page - 1) * q.pageSize)
        .limit(q.pageSize)
        .lean<ISubscriber[]>(),
      Subscriber.countDocuments({}),
    ]);
    res.json({
      items: items.map((s) => ({ id: s._id.toString(), email: s.email, createdAt: new Date(s.createdAt).toISOString() })),
      total,
      page: q.page,
      pageSize: q.pageSize,
    });
  });

  return r;
}
