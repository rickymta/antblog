import { Schema, model, type Types } from 'mongoose';
import { REACTION_TYPES, type ReactionType } from './article.js';

export interface IReaction {
  _id: Types.ObjectId;
  articleId: Types.ObjectId;
  userId: Types.ObjectId;
  type: ReactionType;
  isDemo?: boolean;
  createdAt: Date;
}

const reactionSchema = new Schema<IReaction>({
  articleId: { type: Schema.Types.ObjectId, required: true },
  userId: { type: Schema.Types.ObjectId, required: true },
  type: { type: String, enum: REACTION_TYPES, required: true },
  isDemo: Boolean,
  createdAt: { type: Date, default: () => new Date() },
});
reactionSchema.index({ articleId: 1, userId: 1, type: 1 }, { unique: true });
reactionSchema.index({ createdAt: 1 });
reactionSchema.index({ userId: 1 });

export const Reaction = model<IReaction>('Reaction', reactionSchema);

export interface ISubscriber {
  _id: Types.ObjectId;
  email: string;
  createdAt: Date;
}

const subscriberSchema = new Schema<ISubscriber>({
  email: { type: String, required: true, unique: true, lowercase: true, trim: true },
  createdAt: { type: Date, default: () => new Date() },
});

export const Subscriber = model<ISubscriber>('Subscriber', subscriberSchema);

/** Chống đếm trùng lượt xem: một dòng mỗi cặp (bài, khách), hết hạn sau 30 phút. */
export const VIEW_DEDUPE_SECONDS = 30 * 60;

export interface IArticleView {
  articleId: Types.ObjectId;
  visitorId: string;
  createdAt: Date;
}
const articleViewSchema = new Schema<IArticleView>({
  articleId: { type: Schema.Types.ObjectId, required: true },
  visitorId: { type: String, required: true },
  createdAt: { type: Date, default: () => new Date() },
});
articleViewSchema.index({ articleId: 1, visitorId: 1 }, { unique: true });
articleViewSchema.index({ createdAt: 1 }, { expireAfterSeconds: VIEW_DEDUPE_SECONDS });
export const ArticleView = model<IArticleView>('ArticleView', articleViewSchema);

/** Khách đã ghi nhận trong một ngày (giờ VN) — để đếm khách duy nhất. Tự hết hạn sau 3 ngày. */
export interface IDailyVisitor {
  date: string;
  visitorId: string;
  createdAt: Date;
}
const dailyVisitorSchema = new Schema<IDailyVisitor>({
  date: { type: String, required: true },
  visitorId: { type: String, required: true },
  createdAt: { type: Date, default: () => new Date() },
});
dailyVisitorSchema.index({ date: 1, visitorId: 1 }, { unique: true });
dailyVisitorSchema.index({ createdAt: 1 }, { expireAfterSeconds: 3 * 24 * 3600 });
export const DailyVisitor = model<IDailyVisitor>('DailyVisitor', dailyVisitorSchema);

/** Truy cập toàn site theo ngày VN. `isDemo` tách số liệu demo để gỡ được. */
export interface IDailySiteStat {
  date: string;
  isDemo: boolean;
  visits: number;
  uniqueVisitors: number;
}
const dailySiteStatSchema = new Schema<IDailySiteStat>({
  date: { type: String, required: true },
  isDemo: { type: Boolean, default: false },
  visits: { type: Number, default: 0 },
  uniqueVisitors: { type: Number, default: 0 },
});
dailySiteStatSchema.index({ date: 1, isDemo: 1 }, { unique: true });
export const DailySiteStat = model<IDailySiteStat>('DailySiteStat', dailySiteStatSchema);

/** Lượt xem từng bài theo ngày VN. */
export interface IDailyArticleStat {
  date: string;
  articleId: Types.ObjectId;
  isDemo: boolean;
  views: number;
}
const dailyArticleStatSchema = new Schema<IDailyArticleStat>({
  date: { type: String, required: true },
  articleId: { type: Schema.Types.ObjectId, required: true },
  isDemo: { type: Boolean, default: false },
  views: { type: Number, default: 0 },
});
dailyArticleStatSchema.index({ date: 1, articleId: 1, isDemo: 1 }, { unique: true });
dailyArticleStatSchema.index({ articleId: 1, date: 1 });
export const DailyArticleStat = model<IDailyArticleStat>('DailyArticleStat', dailyArticleStatSchema);
