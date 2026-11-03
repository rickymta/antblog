import { Schema, model, type Types } from 'mongoose';

export const COMMENT_STATUSES = ['visible', 'hidden', 'deleted'] as const;
export type CommentStatus = (typeof COMMENT_STATUSES)[number];

export interface IComment {
  _id: Types.ObjectId;
  articleId: Types.ObjectId;
  /** Id bình luận GỐC (chỉ 2 cấp) — vắng ⇒ đây là bình luận gốc. */
  parentId?: Types.ObjectId;
  replyToUserId?: Types.ObjectId;
  authorId: Types.ObjectId;
  body: string;
  status: CommentStatus;
  likeCount: number;
  editedAt?: Date;
  isDemo?: boolean;
  createdAt: Date;
}

const commentSchema = new Schema<IComment>({
  articleId: { type: Schema.Types.ObjectId, ref: 'Article', required: true },
  parentId: { type: Schema.Types.ObjectId, ref: 'Comment' },
  replyToUserId: { type: Schema.Types.ObjectId, ref: 'User' },
  authorId: { type: Schema.Types.ObjectId, ref: 'User', required: true, index: true },
  body: { type: String, default: '' },
  status: { type: String, enum: COMMENT_STATUSES, default: 'visible' },
  likeCount: { type: Number, default: 0 },
  editedAt: Date,
  isDemo: Boolean,
  createdAt: { type: Date, default: () => new Date() },
});
commentSchema.index({ articleId: 1, parentId: 1, createdAt: -1 });
commentSchema.index({ parentId: 1, createdAt: 1 });
commentSchema.index({ status: 1, createdAt: -1 });

export const Comment = model<IComment>('Comment', commentSchema);

export interface ICommentLike {
  _id: Types.ObjectId;
  commentId: Types.ObjectId;
  userId: Types.ObjectId;
  isDemo?: boolean;
  createdAt: Date;
}

const commentLikeSchema = new Schema<ICommentLike>({
  commentId: { type: Schema.Types.ObjectId, required: true },
  userId: { type: Schema.Types.ObjectId, required: true },
  isDemo: Boolean,
  createdAt: { type: Date, default: () => new Date() },
});
commentLikeSchema.index({ commentId: 1, userId: 1 }, { unique: true });
commentLikeSchema.index({ userId: 1 });

export const CommentLike = model<ICommentLike>('CommentLike', commentLikeSchema);
