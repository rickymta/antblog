import { categorySubtreeIds } from '../services/categories.js';
import { Router, type Request } from 'express';
import type { Types } from 'mongoose';
import { z } from 'zod';
import { isDuplicateKeyError, notFound, validation } from '../lib/errors.js';
import { isBot } from '../lib/text.js';
import { optionalQueryString, pagingSchema, parse, queryInt } from '../lib/validate.js';
import { currentUser, requireAuth } from '../middleware/auth.js';
import {
  Article,
  Category,
  DailyArticleStat,
  LEVELS,
  Reaction,
  REACTION_TYPES,
  type IArticle,
  type ICategory,
  type ReactionType,
} from '../models/index.js';
import { recentViewsByArticle, recordView } from '../services/stats.js';
import { publicArticleFilter, reactionCounts, toArticleCards, toArticleDetail } from '../services/serializers.js';
import { addDays, vnDate } from '../lib/dates.js';

const emptyToUndefined = (v: unknown) => (v === '' ? undefined : v);

const listSchema = z.object({
  category: optionalQueryString(100),
  level: z.preprocess(emptyToUndefined, z.enum(LEVELS).optional()),
  tag: optionalQueryString(60),
  q: optionalQueryString(200),
  sort: z.preprocess(emptyToUndefined, z.enum(['latest', 'popular', 'trending']).default('latest')),
  ...pagingSchema(12),
});

export async function findPublicArticle(slug: string): Promise<IArticle> {
  const a = await Article.findOne({ slug, ...publicArticleFilter() }).lean<IArticle>();
  if (!a) throw notFound('Không tìm thấy bài viết.');
  return a;
}

async function myReactionsOf(articleId: Types.ObjectId, req: Request): Promise<ReactionType[]> {
  if (!req.user) return [];
  const types = new Set(
    (await Reaction.find({ articleId, userId: req.user._id }).select({ type: 1 }).lean()).map((r) => r.type),
  );
  return REACTION_TYPES.filter((t) => types.has(t));
}

export function articlesRouter(): Router {
  const r = Router();

  r.get('/articles', async (req, res) => {
    const q = parse(listSchema, req.query);
    const empty = { items: [], total: 0, page: q.page, pageSize: q.pageSize };
    const filter: Record<string, unknown> = publicArticleFilter();
    if (q.category) {
      const cat = await Category.findOne({ slug: q.category }).select({ _id: 1 }).lean<ICategory>();
      if (!cat) return void res.json(empty);
      filter.categoryId = { $in: await categorySubtreeIds(cat._id) };
    }
    if (q.level) filter.level = q.level;
    if (q.tag) filter.tags = q.tag.toLowerCase();
    if (q.q) filter.$text = { $search: q.q };
    const skip = (q.page - 1) * q.pageSize;

    let articles: IArticle[];
    let total: number;
    if (q.sort === 'trending') {
      const from = addDays(vnDate(), -6);
      const [result] = await Article.aggregate<{ items: IArticle[]; total: { n: number }[] }>([
        { $match: filter },
        {
          $lookup: {
            from: DailyArticleStat.collection.name,
            localField: '_id',
            foreignField: 'articleId',
            as: 'recent',
            pipeline: [{ $match: { date: { $gte: from } } }, { $group: { _id: null, v: { $sum: '$views' } } }],
          },
        },
        { $addFields: { recentViews: { $ifNull: [{ $first: '$recent.v' }, 0] } } },
        { $sort: { recentViews: -1, 'stats.views': -1, publishedAt: -1, _id: -1 } },
        {
          $facet: {
            items: [{ $skip: skip }, { $limit: q.pageSize }, { $project: { recent: 0, recentViews: 0 } }],
            total: [{ $count: 'n' }],
          },
        },
      ]);
      articles = result?.items ?? [];
      total = result?.total[0]?.n ?? 0;
    } else {
      const sort: Record<string, 1 | -1> =
        q.sort === 'popular' ? { 'stats.views': -1, publishedAt: -1, _id: -1 } : { publishedAt: -1, _id: -1 };
      [articles, total] = await Promise.all([
        Article.find(filter).sort(sort).skip(skip).limit(q.pageSize).lean<IArticle[]>(),
        Article.countDocuments(filter),
      ]);
    }
    res.json({ items: await toArticleCards(articles), total, page: q.page, pageSize: q.pageSize });
  });

  r.get('/articles/trending', async (req, res) => {
    const { limit } = parse(z.object({ limit: queryInt(1, 12, 4) }), req.query);
    const recent = await recentViewsByArticle(7);
    const ids = recent.map((x) => x.articleId);
    const found = await Article.find({ _id: { $in: ids }, ...publicArticleFilter() }).lean<IArticle[]>();
    const byId = new Map(found.map((a) => [a._id.toString(), a]));
    const picked = ids
      .map((id) => byId.get(id.toString()))
      .filter((a): a is IArticle => !!a)
      .slice(0, limit);
    if (picked.length < limit) {
      const fill = await Article.find({ ...publicArticleFilter(), _id: { $nin: picked.map((a) => a._id) } })
        .sort({ 'stats.views': -1, publishedAt: -1, _id: -1 })
        .limit(limit - picked.length)
        .lean<IArticle[]>();
      picked.push(...fill);
    }
    res.json(await toArticleCards(picked));
  });

  r.get('/articles/:slug', async (req, res) => {
    const article = await findPublicArticle(String(req.params.slug));
    res.json(await toArticleDetail(article, await myReactionsOf(article._id, req)));
  });

  r.get('/articles/:slug/related', async (req, res) => {
    const { limit } = parse(z.object({ limit: queryInt(1, 12, 3) }), req.query);
    const article = await findPublicArticle(String(req.params.slug));
    const related = await Article.aggregate<IArticle>([
      { $match: { ...publicArticleFilter(), categoryId: article.categoryId, _id: { $ne: article._id } } },
      { $addFields: { sharedTags: { $size: { $setIntersection: [{ $ifNull: ['$tags', []] }, article.tags ?? []] } } } },
      { $sort: { sharedTags: -1, publishedAt: -1, _id: -1 } },
      { $limit: limit },
      { $project: { sharedTags: 0 } },
    ]);
    res.json(await toArticleCards(related));
  });

  r.post('/articles/:slug/view', async (req, res) => {
    const article = await findPublicArticle(String(req.params.slug));
    if (isBot(req.get('user-agent')) || !req.visitorId) {
      return void res.json({ counted: false, views: article.stats?.views ?? 0 });
    }
    res.json(await recordView(article, req.visitorId));
  });

  // ---- Reaction ----
  const reactionParams = z.object({ type: z.enum(REACTION_TYPES) });

  async function reactionState(articleId: Types.ObjectId, req: Request) {
    const a = await Article.findById(articleId).select({ 'stats.reactions': 1 }).lean<Pick<IArticle, 'stats'>>();
    return { reactions: reactionCounts(a?.stats?.reactions), myReactions: await myReactionsOf(articleId, req) };
  }

  r.put('/articles/:slug/reactions/:type', requireAuth, async (req, res) => {
    const parsed = reactionParams.safeParse(req.params);
    if (!parsed.success) throw validation({ type: 'Loại reaction không hợp lệ' });
    const { type } = parsed.data;
    const article = await findPublicArticle(String(req.params.slug));
    const user = currentUser(req);
    try {
      await Reaction.create({ articleId: article._id, userId: user._id, type });
      await Article.updateOne({ _id: article._id }, { $inc: { [`stats.reactions.${type}`]: 1 } });
    } catch (err) {
      if (!isDuplicateKeyError(err)) throw err; // đã có ⇒ idempotent
    }
    res.json(await reactionState(article._id, req));
  });

  r.delete('/articles/:slug/reactions/:type', requireAuth, async (req, res) => {
    const parsed = reactionParams.safeParse(req.params);
    if (!parsed.success) throw validation({ type: 'Loại reaction không hợp lệ' });
    const { type } = parsed.data;
    const article = await findPublicArticle(String(req.params.slug));
    const user = currentUser(req);
    const removed = await Reaction.deleteOne({ articleId: article._id, userId: user._id, type });
    if (removed.deletedCount === 1) {
      await Article.updateOne({ _id: article._id }, { $inc: { [`stats.reactions.${type}`]: -1 } });
    }
    res.json(await reactionState(article._id, req));
  });

  return r;
}
