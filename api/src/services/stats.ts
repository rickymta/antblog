import type { Types } from 'mongoose';
import { addDays, dateRange, vnDate, vnDayStart } from '../lib/dates.js';
import { isDuplicateKeyError } from '../lib/errors.js';
import {
  Article,
  ArticleView,
  Comment,
  DailyArticleStat,
  DailySiteStat,
  DailyVisitor,
  Reaction,
  Subscriber,
  User,
  VIEW_DEDUPE_SECONDS,
  type IArticle,
} from '../models/index.js';

/** POST /track/visit: tăng lượt truy cập hôm nay; khách chưa ghi nhận trong ngày ⇒ tăng khách duy nhất. */
export async function recordVisit(visitorId: string, now = new Date()): Promise<void> {
  const date = vnDate(now);
  let isNewVisitor = false;
  try {
    await DailyVisitor.create({ date, visitorId, createdAt: now });
    isNewVisitor = true;
  } catch (err) {
    if (!isDuplicateKeyError(err)) throw err;
  }
  await DailySiteStat.updateOne(
    { date, isDemo: false },
    { $inc: { visits: 1, uniqueVisitors: isNewVisitor ? 1 : 0 } },
    { upsert: true },
  );
}

/**
 * Đếm lượt xem nếu cặp (bài, khách) chưa được đếm trong 30 phút gần nhất.
 * Chống trùng nhờ index unique (articleId, visitorId); dòng cũ hơn 30 phút được "làm mới" nguyên tử
 * (không phụ thuộc chu kỳ dọn TTL của MongoDB).
 */
export async function recordView(article: IArticle, visitorId: string, now = new Date()) {
  let counted = false;
  try {
    await ArticleView.create({ articleId: article._id, visitorId, createdAt: now });
    counted = true;
  } catch (err) {
    if (!isDuplicateKeyError(err)) throw err;
    const cutoff = new Date(now.getTime() - VIEW_DEDUPE_SECONDS * 1000);
    const res = await ArticleView.updateOne(
      { articleId: article._id, visitorId, createdAt: { $lte: cutoff } },
      { $set: { createdAt: now } },
    );
    counted = res.modifiedCount === 1;
  }
  if (!counted) {
    const fresh = await Article.findById(article._id).select({ 'stats.views': 1 }).lean<Pick<IArticle, 'stats'>>();
    return { counted: false, views: fresh?.stats?.views ?? article.stats?.views ?? 0 };
  }
  const [updated] = await Promise.all([
    Article.findByIdAndUpdate(article._id, { $inc: { 'stats.views': 1 } }, { new: true, projection: { 'stats.views': 1 } }).lean<
      Pick<IArticle, 'stats'>
    >(),
    DailyArticleStat.updateOne(
      { date: vnDate(now), articleId: article._id, isDemo: false },
      { $inc: { views: 1 } },
      { upsert: true },
    ),
  ]);
  return { counted: true, views: updated?.stats?.views ?? 0 };
}

/** Lượt xem theo bài trong `days` ngày gần nhất (tính cả hôm nay), giảm dần. */
export async function recentViewsByArticle(days: number): Promise<{ articleId: Types.ObjectId; views: number }[]> {
  const to = vnDate();
  const from = addDays(to, -(days - 1));
  return DailyArticleStat.aggregate<{ articleId: Types.ObjectId; views: number }>([
    { $match: { date: { $gte: from, $lte: to } } },
    { $group: { _id: '$articleId', views: { $sum: '$views' } } },
    { $match: { views: { $gt: 0 } } },
    { $sort: { views: -1, _id: 1 } },
    { $project: { _id: 0, articleId: '$_id', views: 1 } },
  ]);
}

type DayCount = { _id: string; n: number };

function vnDayGroup(field: string) {
  return { $dateToString: { format: '%Y-%m-%d', date: field, timezone: '+07:00' } };
}

export async function statsOverview(days: number) {
  const to = vnDate();
  const from = addDays(to, -(days - 1));
  const prevTo = addDays(from, -1);
  const prevFrom = addDays(prevTo, -(days - 1));
  const rangeStart = vnDayStart(prevFrom);
  const rangeEnd = vnDayStart(addDays(to, 1));
  const periodStart = vnDayStart(from);

  const [site, views, comments, reactions, articles, published, drafts, users, subscribers] = await Promise.all([
    DailySiteStat.aggregate<{ _id: string; visits: number; uniqueVisitors: number }>([
      { $match: { date: { $gte: prevFrom, $lte: to } } },
      { $group: { _id: '$date', visits: { $sum: '$visits' }, uniqueVisitors: { $sum: '$uniqueVisitors' } } },
    ]),
    DailyArticleStat.aggregate<DayCount>([
      { $match: { date: { $gte: prevFrom, $lte: to } } },
      { $group: { _id: '$date', n: { $sum: '$views' } } },
    ]),
    Comment.aggregate<DayCount>([
      { $match: { status: 'visible', createdAt: { $gte: rangeStart, $lt: rangeEnd } } },
      { $group: { _id: vnDayGroup('$createdAt'), n: { $sum: 1 } } },
    ]),
    Reaction.aggregate<DayCount>([
      { $match: { createdAt: { $gte: rangeStart, $lt: rangeEnd } } },
      { $group: { _id: vnDayGroup('$createdAt'), n: { $sum: 1 } } },
    ]),
    Article.countDocuments({ status: { $ne: 'archived' } }),
    Article.countDocuments({ status: 'published' }),
    Article.countDocuments({ status: 'draft' }),
    User.countDocuments({}),
    Subscriber.countDocuments({}),
  ]);

  const siteBy = new Map(site.map((s) => [s._id, s]));
  const toMap = (rows: DayCount[]) => new Map(rows.map((r) => [r._id, r.n]));
  const viewsBy = toMap(views);
  const commentsBy = toMap(comments);
  const reactionsBy = toMap(reactions);
  const point = (date: string) => ({
    date,
    views: viewsBy.get(date) ?? 0,
    visits: siteBy.get(date)?.visits ?? 0,
    uniqueVisitors: siteBy.get(date)?.uniqueVisitors ?? 0,
    comments: commentsBy.get(date) ?? 0,
    reactions: reactionsBy.get(date) ?? 0,
  });
  const series = dateRange(from, to).map(point);
  const prevSeries = dateRange(prevFrom, prevTo).map(point);
  const sum = (rows: ReturnType<typeof point>[], k: 'views' | 'visits' | 'uniqueVisitors' | 'comments') =>
    rows.reduce((s, r) => s + r[k], 0);

  return {
    range: { from, to, days },
    totals: {
      articles,
      published,
      drafts,
      users,
      comments: sum(series, 'comments'),
      subscribers,
      views: sum(series, 'views'),
      visits: sum(series, 'visits'),
      uniqueVisitors: sum(series, 'uniqueVisitors'),
    },
    previous: {
      views: sum(prevSeries, 'views'),
      visits: sum(prevSeries, 'visits'),
      uniqueVisitors: sum(prevSeries, 'uniqueVisitors'),
      comments: sum(prevSeries, 'comments'),
    },
    series,
    topArticles: await topArticles(from, to, periodStart, rangeEnd),
  };
}

async function topArticles(from: string, to: string, start: Date, end: Date) {
  const top = await DailyArticleStat.aggregate<{ _id: Types.ObjectId; views: number }>([
    { $match: { date: { $gte: from, $lte: to } } },
    { $group: { _id: '$articleId', views: { $sum: '$views' } } },
    { $match: { views: { $gt: 0 } } },
    { $lookup: { from: Article.collection.name, localField: '_id', foreignField: '_id', as: 'a', pipeline: [{ $project: { _id: 1 } }] } },
    { $match: { 'a.0': { $exists: true } } },
    { $sort: { views: -1, _id: 1 } },
    { $limit: 10 },
    { $project: { a: 0 } },
  ]);
  if (!top.length) return [];
  const ids = top.map((t) => t._id);
  const [arts, comments, reactions] = await Promise.all([
    Article.find({ _id: { $in: ids } }).select({ slug: 1, title: 1 }).lean<Pick<IArticle, '_id' | 'slug' | 'title'>[]>(),
    Comment.aggregate<{ _id: Types.ObjectId; n: number }>([
      { $match: { articleId: { $in: ids }, status: 'visible', createdAt: { $gte: start, $lt: end } } },
      { $group: { _id: '$articleId', n: { $sum: 1 } } },
    ]),
    Reaction.aggregate<{ _id: Types.ObjectId; n: number }>([
      { $match: { articleId: { $in: ids }, createdAt: { $gte: start, $lt: end } } },
      { $group: { _id: '$articleId', n: { $sum: 1 } } },
    ]),
  ]);
  const artBy = new Map(arts.map((a) => [a._id.toString(), a]));
  const cBy = new Map(comments.map((c) => [c._id.toString(), c.n]));
  const rBy = new Map(reactions.map((r) => [r._id.toString(), r.n]));
  return top
    .filter((t) => artBy.has(t._id.toString()))
    .map((t) => {
      const a = artBy.get(t._id.toString())!;
      const id = t._id.toString();
      return { id, slug: a.slug, title: a.title, views: t.views, comments: cBy.get(id) ?? 0, reactions: rBy.get(id) ?? 0 };
    });
}
