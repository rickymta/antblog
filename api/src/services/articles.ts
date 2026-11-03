import { Types } from 'mongoose';
import { conflict } from '../lib/errors.js';
import { slugify } from '../lib/text.js';
import { Article, LearningPath } from '../models/index.js';

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
