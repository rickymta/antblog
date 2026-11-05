import type { Types } from 'mongoose';
import {
  Article,
  Category,
  CommentLike,
  LearningPath,
  User,
  type IArticle,
  type IArticleSeo,
  type ICategory,
  type IMedia,
  type IComment,
  type ILearningPath,
  type IUser,
  type ReactionCounts,
} from '../models/index.js';

/** Bỏ các khoá có giá trị undefined/null (hợp đồng: trường vắng bị bỏ khỏi JSON). */
export function compact<T extends Record<string, unknown>>(obj: T): T {
  for (const k of Object.keys(obj)) {
    if (obj[k] === undefined || obj[k] === null) delete obj[k];
  }
  return obj;
}

const iso = (d: Date | undefined | null) => (d ? new Date(d).toISOString() : undefined);
const idOf = (v: Types.ObjectId | string) => v.toString();

// ---------- Người dùng ----------

export function toUser(u: IUser) {
  return compact({
    id: idOf(u._id),
    email: u.email,
    displayName: u.displayName,
    avatarUrl: u.avatarUrl || undefined,
    bio: u.bio || undefined,
    role: u.role,
    status: u.status,
    createdAt: iso(u.createdAt)!,
  });
}

export type AuthorRef = { id: string; displayName: string; avatarUrl?: string; role: IUser['role'] };

export function toAuthorRef(u: IUser | undefined, fallbackId?: Types.ObjectId | string): AuthorRef {
  if (!u) return { id: fallbackId ? idOf(fallbackId) : '', displayName: 'Người dùng không còn tồn tại', role: 'reader' };
  return compact({ id: idOf(u._id), displayName: u.displayName, avatarUrl: u.avatarUrl || undefined, role: u.role });
}

async function usersById(ids: (Types.ObjectId | undefined)[]): Promise<Map<string, IUser>> {
  const unique = [...new Set(ids.filter((x): x is Types.ObjectId => !!x).map(idOf))];
  if (!unique.length) return new Map();
  const users = await User.find({ _id: { $in: unique } }).lean<IUser[]>();
  return new Map(users.map((u) => [idOf(u._id), u]));
}

// ---------- Chủ đề ----------

export function toCategory(c: ICategory) {
  return {
    id: idOf(c._id),
    slug: c.slug,
    ...(c.parentId ? { parentId: idOf(c.parentId) } : {}),
    name: { vi: c.name?.vi ?? '', en: c.name?.en ?? '' },
    description: { vi: c.description?.vi ?? '', en: c.description?.en ?? '' },
    icon: c.icon,
    color: c.color,
    order: c.order ?? 0,
    articleCount: c.articleCount ?? 0,
  };
}

function toCategoryRef(c: ICategory | undefined, fallbackId: Types.ObjectId) {
  if (!c) return { id: idOf(fallbackId), slug: '', name: { vi: '', en: '' }, icon: 'code' as const, color: '#64748B' };
  return { id: idOf(c._id), slug: c.slug, name: { vi: c.name?.vi ?? '', en: c.name?.en ?? '' }, icon: c.icon, color: c.color };
}

async function categoriesById(ids: (Types.ObjectId | undefined)[]): Promise<Map<string, ICategory>> {
  const unique = [...new Set(ids.filter((x): x is Types.ObjectId => !!x).map(idOf))];
  if (!unique.length) return new Map();
  const cats = await Category.find({ _id: { $in: unique } }).lean<ICategory[]>();
  return new Map(cats.map((c) => [idOf(c._id), c]));
}

// ---------- Bài viết ----------

export function reactionCounts(r: Partial<ReactionCounts> | undefined): ReactionCounts {
  return {
    love: Math.max(0, r?.love ?? 0),
    helpful: Math.max(0, r?.helpful ?? 0),
    insightful: Math.max(0, r?.insightful ?? 0),
    inspiring: Math.max(0, r?.inspiring ?? 0),
  };
}

function statsOf(a: IArticle) {
  return {
    views: Math.max(0, a.stats?.views ?? 0),
    comments: Math.max(0, a.stats?.comments ?? 0),
    reactions: reactionCounts(a.stats?.reactions),
  };
}

function cardFrom(a: IArticle, cats: Map<string, ICategory>, users: Map<string, IUser>) {
  return compact({
    id: idOf(a._id),
    slug: a.slug,
    title: a.title,
    summary: a.summary ?? '',
    coverImageUrl: a.coverImageUrl || undefined,
    language: a.language,
    category: toCategoryRef(cats.get(idOf(a.categoryId)), a.categoryId),
    level: a.level,
    tags: a.tags ?? [],
    author: toAuthorRef(users.get(idOf(a.authorId)), a.authorId),
    readingMinutes: a.readingMinutes ?? 1,
    publishedAt: iso(a.publishedAt),
    updatedAt: iso(a.updatedAt)!,
    stats: statsOf(a),
  });
}

export type ArticleCardOut = ReturnType<typeof cardFrom>;

/** Dựng `ArticleCard[]` (giữ nguyên thứ tự đầu vào), tải chủ đề + tác giả theo lô. */
export async function toArticleCards(articles: IArticle[]): Promise<ArticleCardOut[]> {
  const [cats, users] = await Promise.all([
    categoriesById(articles.map((a) => a.categoryId)),
    usersById(articles.map((a) => a.authorId)),
  ]);
  return articles.map((a) => cardFrom(a, cats, users));
}

/** Lộ trình (đã xuất bản) chứa bài — dùng cho `ArticleDetail.learningPath`. */
async function learningPathRefFor(a: IArticle) {
  const path = await LearningPath.findOne({ articleIds: a._id, status: 'published' }).lean<ILearningPath>();
  if (!path) return undefined;
  const arts = await Article.find({ _id: { $in: path.articleIds }, ...publicArticleFilter() })
    .select({ slug: 1, title: 1, readingMinutes: 1 })
    .lean<Pick<IArticle, '_id' | 'slug' | 'title' | 'readingMinutes'>[]>();
  const byId = new Map(arts.map((x) => [idOf(x._id), x]));
  return {
    id: idOf(path._id),
    slug: path.slug,
    title: path.title,
    level: path.level,
    articles: path.articleIds
      .map((id) => byId.get(idOf(id)))
      .filter((x): x is NonNullable<typeof x> => !!x)
      .map((x) => ({ id: idOf(x._id), slug: x.slug, title: x.title, readingMinutes: x.readingMinutes ?? 1 })),
  };
}

/** `seo` theo hợp đồng §8.1: bỏ trường rỗng, `noindex` chỉ trả khi true; không còn gì ⇒ vắng. */
export function toSeo(seo: IArticleSeo | undefined) {
  if (!seo) return undefined;
  const out = compact({
    title: seo.title?.trim() || undefined,
    description: seo.description?.trim() || undefined,
    ogImageUrl: seo.ogImageUrl?.trim() || undefined,
    noindex: seo.noindex === true ? true : undefined,
  });
  return Object.keys(out).length ? out : undefined;
}

export async function toArticleDetail(a: IArticle, myReactions: string[]) {
  const [card] = await toArticleCards([a]);
  const learningPath = await learningPathRefFor(a);
  return compact({ ...card!, content: a.content ?? '', seo: toSeo(a.seo), learningPath, myReactions });
}

/** `AdminArticle`: có cả bài nháp; `withContent` = false cho danh sách. */
export async function toAdminArticles(articles: IArticle[], withContent: boolean) {
  const cards = await toArticleCards(articles);
  return cards.map((card, i) => {
    const a = articles[i]!;
    return compact({
      ...card,
      content: withContent ? (a.content ?? '') : undefined,
      status: a.status,
      seo: toSeo(a.seo),
      learningPathId: a.learningPathId ? idOf(a.learningPathId) : undefined,
      editedInCms: a.editedInCms === true ? true : undefined,
    });
  });
}

/** Điều kiện bài công khai: đã xuất bản và `publishedAt <= now`. */
export function publicArticleFilter() {
  return { status: 'published' as const, publishedAt: { $lte: new Date() } };
}

// ---------- Lộ trình học ----------

export async function toLearningPathCards(paths: ILearningPath[]) {
  const allIds = paths.flatMap((p) => p.articleIds);
  const [cats, arts] = await Promise.all([
    categoriesById(paths.map((p) => p.categoryId)),
    Article.find({ _id: { $in: allIds }, ...publicArticleFilter() })
      .select({ readingMinutes: 1 })
      .lean<Pick<IArticle, '_id' | 'readingMinutes'>[]>(),
  ]);
  const minutes = new Map(arts.map((a) => [idOf(a._id), a.readingMinutes ?? 1]));
  return paths.map((p) => {
    const published = p.articleIds.filter((id) => minutes.has(idOf(id)));
    const cat = p.categoryId ? cats.get(idOf(p.categoryId)) : undefined;
    return compact({
      id: idOf(p._id),
      slug: p.slug,
      title: p.title,
      description: p.description ?? '',
      level: p.level,
      category: cat
        ? { slug: cat.slug, name: { vi: cat.name?.vi ?? '', en: cat.name?.en ?? '' }, color: cat.color, icon: cat.icon }
        : undefined,
      articleCount: published.length,
      totalMinutes: published.reduce((s, id) => s + (minutes.get(idOf(id)) ?? 0), 0),
      updatedAt: iso(p.updatedAt)!,
    });
  });
}

export async function toAdminLearningPaths(paths: ILearningPath[]) {
  const cards = await toLearningPathCards(paths);
  return cards.map((card, i) => {
    const p = paths[i]!;
    return compact({
      ...card,
      status: p.status,
      categoryId: p.categoryId ? idOf(p.categoryId) : undefined,
      articleIds: p.articleIds.map(idOf),
    });
  });
}

// ---------- Bình luận ----------

export interface CommentOut {
  id: string;
  articleId: string;
  parentId?: string;
  replyTo?: AuthorRef;
  author: AuthorRef;
  body: string;
  status: 'visible' | 'deleted' | 'hidden';
  likeCount: number;
  likedByMe: boolean;
  editedAt?: string;
  createdAt: string;
}

/**
 * Dựng `Comment[]`. `mode = 'public'`: bình luận ẩn/xoá trả `status: 'deleted'` + `body: ''`.
 * `mode = 'admin'`: giữ trạng thái thật (bình luận ẩn vẫn thấy nội dung).
 */
export async function toComments(
  comments: IComment[],
  viewer: IUser | undefined,
  mode: 'public' | 'admin' = 'public',
): Promise<CommentOut[]> {
  const users = await usersById(comments.flatMap((c) => [c.authorId, c.replyToUserId]));
  let liked = new Set<string>();
  if (viewer && comments.length) {
    const likes = await CommentLike.find({ userId: viewer._id, commentId: { $in: comments.map((c) => c._id) } })
      .select({ commentId: 1 })
      .lean();
    liked = new Set(likes.map((l) => idOf(l.commentId)));
  }
  return comments.map((c) => {
    const hiddenForPublic = mode === 'public' && c.status !== 'visible';
    return compact({
      id: idOf(c._id),
      articleId: idOf(c.articleId),
      parentId: c.parentId ? idOf(c.parentId) : undefined,
      replyTo: c.replyToUserId ? toAuthorRef(users.get(idOf(c.replyToUserId)), c.replyToUserId) : undefined,
      author: toAuthorRef(users.get(idOf(c.authorId)), c.authorId),
      body: hiddenForPublic || c.status === 'deleted' ? '' : c.body,
      status: hiddenForPublic ? 'deleted' : c.status,
      likeCount: Math.max(0, c.likeCount ?? 0),
      likedByMe: liked.has(idOf(c._id)),
      editedAt: iso(c.editedAt),
      createdAt: iso(c.createdAt)!,
    }) as CommentOut;
  });
}

// ---------- Thư viện media ----------

export function mediaUrl(storagePath: string): string {
  return `/uploads/${storagePath}`;
}

export async function toMediaItems(items: IMedia[]) {
  const users = await usersById(items.map((m) => m.uploadedBy));
  return items.map((m) =>
    compact({
      id: idOf(m._id),
      url: mediaUrl(m.storagePath),
      fileName: m.fileName,
      mimeType: m.mimeType,
      size: m.size,
      width: m.width,
      height: m.height,
      alt: m.alt || undefined,
      uploadedBy: toAuthorRef(users.get(idOf(m.uploadedBy)), m.uploadedBy),
      createdAt: iso(m.createdAt)!,
    }),
  );
}
