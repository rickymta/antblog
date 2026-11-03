import { Router } from 'express';
import { z } from 'zod';
import { notFound } from '../lib/errors.js';
import { escapeRegex, isBot } from '../lib/text.js';
import { optionalQueryString, parse, queryInt } from '../lib/validate.js';
import {
  Article,
  Category,
  LearningPath,
  LEVELS,
  Subscriber,
  User,
  type IArticle,
  type ICategory,
  type ILearningPath,
} from '../models/index.js';
import { recordVisit } from '../services/stats.js';
import { publicArticleFilter, toArticleCards, toCategory, toLearningPathCards } from '../services/serializers.js';
import { isDuplicateKeyError } from '../lib/errors.js';

const emptyToUndefined = (v: unknown) => (v === '' ? undefined : v);

export function publicRouter(): Router {
  const r = Router();

  r.get('/site/summary', async (_req, res) => {
    const [articleCount, categoryCount, learningPathCount, memberCount, views] = await Promise.all([
      Article.countDocuments(publicArticleFilter()),
      Category.countDocuments({}),
      LearningPath.countDocuments({ status: 'published' }),
      User.countDocuments({ status: 'active' }),
      Article.aggregate<{ total: number }>([
        { $match: publicArticleFilter() },
        { $group: { _id: null, total: { $sum: '$stats.views' } } },
      ]),
    ]);
    res.json({ articleCount, categoryCount, learningPathCount, memberCount, totalViews: views[0]?.total ?? 0 });
  });

  r.get('/categories', async (_req, res) => {
    const cats = await Category.find({}).sort({ order: 1, slug: 1 }).lean<ICategory[]>();
    res.json(cats.map(toCategory));
  });

  r.get('/categories/:slug', async (req, res) => {
    const cat = await Category.findOne({ slug: req.params.slug }).lean<ICategory>();
    if (!cat) throw notFound('Không tìm thấy chủ đề.');
    res.json(toCategory(cat));
  });

  r.get('/tags/popular', async (req, res) => {
    const { limit } = parse(z.object({ limit: queryInt(1, 100, 20) }), req.query);
    const rows = await Article.aggregate<{ tag: string; count: number }>([
      { $match: publicArticleFilter() },
      { $unwind: '$tags' },
      { $group: { _id: '$tags', count: { $sum: 1 } } },
      { $sort: { count: -1, _id: 1 } },
      { $limit: limit },
      { $project: { _id: 0, tag: '$_id', count: 1 } },
    ]);
    res.json(rows);
  });

  r.get('/learning-paths', async (req, res) => {
    const q = parse(
      z.object({ level: z.preprocess(emptyToUndefined, z.enum(LEVELS).optional()), q: optionalQueryString(200) }),
      req.query,
    );
    const filter: Record<string, unknown> = { status: 'published' };
    if (q.level) filter.level = q.level;
    if (q.q) {
      const rx = new RegExp(escapeRegex(q.q), 'i');
      filter.$or = [{ title: rx }, { description: rx }];
    }
    const paths = await LearningPath.find(filter).sort({ createdAt: 1, _id: 1 }).lean<ILearningPath[]>();
    res.json(await toLearningPathCards(paths));
  });

  r.get('/learning-paths/:slug', async (req, res) => {
    const path = await LearningPath.findOne({ slug: req.params.slug, status: 'published' }).lean<ILearningPath>();
    if (!path) throw notFound('Không tìm thấy lộ trình học.');
    const [card] = await toLearningPathCards([path]);
    const arts = await Article.find({ _id: { $in: path.articleIds }, ...publicArticleFilter() }).lean<IArticle[]>();
    const byId = new Map(arts.map((a) => [a._id.toString(), a]));
    const ordered = path.articleIds.map((id) => byId.get(id.toString())).filter((a): a is IArticle => !!a);
    res.json({ ...card, articles: await toArticleCards(ordered) });
  });

  r.post('/track/visit', async (req, res) => {
    parse(z.object({ path: z.string().max(2048) }), req.body);
    if (!isBot(req.get('user-agent')) && req.visitorId) await recordVisit(req.visitorId);
    res.status(204).end();
  });

  r.post('/subscribers', async (req, res) => {
    const { email } = parse(z.object({ email: z.string().trim().toLowerCase().max(254).pipe(z.email()) }), req.body);
    try {
      await Subscriber.create({ email });
      res.status(201).json({ ok: true });
    } catch (err) {
      if (!isDuplicateKeyError(err)) throw err;
      res.status(200).json({ ok: true });
    }
  });

  return r;
}
