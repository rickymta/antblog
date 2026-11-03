import type { Types } from 'mongoose';
import { Article, Category, Comment, CommentLike, Reaction, REACTION_TYPES } from '../models/index.js';

/** `category.articleCount` = số bài `published`. Không truyền id ⇒ tính lại mọi chủ đề. */
export async function recountCategories(ids?: (Types.ObjectId | string | undefined)[]): Promise<void> {
  const filter = ids ? { _id: { $in: ids.filter(Boolean) } } : {};
  const cats = await Category.find(filter).select({ _id: 1 }).lean();
  await Promise.all(
    cats.map(async (c) => {
      const n = await Article.countDocuments({ categoryId: c._id, status: 'published' });
      await Category.updateOne({ _id: c._id }, { $set: { articleCount: n } });
    }),
  );
}

/** Tính lại `stats.comments` (bình luận `visible`) + `stats.reactions` từ dữ liệu gốc. Không truyền id ⇒ mọi bài. */
export async function recountArticleActivity(ids?: Types.ObjectId[]): Promise<void> {
  const filter = ids ? { _id: { $in: ids } } : {};
  const articles = await Article.find(filter).select({ _id: 1 }).lean();
  await Promise.all(
    articles.map(async (a) => {
      const [comments, reactions] = await Promise.all([
        Comment.countDocuments({ articleId: a._id, status: 'visible' }),
        Reaction.aggregate<{ _id: string; n: number }>([
          { $match: { articleId: a._id } },
          { $group: { _id: '$type', n: { $sum: 1 } } },
        ]),
      ]);
      const set: Record<string, number> = { 'stats.comments': comments };
      for (const t of REACTION_TYPES) set[`stats.reactions.${t}`] = reactions.find((r) => r._id === t)?.n ?? 0;
      await Article.updateOne({ _id: a._id }, { $set: set });
    }),
  );
}

/** Tính lại `comment.likeCount` từ bảng like. */
export async function recountCommentLikes(commentIds?: Types.ObjectId[]): Promise<void> {
  const filter = commentIds ? { _id: { $in: commentIds } } : {};
  const comments = await Comment.find(filter).select({ _id: 1 }).lean();
  await Promise.all(
    comments.map(async (c) => {
      const n = await CommentLike.countDocuments({ commentId: c._id });
      await Comment.updateOne({ _id: c._id }, { $set: { likeCount: n } });
    }),
  );
}
