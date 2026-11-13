import { Types } from 'mongoose';
import { conflict } from '../lib/errors.js';
import { slugify } from '../lib/text.js';
import { Article, Category, LearningPath, LEVELS } from '../models/index.js';

/**
 * Slug cho bài viết. `explicit` (người dùng nhập) trùng ⇒ 409 SLUG_TAKEN.
 * Slug tự sinh trùng ⇒ thêm hậu tố -2, -3, ...
 */
export async function resolveArticleSlug(opts: { explicit?: string; title: string; excludeId?: Types.ObjectId }): Promise<string> {
  const exclude = opts.excludeId ? { _id: { $ne: opts.excludeId } } : {};
  if (opts.explicit) {
    if (await Article.exists({ slug: opts.explicit, ...exclude })) {
      throw conflict('SLUG_TAKEN', 'Slug này đã được dùng cho bài viết khác.');
    }
    return opts.explicit;
  }
  const base = slugify(opts.title) || 'bai-viet';
  for (let i = 1; i < 1000; i++) {
    const candidate = i === 1 ? base : `${base}-${i}`;
    if (!(await Article.exists({ slug: candidate, ...exclude }))) return candidate;
  }
  return `${base}-${new Types.ObjectId().toString().slice(-6)}`;
}

/**
 * Đặt danh sách bài của một lộ trình, giữ luật "một bài thuộc tối đa một lộ trình":
 * gỡ các bài này khỏi lộ trình khác, cập nhật `article.learningPathId` cho bài thêm/bớt.
 */
export async function syncLearningPathArticles(
  pathId: Types.ObjectId,
  articleIds: Types.ObjectId[],
  previousIds: Types.ObjectId[],
): Promise<void> {
  if (articleIds.length) {
    await LearningPath.updateMany({ _id: { $ne: pathId }, articleIds: { $in: articleIds } }, { $pull: { articleIds: { $in: articleIds } } });
    await Article.updateMany({ _id: { $in: articleIds } }, { $set: { learningPathId: pathId } });
  }
  const keep = new Set(articleIds.map(String));
  const removed = previousIds.filter((id) => !keep.has(String(id)));
  if (removed.length) {
    await Article.updateMany({ _id: { $in: removed }, learningPathId: pathId }, { $unset: { learningPathId: 1 } });
  }
}

/**
 * Thứ tự các phần (lộ trình) trong cùng một chủ đề: `priority` tăng dần (vắng ⇒ sau mọi phần có số) → cấp độ → slug
 * (để "-phan-02" đứng sau phần gốc). Phải khớp `orderTopicParts` ở apps/web.
 */
export function comparePathOrder(
  a: { priority?: number | null; level: (typeof LEVELS)[number]; slug: string },
  b: { priority?: number | null; level: (typeof LEVELS)[number]; slug: string },
): number {
  const pa = a.priority ?? Number.POSITIVE_INFINITY;
  const pb = b.priority ?? Number.POSITIVE_INFINITY;
  return (pa === pb ? 0 : pa < pb ? -1 : 1)
    || LEVELS.indexOf(a.level) - LEVELS.indexOf(b.level)
    || a.slug.localeCompare(b.slug, 'en', { numeric: true });
}

/**
 * Thứ hạng đọc của từng bài theo lộ trình (`sort=path`): chủ đề theo thứ tự cây (order, slug) → phần theo `comparePathOrder`
 * → vị trí bài trong phần. Bài không thuộc lộ trình đã xuất bản nào thì không có hạng.
 */
export async function learningOrderRank(): Promise<Map<string, number>> {
  const [paths, cats] = await Promise.all([
    LearningPath.find({ status: 'published' }).select({ slug: 1, level: 1, priority: 1, categoryId: 1, articleIds: 1 }).lean(),
    Category.find({}).select({ parentId: 1, order: 1, slug: 1 }).lean(),
  ]);
  const children = new Map<string, typeof cats>();
  for (const c of cats) {
    const key = c.parentId ? String(c.parentId) : '';
    children.set(key, [...(children.get(key) ?? []), c]);
  }
  const catRank = new Map<string, number>();
  const walk = (parent: string) => {
    for (const c of (children.get(parent) ?? []).sort((a, b) => (a.order ?? 0) - (b.order ?? 0) || a.slug.localeCompare(b.slug))) {
      if (catRank.has(String(c._id))) continue;
      catRank.set(String(c._id), catRank.size);
      walk(String(c._id));
    }
  };
  walk('');
  const rankOf = (p: (typeof paths)[number]) => (p.categoryId ? catRank.get(String(p.categoryId)) : undefined) ?? Number.MAX_SAFE_INTEGER;
  paths.sort((a, b) => rankOf(a) - rankOf(b) || comparePathOrder(a, b));
  const rank = new Map<string, number>();
  for (const p of paths) for (const id of p.articleIds) if (!rank.has(String(id))) rank.set(String(id), rank.size);
  return rank;
}
