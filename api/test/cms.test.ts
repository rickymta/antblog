import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { Article, Category, LearningPath } from '../src/models/index.js';
import { assertCleanJson, createArticle, createCategory, createUser, makeApp, resetDb, sessionCookie, useTestDb } from './helpers.js';

useTestDb();
const app = makeApp();

const input = (categoryId: string, extra: Record<string, unknown> = {}) => ({
  title: 'Bài viết thử SEO',
  summary: 'Tóm tắt',
  content: 'Nội dung bài.',
  language: 'vi',
  categoryId,
  level: 'beginner',
  tags: [],
  ...extra,
});

describe('CMS — SEO', () => {
  beforeEach(resetDb);

  it('seo: lưu/trả ở admin + chi tiết công khai, rỗng ⇒ gỡ, kiểm độ dài và URL', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory();
    const c = sessionCookie(admin);
    const seo = { title: '  Tiêu đề SEO  ', description: 'Mô tả SEO', ogImageUrl: '/uploads/2026/11/abc.png', noindex: true };
    const created = await request(app).post('/api/v1/admin/articles').set('Cookie', c).send(input(cat._id.toString(), { seo })).expect(201);
    expect(created.body.seo).toEqual({ ...seo, title: 'Tiêu đề SEO' });
    expect(created.body.editedInCms).toBe(true);
    const id = created.body.id as string;
    await request(app).post(`/api/v1/admin/articles/${id}/publish`).set('Cookie', c).send({}).expect(200);
    const pub = await request(app).get(`/api/v1/articles/${created.body.slug}`).expect(200);
    expect(pub.body.seo).toEqual({ ...seo, title: 'Tiêu đề SEO' });
    assertCleanJson(pub.body);

    // Ảnh bìa cũng nhận đường dẫn thư viện media; seo toàn trường rỗng ⇒ gỡ ghi đè.
    const upd = await request(app)
      .put(`/api/v1/admin/articles/${id}`)
      .set('Cookie', c)
      .send(input(cat._id.toString(), { coverImageUrl: '/uploads/2026/11/bia.webp', seo: { title: '', description: ' ', ogImageUrl: '', noindex: false } }))
      .expect(200);
    expect(upd.body).not.toHaveProperty('seo');
    expect(upd.body.coverImageUrl).toBe('/uploads/2026/11/bia.webp');
    expect((await Article.findById(id).lean())!.seo).toBeUndefined();
    const pub2 = await request(app).get(`/api/v1/articles/${created.body.slug}`).expect(200);
    expect(pub2.body).not.toHaveProperty('seo');

    const bad = await request(app)
      .put(`/api/v1/admin/articles/${id}`)
      .set('Cookie', c)
      .send(input(cat._id.toString(), { seo: { title: 'x'.repeat(71), description: 'y'.repeat(171), ogImageUrl: 'javascript:alert(1)' } }))
      .expect(400);
    expect(Object.keys(bad.body.error.details.fields).sort()).toEqual(['seo.description', 'seo.ogImageUrl', 'seo.title']);
    const traversal = await request(app)
      .put(`/api/v1/admin/articles/${id}`)
      .set('Cookie', c)
      .send(input(cat._id.toString(), { coverImageUrl: '/uploads/../.env' }))
      .expect(400);
    expect(traversal.body.error.details.fields.coverImageUrl).toBeTruthy();
    // Đúng 70 / 170 ký tự vẫn hợp lệ.
    const edge = await request(app)
      .put(`/api/v1/admin/articles/${id}`)
      .set('Cookie', c)
      .send(input(cat._id.toString(), { seo: { title: 'x'.repeat(70), description: 'y'.repeat(170) } }))
      .expect(200);
    expect(edge.body.seo).toEqual({ title: 'x'.repeat(70), description: 'y'.repeat(170) });
  });

  it('PUT đặt editedInCms; danh sách admin trả cờ', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory();
    const art = await createArticle(cat, admin);
    const c = sessionCookie(admin);
    const before = await request(app).get(`/api/v1/admin/articles/${art._id}`).set('Cookie', c).expect(200);
    expect(before.body).not.toHaveProperty('editedInCms');
    await request(app).put(`/api/v1/admin/articles/${art._id}`).set('Cookie', c).send(input(cat._id.toString())).expect(200);
    expect((await Article.findById(art._id).lean())!.editedInCms).toBe(true);
    const list = await request(app).get('/api/v1/admin/articles').set('Cookie', c).expect(200);
    expect(list.body.items[0].editedInCms).toBe(true);
  });
});

describe('CMS — lên lịch đăng', () => {
  beforeEach(resetDb);

  it('bài lên lịch ẩn ở mọi endpoint công khai, hiện khi tới giờ; admin thấy và lọc được', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory();
    const c = sessionCookie(admin);
    const live = await createArticle(cat, admin, { tags: ['chung'] });
    const created = await request(app)
      .post('/api/v1/admin/articles')
      .set('Cookie', c)
      .send(input(cat._id.toString(), { title: 'Bài lên lịch', tags: ['chung', 'rieng-lich'] }))
      .expect(201);
    const id = created.body.id as string;
    const slug = created.body.slug as string;
    const publishAt = new Date(Date.now() + 3 * 3600 * 1000).toISOString();
    const sched = await request(app).post(`/api/v1/admin/articles/${id}/publish`).set('Cookie', c).send({ publishAt }).expect(200);
    expect(sched.body).toMatchObject({ status: 'published', publishedAt: publishAt });
    await request(app)
      .post('/api/v1/admin/learning-paths')
      .set('Cookie', c)
      .send({ slug: 'lo-trinh', title: 'Lộ trình', description: '', level: 'beginner', status: 'published', articleIds: [live._id.toString(), id] })
      .expect(201);

    await request(app).get(`/api/v1/articles/${slug}`).expect(404);
    await request(app).get(`/api/v1/articles/${slug}/comments`).expect(404);
    const list0 = await request(app).get('/api/v1/articles').expect(200);
    expect(list0.body.items.map((a: { slug: string }) => a.slug)).toEqual([live.slug]);
    const trending = await request(app).get('/api/v1/articles/trending?limit=12').expect(200);
    expect(trending.body.map((a: { slug: string }) => a.slug)).not.toContain(slug);
    const related = await request(app).get(`/api/v1/articles/${live.slug}/related`).expect(200);
    expect(related.body).toEqual([]);
    const tags = await request(app).get('/api/v1/tags/popular').expect(200);
    expect(tags.body.map((t: { tag: string }) => t.tag)).not.toContain('rieng-lich');
    expect((await request(app).get('/api/v1/site/summary').expect(200)).body.articleCount).toBe(1);
    expect((await request(app).get('/api/v1/categories').expect(200)).body[0].articleCount).toBe(1);
    expect((await request(app).get(`/api/v1/categories/${cat.slug}`).expect(200)).body.articleCount).toBe(1);
    const path0 = await request(app).get('/api/v1/learning-paths/lo-trinh').expect(200);
    expect(path0.body.articles.map((a: { slug: string }) => a.slug)).toEqual([live.slug]);
    expect(path0.body.articleCount).toBe(1);
    const liveDetail = await request(app).get(`/api/v1/articles/${live.slug}`).expect(200);
    expect(liveDetail.body.learningPath.articles.map((a: { slug: string }) => a.slug)).toEqual([live.slug]);
    const sitemap0 = await request(app).get('/api/v1/site/sitemap').expect(200);
    expect(sitemap0.body.articles.map((a: { slug: string }) => a.slug)).toEqual([live.slug]);

    // Admin vẫn thấy: lọc "scheduled" chỉ ra bài lên lịch, "published" chỉ bài đang công khai.
    const scheduled = await request(app).get('/api/v1/admin/articles?status=scheduled').set('Cookie', c).expect(200);
    expect(scheduled.body.items.map((a: { id: string }) => a.id)).toEqual([id]);
    expect(scheduled.body.items[0]).toMatchObject({ status: 'published', publishedAt: publishAt });
    const published = await request(app).get('/api/v1/admin/articles?status=published').set('Cookie', c).expect(200);
    expect(published.body.items.map((a: { id: string }) => a.id)).toEqual([live._id.toString()]);
    expect((await request(app).get('/api/v1/admin/articles').set('Cookie', c).expect(200)).body.total).toBe(2);

    // Tới giờ (đặt thẳng publishedAt về quá khứ) ⇒ hiện ở mọi nơi, kể cả bộ đếm chủ đề.
    await Article.updateOne({ _id: id }, { $set: { publishedAt: new Date(Date.now() - 1000) } });
    await request(app).get(`/api/v1/articles/${slug}`).expect(200);
    expect((await request(app).get('/api/v1/articles').expect(200)).body.total).toBe(2);
    expect((await request(app).get('/api/v1/site/summary').expect(200)).body.articleCount).toBe(2);
    expect((await request(app).get('/api/v1/categories').expect(200)).body[0].articleCount).toBe(2);
    const sitemap = await request(app).get('/api/v1/site/sitemap').expect(200);
    expect(sitemap.body.articles.map((a: { slug: string }) => a.slug).sort()).toEqual([live.slug, slug].sort());
    expect((await request(app).get('/api/v1/learning-paths/lo-trinh').expect(200)).body.articleCount).toBe(2);
  });

  it('publishAt ở quá khứ hoặc sai định dạng ⇒ 400; xuất bản ngay bài đang lên lịch ⇒ publishedAt = bây giờ', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory();
    const c = sessionCookie(admin);
    const draft = await createArticle(cat, admin, { status: 'draft', publishedAt: undefined });
    const past = await request(app)
      .post(`/api/v1/admin/articles/${draft._id}/publish`)
      .set('Cookie', c)
      .send({ publishAt: new Date(Date.now() - 60_000).toISOString() })
      .expect(400);
    expect(past.body.error.code).toBe('VALIDATION');
    expect(past.body.error.details.fields.publishAt).toBeTruthy();
    const junk = await request(app).post(`/api/v1/admin/articles/${draft._id}/publish`).set('Cookie', c).send({ publishAt: 'ngày mai' }).expect(400);
    expect(junk.body.error.details.fields.publishAt).toBeTruthy();
    expect((await Article.findById(draft._id).lean())!.status).toBe('draft');

    const future = new Date(Date.now() + 86_400_000).toISOString();
    await request(app).post(`/api/v1/admin/articles/${draft._id}/publish`).set('Cookie', c).send({ publishAt: future }).expect(200);
    expect((await Category.findById(cat._id).lean())!.articleCount).toBe(0);
    const t0 = Date.now();
    const now = await request(app).post(`/api/v1/admin/articles/${draft._id}/publish`).set('Cookie', c).send({}).expect(200);
    expect(Date.parse(now.body.publishedAt)).toBeGreaterThanOrEqual(t0 - 1000);
    expect(Date.parse(now.body.publishedAt)).toBeLessThanOrEqual(Date.now());
    await request(app).get(`/api/v1/articles/${draft.slug}`).expect(200);
    expect((await Category.findById(cat._id).lean())!.articleCount).toBe(1);
  });

  it('author chỉ lên lịch bài của mình', async () => {
    const admin = await createUser('admin');
    const author = await createUser('author');
    const cat = await createCategory();
    const others = await createArticle(cat, admin, { status: 'draft', publishedAt: undefined });
    await request(app)
      .post(`/api/v1/admin/articles/${others._id}/publish`)
      .set('Cookie', sessionCookie(author))
      .send({ publishAt: new Date(Date.now() + 86_400_000).toISOString() })
      .expect(403);
  });
});

describe('sitemap', () => {
  beforeEach(resetDb);

  it('bài công khai (kèm noindex), mọi chủ đề, lộ trình đã xuất bản; bỏ nháp/lưu trữ', async () => {
    const admin = await createUser('admin');
    const cat = await createCategory();
    const emptyCat = await createCategory();
    const a = await createArticle(cat, admin);
    const b = await createArticle(cat, admin, { seo: { noindex: true } });
    await createArticle(cat, admin, { status: 'draft' });
    await createArticle(cat, admin, { status: 'archived' });
    await LearningPath.create({ slug: 'cong-khai', title: 'Công khai', level: 'beginner', status: 'published', articleIds: [a._id] });
    await LearningPath.create({ slug: 'nhap', title: 'Nháp', level: 'beginner', status: 'draft', articleIds: [] });

    const res = await request(app).get('/api/v1/site/sitemap').expect(200);
    assertCleanJson(res.body);
    const bySlug = new Map<string, unknown>(res.body.articles.map((x: { slug: string }) => [x.slug, x]));
    expect([...bySlug.keys()].sort()).toEqual([a.slug, b.slug].sort());
    expect(bySlug.get(a.slug)).toEqual({ slug: a.slug, updatedAt: new Date(a.updatedAt).toISOString() });
    expect(bySlug.get(b.slug)).toMatchObject({ slug: b.slug, noindex: true });
    expect(res.body.categories.map((x: { slug: string }) => x.slug).sort()).toEqual([cat.slug, emptyCat.slug].sort());
    for (const x of res.body.categories) expect(x.updatedAt).toMatch(/Z$/);
    expect(res.body.learningPaths).toEqual([{ slug: 'cong-khai', updatedAt: expect.stringMatching(/Z$/) }]);
  });
});
