import request from 'supertest';
import { randomUUID } from 'node:crypto';
import { beforeEach, describe, expect, it } from 'vitest';
import { vnDate } from '../src/lib/dates.js';
import { ArticleView, DailyArticleStat, DailySiteStat, LearningPath } from '../src/models/index.js';
import { syncLearningPathArticles } from '../src/services/articles.js';
import {
  assertCleanJson,
  createArticle,
  createCategory,
  createUser,
  makeApp,
  resetDb,
  sessionCookie,
  useTestDb,
  visitorCookie,
} from './helpers.js';

useTestDb();
const app = makeApp();

describe('nội dung công khai', () => {
  beforeEach(resetDb);

  it('danh sách: chỉ bài đã xuất bản, lọc chủ đề/level/tag/q, sắp xếp + phân trang', async () => {
    const admin = await createUser('admin');
    const db = await createCategory({ slug: 'database' });
    const code = await createCategory({ slug: 'code' });
    const a1 = await createArticle(db, admin, { title: 'Chỉ mục trong MongoDB', tags: ['mongodb', 'index'], level: 'intermediate', stats: { views: 5, comments: 0, reactions: { love: 0, helpful: 0, insightful: 0, inspiring: 0 } } });
    const a2 = await createArticle(db, admin, { title: 'Giao dịch SQL', tags: ['sql'], stats: { views: 50, comments: 0, reactions: { love: 0, helpful: 0, insightful: 0, inspiring: 0 } } });
    await createArticle(code, admin, { title: 'Clean code', tags: ['mongodb'] });
    await createArticle(db, admin, { title: 'Bản nháp bí mật', status: 'draft', publishedAt: undefined });
    await createArticle(db, admin, { title: 'Bài lưu trữ', status: 'archived' });
    await createArticle(db, admin, { title: 'Hẹn giờ tương lai', publishedAt: new Date(Date.now() + 3600_000) });

    const all = await request(app).get('/api/v1/articles').expect(200);
    expect(all.body).toMatchObject({ total: 3, page: 1, pageSize: 12 });
    expect(all.body.items.map((x: { title: string }) => x.title)).not.toContain('Bản nháp bí mật');
    assertCleanJson(all.body);
    const card = all.body.items[0];
    expect(Object.keys(card).sort()).toEqual(
      ['author', 'category', 'id', 'language', 'level', 'publishedAt', 'readingMinutes', 'slug', 'stats', 'summary', 'tags', 'title', 'updatedAt'].sort(),
    );
    expect(card.author).not.toHaveProperty('email');
    expect(card.stats.reactions).toEqual({ love: 0, helpful: 0, insightful: 0, inspiring: 0 });
    expect(Object.keys(card.category).sort()).toEqual(['color', 'icon', 'id', 'name', 'slug']);

    const byCat = await request(app).get('/api/v1/articles?category=database').expect(200);
    expect(byCat.body.total).toBe(2);
    expect((await request(app).get('/api/v1/articles?category=khong-co').expect(200)).body.total).toBe(0);
    expect((await request(app).get('/api/v1/articles?level=intermediate').expect(200)).body.items[0].id).toBe(a1._id.toString());
    expect((await request(app).get('/api/v1/articles?tag=MongoDB').expect(200)).body.total).toBe(2);
    const q = await request(app).get('/api/v1/articles?q=giao%20d%E1%BB%8Bch').expect(200);
    expect(q.body.items.map((x: { id: string }) => x.id)).toContain(a2._id.toString());
    const qNoAccent = await request(app).get('/api/v1/articles?q=mongodb').expect(200);
    expect(qNoAccent.body.total).toBe(2);

    const popular = await request(app).get('/api/v1/articles?sort=popular').expect(200);
    expect(popular.body.items[0].id).toBe(a2._id.toString());
    const paged = await request(app).get('/api/v1/articles?pageSize=2&page=2').expect(200);
    expect(paged.body).toMatchObject({ total: 3, page: 2, pageSize: 2 });
    expect(paged.body.items).toHaveLength(1);
    const bad = await request(app).get('/api/v1/articles?pageSize=51&level=expert').expect(400);
    expect(Object.keys(bad.body.error.details.fields).sort()).toEqual(['level', 'pageSize']);
  });

  it('chi tiết: có content + myReactions, bài nháp/lưu trữ ⇒ 404; không tự tăng lượt xem', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory();
    const pub = await createArticle(cat, admin, { content: '# Tiêu đề\n\nNội dung <svg><text>bỏ</text></svg>' });
    const draft = await createArticle(cat, admin, { status: 'draft', publishedAt: undefined });
    const res = await request(app).get(`/api/v1/articles/${pub.slug}`).expect(200);
    expect(res.body.content).toContain('# Tiêu đề');
    expect(res.body.myReactions).toEqual([]);
    expect(res.body).not.toHaveProperty('learningPath');
    expect(res.body.stats.views).toBe(0);
    assertCleanJson(res.body);
    await request(app).get(`/api/v1/articles/${draft.slug}`).expect(404);
    const again = await request(app).get(`/api/v1/articles/${pub.slug}`).expect(200);
    expect(again.body.stats.views).toBe(0);
  });

  it('sort=path: theo thứ tự chủ đề → phần → vị trí bài, bài ngoài lộ trình xếp cuối; thẻ kèm pathRef', async () => {
    const admin = await createUser('admin');
    const first = await createCategory({ slug: 'alpha', order: 1 });
    const second = await createCategory({ slug: 'beta', order: 2 });
    const [b2, a2, a1, loose, b1] = await Promise.all([
      createArticle(second, admin), createArticle(first, admin), createArticle(first, admin), createArticle(first, admin), createArticle(second, admin),
    ]);
    const mk = async (slug: string, level: 'beginner' | 'advanced', categoryId: unknown, ids: (typeof a1)['_id'][]) => {
      const p = await LearningPath.create({ slug, title: slug, level, categoryId, articleIds: ids, status: 'published' });
      await syncLearningPathArticles(p._id, ids, []);
    };
    await mk('beta-co-ban', 'beginner', second._id, [b1._id, b2._id]);
    await mk('alpha-nang-cao', 'advanced', first._id, [a2._id]);
    await mk('alpha-co-ban', 'beginner', first._id, [a1._id]);

    const res = await request(app).get('/api/v1/articles?sort=path').expect(200);
    expect(res.body.items.map((x: { id: string }) => x.id)).toEqual([a1, a2, b1, b2, loose].map((x) => x._id.toString()));
    expect(res.body.total).toBe(5);
    expect(res.body.items[3].pathRef).toEqual({ slug: 'beta-co-ban', title: 'beta-co-ban', position: 2, total: 2 });
    expect(res.body.items[4]).not.toHaveProperty('pathRef');
    const page2 = await request(app).get('/api/v1/articles?sort=path&pageSize=2&page=2').expect(200);
    expect(page2.body.items.map((x: { id: string }) => x.id)).toEqual([b1, b2].map((x) => x._id.toString()));
  });

  it('sort=path: priority trong chủ đề thắng cấp độ; phần không có priority xếp sau, giữa chúng theo cấp độ rồi slug', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory({ slug: 'testing', order: 1 });
    const [x, y, z, w] = await Promise.all([1, 2, 3, 4].map(() => createArticle(cat, admin)));
    const mk = async (slug: string, level: 'beginner' | 'intermediate' | 'advanced', ids: (typeof x)['_id'][], priority?: number) => {
      const p = await LearningPath.create({ slug, title: slug, level, priority, categoryId: cat._id, articleIds: ids, status: 'published' });
      await syncLearningPathArticles(p._id, ids, []);
    };
    // Không có priority, phần intermediate sẽ đứng trước phần advanced; priority đảo lại đúng thứ tự học.
    await mk('he-thong-phan-1', 'intermediate', [z._id], 70);
    await mk('unit-component', 'advanced', [y._id], 60);
    await mk('nen-tang', 'beginner', [x._id], 10);
    await mk('chua-xep', 'beginner', [w._id]);

    const res = await request(app).get('/api/v1/articles?sort=path').expect(200);
    expect(res.body.items.map((i: { id: string }) => i.id)).toEqual([x, y, z, w].map((a) => a._id.toString()));
    const paths = await request(app).get('/api/v1/learning-paths').expect(200);
    const bySlug = Object.fromEntries(paths.body.map((p: { slug: string; priority?: number }) => [p.slug, p]));
    expect(bySlug['unit-component'].priority).toBe(60);
    expect(bySlug['chua-xep']).not.toHaveProperty('priority');
  });

  it('chi tiết kèm lộ trình (chỉ bài đã xuất bản, đúng thứ tự)', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory();
    const a = await createArticle(cat, admin);
    const b = await createArticle(cat, admin, { status: 'draft' });
    const c = await createArticle(cat, admin);
    const path = await LearningPath.create({ slug: 'lo-trinh', title: 'Lộ trình', level: 'beginner', articleIds: [c._id, b._id, a._id], status: 'published' });
    const res = await request(app).get(`/api/v1/articles/${a.slug}`).expect(200);
    expect(res.body.learningPath).toMatchObject({ id: path._id.toString(), slug: 'lo-trinh', level: 'beginner' });
    expect(res.body.learningPath.articles.map((x: { id: string }) => x.id)).toEqual([c._id.toString(), a._id.toString()]);

    const lp = await request(app).get('/api/v1/learning-paths/lo-trinh').expect(200);
    expect(lp.body.articles.map((x: { id: string }) => x.id)).toEqual([c._id.toString(), a._id.toString()]);
    expect(lp.body.articleCount).toBe(2);
    expect(lp.body.totalMinutes).toBe(a.readingMinutes + c.readingMinutes);
    const list = await request(app).get('/api/v1/learning-paths?level=beginner').expect(200);
    expect(list.body).toHaveLength(1);
    expect(list.body[0]).not.toHaveProperty('articles');
    await LearningPath.updateOne({ _id: path._id }, { $set: { status: 'draft' } });
    await request(app).get('/api/v1/learning-paths/lo-trinh').expect(404);
  });

  it('liên quan: cùng chủ đề, ưu tiên trùng tag, bỏ chính nó; tag phổ biến; tổng quan site; chủ đề', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory({ order: 2 });
    const other = await createCategory({ order: 1 });
    const base = await createArticle(cat, admin, { tags: ['a', 'b'] });
    const near = await createArticle(cat, admin, { tags: ['a', 'b'] });
    const far = await createArticle(cat, admin, { tags: ['z'] });
    await createArticle(other, admin, { tags: ['a', 'b'] });
    const rel = await request(app).get(`/api/v1/articles/${base.slug}/related?limit=5`).expect(200);
    expect(rel.body.map((x: { id: string }) => x.id)).toEqual([near._id.toString(), far._id.toString()]);

    const tags = await request(app).get('/api/v1/tags/popular?limit=2').expect(200);
    expect(tags.body).toEqual([
      { tag: 'a', count: 3 },
      { tag: 'b', count: 3 },
    ]);

    const summary = await request(app).get('/api/v1/site/summary').expect(200);
    expect(summary.body).toEqual({ articleCount: 4, categoryCount: 2, learningPathCount: 0, memberCount: 1, totalViews: 0 });

    const cats = await request(app).get('/api/v1/categories').expect(200);
    expect(cats.body.map((c: { id: string }) => c.id)).toEqual([other._id.toString(), cat._id.toString()]);
    expect(Object.keys(cats.body[0]).sort()).toEqual(['articleCount', 'color', 'description', 'icon', 'id', 'name', 'order', 'slug']);
    await request(app).get(`/api/v1/categories/${cat.slug}`).expect(200);
    await request(app).get('/api/v1/categories/khong-co').expect(404);
  });

  it('đăng ký nhận tin: mới ⇒ 201, trùng ⇒ 200, không lộ', async () => {
    await request(app).post('/api/v1/subscribers').send({ email: 'Doc@Gia.vn' }).expect(201, { ok: true });
    await request(app).post('/api/v1/subscribers').send({ email: 'doc@gia.vn' }).expect(200, { ok: true });
    await request(app).post('/api/v1/subscribers').send({ email: 'sai' }).expect(400);
  });
});

describe('lượt xem + truy cập', () => {
  beforeEach(resetDb);

  it('chống đếm trùng 30 phút theo (bài, khách); khách khác vẫn được đếm; bot không đếm', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory();
    const art = await createArticle(cat, admin);
    const v1 = visitorCookie(randomUUID());
    const v2 = visitorCookie(randomUUID());
    const url = `/api/v1/articles/${art.slug}/view`;

    expect((await request(app).post(url).set('Cookie', v1).send({}).expect(200)).body).toEqual({ counted: true, views: 1 });
    expect((await request(app).post(url).set('Cookie', v1).send({}).expect(200)).body).toEqual({ counted: false, views: 1 });
    expect((await request(app).post(url).set('Cookie', v2).send({}).expect(200)).body).toEqual({ counted: true, views: 2 });
    const bot = await request(app).post(url).set('User-Agent', 'Googlebot/2.1').set('Cookie', visitorCookie(randomUUID())).send({}).expect(200);
    expect(bot.body).toEqual({ counted: false, views: 2 });

    // Bấm liên tục cùng lúc vẫn chỉ đếm một lần.
    const v3 = visitorCookie(randomUUID());
    const burst = await Promise.all(Array.from({ length: 5 }, () => request(app).post(url).set('Cookie', v3).send({})));
    expect(burst.filter((r) => r.body.counted)).toHaveLength(1);

    // Sau 30 phút ⇒ được đếm lại.
    await ArticleView.updateMany({}, { $set: { createdAt: new Date(Date.now() - 31 * 60 * 1000) } });
    expect((await request(app).post(url).set('Cookie', v1).send({}).expect(200)).body).toEqual({ counted: true, views: 4 });

    const daily = await DailyArticleStat.findOne({ articleId: art._id, date: vnDate() }).lean();
    expect(daily!.views).toBe(4);
    const draft = await createArticle(cat, admin, { status: 'draft' });
    await request(app).post(`/api/v1/articles/${draft.slug}/view`).send({}).expect(404);
  });

  it('track/visit: tăng visits mỗi lần, khách duy nhất một lần/ngày, bot bỏ qua', async () => {
    const vid = visitorCookie(randomUUID());
    await request(app).post('/api/v1/track/visit').set('Cookie', vid).send({ path: '/' }).expect(204);
    await request(app).post('/api/v1/track/visit').set('Cookie', vid).send({ path: '/bai-viet' }).expect(204);
    await request(app).post('/api/v1/track/visit').set('Cookie', visitorCookie(randomUUID())).send({ path: '/' }).expect(204);
    await request(app).post('/api/v1/track/visit').set('User-Agent', 'facebookexternalhit preview').send({ path: '/' }).expect(204);
    await request(app).post('/api/v1/track/visit').send({}).expect(400);
    const stat = await DailySiteStat.findOne({ date: vnDate(), isDemo: false }).lean();
    expect(stat).toMatchObject({ visits: 3, uniqueVisitors: 2 });
  });

  it('trending: lượt xem 7 ngày, bù bằng popular khi thiếu', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory();
    const hot = await createArticle(cat, admin);
    const popular = await createArticle(cat, admin, { stats: { views: 999, comments: 0, reactions: { love: 0, helpful: 0, insightful: 0, inspiring: 0 } } });
    const old = await createArticle(cat, admin);
    await DailyArticleStat.create({ date: vnDate(), articleId: hot._id, isDemo: false, views: 7 });
    await DailyArticleStat.create({ date: '2020-01-01', articleId: old._id, isDemo: false, views: 5000 });
    const res = await request(app).get('/api/v1/articles/trending?limit=2').expect(200);
    expect(res.body.map((x: { id: string }) => x.id)).toEqual([hot._id.toString(), popular._id.toString()]);
    const list = await request(app).get('/api/v1/articles?sort=trending&pageSize=1').expect(200);
    expect(list.body.items[0].id).toBe(hot._id.toString());
    expect(list.body.total).toBe(3);
    await request(app).get('/api/v1/articles/trending?limit=13').expect(400);
  });
});

describe('reaction', () => {
  beforeEach(resetDb);

  it('PUT/DELETE idempotent, đếm đúng, nhiều loại, chống bấm đúp', async () => {
    const admin = await createUser('admin');
    const reader = await createUser('reader');
    const cat = await createCategory();
    const art = await createArticle(cat, admin);
    const base = `/api/v1/articles/${art.slug}/reactions`;
    const cookie = sessionCookie(reader);

    await request(app).put(`${base}/love`).send({}).expect(401);
    const first = await request(app).put(`${base}/love`).set('Cookie', cookie).send({}).expect(200);
    expect(first.body).toEqual({ reactions: { love: 1, helpful: 0, insightful: 0, inspiring: 0 }, myReactions: ['love'] });
    await request(app).put(`${base}/love`).set('Cookie', cookie).send({}).expect(200);
    const burst = await Promise.all(Array.from({ length: 5 }, () => request(app).put(`${base}/helpful`).set('Cookie', cookie).send({})));
    expect(burst.every((r) => r.status === 200)).toBe(true);
    const two = await request(app).put(`${base}/helpful`).set('Cookie', cookie).send({}).expect(200);
    expect(two.body).toEqual({ reactions: { love: 1, helpful: 1, insightful: 0, inspiring: 0 }, myReactions: ['love', 'helpful'] });

    const other = await request(app).put(`${base}/love`).set('Cookie', sessionCookie(admin)).send({}).expect(200);
    expect(other.body.reactions.love).toBe(2);
    expect(other.body.myReactions).toEqual(['love']);

    const del = await request(app).delete(`${base}/love`).set('Cookie', cookie).send({}).expect(200);
    expect(del.body).toEqual({ reactions: { love: 1, helpful: 1, insightful: 0, inspiring: 0 }, myReactions: ['helpful'] });
    await request(app).delete(`${base}/love`).set('Cookie', cookie).send({}).expect(200);
    const detail = await request(app).get(`/api/v1/articles/${art.slug}`).set('Cookie', cookie).expect(200);
    expect(detail.body.stats.reactions).toEqual({ love: 1, helpful: 1, insightful: 0, inspiring: 0 });
    expect(detail.body.myReactions).toEqual(['helpful']);

    const bad = await request(app).put(`${base}/angry`).set('Cookie', cookie).send({}).expect(400);
    expect(bad.body.error.code).toBe('VALIDATION');
  });
});
