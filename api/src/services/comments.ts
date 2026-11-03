import type { Types } from 'mongoose';
import { Article, Comment, type IComment } from '../models/index.js';

/**
 * Xoá mềm một bình luận (chủ bình luận hoặc admin). Nguyên tử: chỉ bình luận chưa xoá mới chuyển trạng thái;
 * nếu trước đó đang `visible` ⇒ giảm `stats.comments` của bài.
 */
export async function softDeleteComment(id: Types.ObjectId): Promise<void> {
  const prev = await Comment.findOneAndUpdate(
    { _id: id, status: { $ne: 'deleted' } },
    { $set: { status: 'deleted', body: '' } },
    { new: false },
  ).lean<IComment>();
  if (prev?.status === 'visible') {
    await Article.updateOne({ _id: prev.articleId }, { $inc: { 'stats.comments': -1 } });
  }
}

/** Ẩn/hiện bình luận (admin). Trả bình luận sau cập nhật; `null` nếu không có thay đổi hợp lệ. */
export async function setCommentVisibility(id: Types.ObjectId, status: 'visible' | 'hidden'): Promise<IComment | null> {
  const from = status === 'visible' ? 'hidden' : 'visible';
  const updated = await Comment.findOneAndUpdate({ _id: id, status: from }, { $set: { status } }, { new: true }).lean<IComment>();
  if (updated) {
    await Article.updateOne({ _id: updated.articleId }, { $inc: { 'stats.comments': status === 'visible' ? 1 : -1 } });
  }
  return updated;
}
