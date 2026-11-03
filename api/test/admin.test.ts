import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { addDays, vnDate } from '../src/lib/dates.js';
import { Article, Category, Comment, DailyArticleStat, DailySiteStat, Reaction } from '../src/models/index.js';
import { assertCleanJson, createArticle, createCategory, createUser, makeApp, resetDb, sessionCookie, useTestDb } from './helpers.js';

useTestDb();
const app = makeApp();

const articleInput = (categoryId: string, extra: Record<string, unknown> = {}) => ({
  title: 'Đồng bộ dữ liệu: Đường đi của sự kiện!',
  summary: 'Tóm tắt',
  content: 'Một hai ba bốn năm.\n\n```js\nconst a = 1;\n```\n<svg><text>bỏ qua chữ này</text></svg>',
  language: 'vi',
  categoryId,
  level: 'advanced',
  tags: ['Kafka', 'kafka', 'Event'],
  ...extra,
});

describe('quản trị — phân quyền', () => {
  beforeEach(resetDb);

  it('reader 403; author chỉ bài của mình và 403 ở mục khác; admin toàn quyền', async () => {
    const admin = await createUser('admin');
    const author = await createUser('author');
    const reader = await createUser('reader');
    const cat = await createCategory();
    const mine = await createArticle(cat, author, { status: 'draft', publishedAt: undefined });
    const others = await createArticle(cat, admin);

    await request(app).get('/api/v1/admin/articles').expect(401);
    for (const path of ['/admin/articles', '/admin/stats/overview', '/admin/categories', '/admin/users']) {
      const r = await request(app).get(`/api/v1${path}`).set('Cookie', sessionCookie(reader)).expect(403);
      expect(r.body.error.code).toBe('FORBIDDEN');
    }

    const a = sessionCookie(author);
    const list = await request(app).get('/api/v1/admin/articles').set('Cookie', a).expect(200);
    expect(list.body.items.map((x: { id: string }) => x.id)).toEqual([mine._id.toString()]);
    expect(list.body.items[0]).not.toHaveProperty('content');
    expect(list.body.items[0]).not.toHaveProperty('publishedAt');
    expect(list.body.items[0].status).toBe('draft');
    await request(app).get(`/api/v1/admin/articles/${mine._id}`).set('Cookie', a).expect(200);
    await request(app).get(`/api/v1/admin/articles/${others._id}`).set('Cookie', a).expect(403);
    await request(app).put(`/api/v1/admin/articles/${others._id}`).set('Cookie', a).send(articleInput(cat._id.toString())).expect(403);
    await request(app).post(`/api/v1/admin/articles/${others._id}/unpublish`).set('Cookie', a).send({}).expect(403);
    await request(app).delete(`/api/v1/admin/articles/${others._id}`).set('Cookie', a).send({}).expect(403);
    for (const path of ['/admin/stats/overview', '/admin/categories', '/admin/learning-paths', '/admin/comments', '/admin/users', '/admin/subscribers']) {
      await request(app).get(`/api/v1${path}`).set('Cookie', a).expect(403);
    }
    const created = await request(app).post('/api/v1/admin/articles').set('Cookie', a).send(articleInput(cat._id.toString())).expect(201);
    expect(created.body.author.id).toBe(author._id.toString());

    const all = await request(app).get('/api/v1/admin/articles').set('Cookie', sessionCookie(admin)).expect(200);
    expect(all.body.total).toBe(3);
    await request(app).get('/api/v1/admin/articles/khong-hop-le').set('Cookie', sessionCookie(admin)).expect(404);
  });
});

describe('quản trị — bài viết', () => {
  beforeEach(resetDb);

  it('tạo bài: slug bỏ dấu, readingMinutes, tag chữ thường; xuất bản/gỡ/lưu trữ cập nhật articleCount', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory();
    const c = sessionCookie(admin);
    const res = await request(app).post('/api/v1/admin/articles').set('Cookie', c).send(articleInput(cat._id.toString())).expect(201);
    expect(res.body).toMatchObject({
      slug: 'dong-bo-du-lieu-duong-di-cua-su-kien',
      status: 'draft',
      readingMinutes: 1,
      tags: ['kafka', 'event'],
      stats: { views: 0, comments: 0, reactions: { love: 0, helpful: 0, insightful: 0, inspiring: 0 } },
    });
    expect(res.body).not.toHaveProperty('publishedAt');
    expect(res.body.content).toContain('Một hai ba');
    assertCleanJson(res.body);
    const id = res.body.id as string;

    const second = await request(app).post('/api/v1/admin/articles').set('Cookie', c).send(articleInput(cat._id.toString())).expect(201);
    expect(second.body.slug).toBe('dong-bo-du-lieu-duong-di-cua-su-kien-2');
    const taken = await request(app).put(`/api/v1/admin/articles/${second.body.id}`).set('Cookie', c).send(articleInput(cat._id.toString(), { slug: res.body.slug })).expect(409);
    expect(taken.body.error.code).toBe('SLUG_TAKEN');
    const badSlug = await request(app).post('/api/v1/admin/articles').set('Cookie', c).send(articleInput(cat._id.toString(), { slug: 'Có Dấu' })).expect(400);
    expect(badSlug.body.error.details.fields.slug).toBeTruthy();
    const badCat = await request(app).post('/api/v1/admin/articles').set('Cookie', c).send(articleInput('000000000000000000000000')).expect(400);
    expect(badCat.body.error.details.fields.categoryId).toBeTruthy();
    const tooMany = await request(app).post('/api/v1/admin/articles').set('Cookie', c).send(articleInput(cat._id.toString(), { tags: Array.from({ length: 11 }, (_, i) => `t${i}`) })).expect(400);
    expect(tooMany.body.error.details.fields.tags).toBeTruthy();

    const pub = await request(app).post(`/api/v1/admin/articles/${id}/publish`).set('Cookie', c).send({}).expect(200);
    expect(pub.body.status).toBe('published');
    const publishedAt = pub.body.publishedAt;
    expect(publishedAt).toMatch(/Z$/);
    expect((await Category.findById(cat._id).lean())!.articleCount).toBe(1);
    await request(app).get(`/api/v1/articles/${res.body.slug}`).expect(200);

    // Đổi chủ đề bài đã xuất bản ⇒ cả hai chủ đề được tính lại.
    const cat2 = await createCategory();
    await request(app).put(`/api/v1/admin/articles/${id}`).set('Cookie', c).send(articleInput(cat2._id.toString(), { content: 'từ '.repeat(450) })).expect(200);
    expect((await Category.findById(cat._id).lean())!.articleCount).toBe(0);
    expect((await Category.findById(cat2._id).lean())!.articleCount).toBe(1);
    const updated = await request(app).get(`/api/v1/admin/articles/${id}`).set('Cookie', c).expect(200);
    expect(updated.body.readingMinutes).toBe(3);
    expect(updated.body.slug).toBe(res.body.slug);

    await request(app).post(`/api/v1/admin/articles/${id}/unpublish`).set('Cookie', c).send({}).expect(200);
    expect((await Category.findById(cat2._id).lean())!.articleCount).toBe(0);
    await request(app).get(`/api/v1/articles/${res.body.slug}`).expect(404);
    const repub = await request(app).post(`/api/v1/admin/articles/${id}/publish`).set('Cookie', c).send({}).expect(200);
    expect(repub.body.publishedAt).toBe(publishedAt);

    await request(app).delete(`/api/v1/admin/articles/${id}`).set('Cookie', c).send({}).expect(204);
    expect((await Article.findById(id).lean())!.status).toBe('archived');
    expect((await Category.findById(cat2._id).lean())!.articleCount).toBe(0);
    const list = await request(app).get('/api/v1/admin/articles').set('Cookie', c).expect(200);
    expect(list.body.items.map((x: { id: string }) => x.id)).not.toContain(id);
    const archived = await request(app).get('/api/v1/admin/articles?status=archived').set('Cookie', c).expect(200);
    expect(archived.body.items.map((x: { id: string }) => x.id)).toEqual([id]);
  });

  it('chủ đề: CRUD, slug trùng 409, còn bài ⇒ 409 CATEGORY_IN_USE', async () => {
    const admin = await createUser('admin');
    const c = sessionCookie(admin);
    const input = { slug: 'he-thong', name: { vi: 'Hệ thống', en: 'Systems' }, description: { vi: 'mô tả', en: 'desc' }, icon: 'system', color: '#123ABC', order: 3 };
    const created = await request(app).post('/api/v1/admin/categories').set('Cookie', c).send(input).expect(201);
    expect(created.body).toMatchObject({ ...input, articleCount: 0 });
    await request(app).post('/api/v1/admin/categories').set('Cookie', c).send(input).expect(409);
    const bad = await request(app).post('/api/v1/admin/categories').set('Cookie', c).send({ ...input, slug: 'x', icon: 'rocket', color: 'red' }).expect(400);
    expect(Object.keys(bad.body.error.details.fields).sort()).toEqual(['color', 'icon']);
    const upd = await request(app).put(`/api/v1/admin/categories/${created.body.id}`).set('Cookie', c).send({ ...input, order: 9 }).expect(200);
    expect(upd.body.order).toBe(9);
    await createArticle({ ...(await Category.findById(created.body.id).lean())! }, admin, { status: 'draft' });
    const inUse = await request(app).delete(`/api/v1/admin/categories/${created.body.id}`).set('Cookie', c).send({}).expect(409);
    expect(inUse.body.error.code).toBe('CATEGORY_IN_USE');
    const empty = await createCategory();
    await request(app).delete(`/api/v1/admin/categories/${empty._id}`).set('Cookie', c).send({}).expect(204);
    await request(app).delete(`/api/v1/admin/categories/${empty._id}`).set('Cookie', c).send({}).expect(404);
  });

  it('lộ trình: một bài thuộc tối đa một lộ trình, totalMinutes chỉ tính bài đã xuất bản', async () => {
    const admin = await createUser('admin');
    const c = sessionCookie(admin);
    const cat = await createCategory();
    const a = await createArticle(cat, admin);
    const b = await createArticle(cat, admin);
    const draft = await createArticle(cat, admin, { status: 'draft' });
    const base = { title: 'Lộ trình A', description: 'mô tả', level: 'beginner', status: 'published' };
    const p1 = await request(app)
      .post('/api/v1/admin/learning-paths')
      .set('Cookie', c)
      .send({ ...base, slug: 'lo-trinh-a', categoryId: cat._id.toString(), articleIds: [a._id.toString(), draft._id.toString(), b._id.toString()] })
      .expect(201);
    expect(p1.body).toMatchObject({ slug: 'lo-trinh-a', status: 'published', categoryId: cat._id.toString(), articleCount: 2, totalMinutes: a.readingMinutes + b.readingMinutes });
    expect(p1.body.articleIds).toEqual([a._id.toString(), draft._id.toString(), b._id.toString()]);
    expect(p1.body.category.slug).toBe(cat.slug);

    const p2 = await request(app).post('/api/v1/admin/learning-paths').set('Cookie', c).send({ ...base, slug: 'lo-trinh-b', articleIds: [b._id.toString()] }).expect(201);
    const p1After = await request(app).get(`/api/v1/admin/learning-paths/${p1.body.id}`).set('Cookie', c).expect(200);
    expect(p1After.body.articleIds).toEqual([a._id.toString(), draft._id.toString()]);
    expect((await Article.findById(b._id).lean())!.learningPathId!.toString()).toBe(p2.body.id);
    const adminArt = await request(app).get(`/api/v1/admin/articles/${b._id}`).set('Cookie', c).expect(200);
    expect(adminArt.body.learningPathId).toBe(p2.body.id);

    await request(app).post('/api/v1/admin/learning-paths').set('Cookie', c).send({ ...base, slug: 'lo-trinh-b', articleIds: [] }).expect(409);
    await request(app).post('/api/v1/admin/learning-paths').set('Cookie', c).send({ ...base, slug: 'lo-trinh-c', articleIds: ['000000000000000000000000'] }).expect(400);

    await request(app).put(`/api/v1/admin/learning-paths/${p1.body.id}`).set('Cookie', c).send({ ...base, slug: 'lo-trinh-a', articleIds: [draft._id.toString()] }).expect(200);
    expect((await Article.findById(a._id).lean())!.learningPathId).toBeUndefined();
    await request(app).delete(`/api/v1/admin/learning-paths/${p2.body.id}`).set('Cookie', c).send({}).expect(204);
    expect((await Article.findById(b._id).lean())!.learningPathId).toBeUndefined();
    const list = await request(app).get('/api/v1/admin/learning-paths').set('Cookie', c).expect(200);
    expect(list.body).toHaveLength(1);
  });

  it('người dùng: lọc, đổi vai trò/khoá; admin không tự hạ/khoá mình', async () => {
    const admin = await createUser('admin');
    const reader = await createUser('reader', { displayName: 'Người Đọc' });
    const c = sessionCookie(admin);
    const list = await request(app).get('/api/v1/admin/users?role=reader').set('Cookie', c).expect(200);
    expect(list.body.total).toBe(1);
    expect(list.body.items[0]).toMatchObject({ id: reader._id.toString(), email: reader.email, commentCount: 0, status: 'active' });
    expect((await request(app).get('/api/v1/admin/users?q=ng%C6%B0%E1%BB%9Di%20%C4%91%E1%BB%8Dc').set('Cookie', c).expect(200)).body.total).toBe(1);

    const up = await request(app).patch(`/api/v1/admin/users/${reader._id}`).set('Cookie', c).send({ role: 'author', status: 'locked' }).expect(200);
    expect(up.body).toMatchObject({ role: 'author', status: 'locked' });
    await request(app).get('/api/v1/auth/me').set('Cookie', sessionCookie(reader)).expect(401);

    for (const body of [{ role: 'reader' }, { status: 'locked' }]) {
      const r = await request(app).patch(`/api/v1/admin/users/${admin._id}`).set('Cookie', c).send(body).expect(409);
      expect(r.body.error.code).toBe('CANNOT_MODIFY_SELF');
    }
    await request(app).patch(`/api/v1/admin/users/${admin._id}`).set('Cookie', c).send({ role: 'admin', status: 'active' }).expect(200);
    await request(app).patch(`/api/v1/admin/users/${reader._id}`).set('Cookie', c).send({ role: 'god' }).expect(400);
  });

  it('bình luận + đăng ký tin ở admin', async () => {
    const admin = await createUser('admin');
    const reader = await createUser('reader');
    const cat = await createCategory();
    const art = await createArticle(cat, admin);
    const c = sessionCookie(admin);
    const made = await request(app).post(`/api/v1/articles/${art.slug}/comments`).set('Cookie', sessionCookie(reader)).send({ body: 'tìm thấy tôi' }).expect(201);
    await request(app).patch(`/api/v1/admin/comments/${made.body.id}`).set('Cookie', c).send({ status: 'hidden' }).expect(200);
    const list = await request(app).get(`/api/v1/admin/comments?status=hidden&q=t%C3%ACm&articleId=${art._id}`).set('Cookie', c).expect(200);
    expect(list.body.total).toBe(1);
    expect(list.body.items[0]).toMatchObject({ status: 'hidden', body: 'tìm thấy tôi', article: { id: art._id.toString(), slug: art.slug, title: art.title } });
    await request(app).delete(`/api/v1/admin/comments/${made.body.id}`).set('Cookie', c).send({}).expect(204);
    expect((await Comment.findById(made.body.id).lean())!.status).toBe('deleted');

    await request(app).post('/api/v1/subscribers').send({ email: 'a@b.vn' }).expect(201);
    const subs = await request(app).get('/api/v1/admin/subscribers?page=1').set('Cookie', c).expect(200);
    expect(subs.body.items[0]).toMatchObject({ email: 'a@b.vn' });
    expect(Object.keys(subs.body.items[0]).sort()).toEqual(['createdAt', 'email', 'id']);
  });
});

describe('quản trị — thống kê', () => {
  beforeEach(resetDb);

  it('overview: đủ ngày (ngày trống = 0), kỳ trước, top bài', async () => {
    const admin = await createUser('admin');
    const reader = await createUser('reader');
    const cat = await createCategory();
    const a = await createArticle(cat, admin);
    const b = await createArticle(cat, admin);
    await createArticle(cat, admin, { status: 'draft' });
    const today = vnDate();
    await DailySiteStat.create({ date: today, isDemo: false, visits: 10, uniqueVisitors: 4 });
    await DailySiteStat.create({ date: today, isDemo: true, visits: 5, uniqueVisitors: 1 });
    await DailySiteStat.create({ date: addDays(today, -8), isDemo: false, visits: 7, uniqueVisitors: 3 });
    await DailyArticleStat.create({ date: today, articleId: a._id, isDemo: false, views: 3 });
    await DailyArticleStat.create({ date: addDays(today, -1), articleId: b._id, isDemo: false, views: 9 });
    await DailyArticleStat.create({ date: addDays(today, -10), articleId: b._id, isDemo: false, views: 100 });
    await request(app).post(`/api/v1/articles/${a.slug}/comments`).set('Cookie', sessionCookie(reader)).send({ body: 'hi' }).expect(201);
    await Reaction.create({ articleId: a._id, userId: reader._id, type: 'love' });
    await request(app).post('/api/v1/subscribers').send({ email: 's@b.vn' }).expect(201);

    const res = await request(app).get('/api/v1/admin/stats/overview?days=7').set('Cookie', sessionCookie(admin)).expect(200);
    const body = res.body;
    expect(body.range).toEqual({ from: addDays(today, -6), to: today, days: 7 });
    expect(body.series).toHaveLength(7);
    expect(body.series[0]).toEqual({ date: addDays(today, -6), views: 0, visits: 0, uniqueVisitors: 0, comments: 0, reactions: 0 });
    expect(body.series[6]).toEqual({ date: today, views: 3, visits: 15, uniqueVisitors: 5, comments: 1, reactions: 1 });
    expect(body.totals).toEqual({ articles: 3, published: 2, drafts: 1, users: 2, comments: 1, subscribers: 1, views: 12, visits: 15, uniqueVisitors: 5 });
    expect(body.previous).toEqual({ views: 100, visits: 7, uniqueVisitors: 3, comments: 0 });
    expect(body.topArticles).toEqual([
      { id: b._id.toString(), slug: b.slug, title: b.title, views: 9, comments: 0, reactions: 0 },
      { id: a._id.toString(), slug: a.slug, title: a.title, views: 3, comments: 1, reactions: 1 },
    ]);

    const def = await request(app).get('/api/v1/admin/stats/overview').set('Cookie', sessionCookie(admin)).expect(200);
    expect(def.body.series).toHaveLength(30);
    await request(app).get('/api/v1/admin/stats/overview?days=6').set('Cookie', sessionCookie(admin)).expect(400);
    await request(app).get('/api/v1/admin/stats/overview?days=181').set('Cookie', sessionCookie(admin)).expect(400);
  });
});
