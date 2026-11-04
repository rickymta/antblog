import path from 'node:path';
import { fileURLToPath } from 'node:url';
import bcrypt from 'bcryptjs';
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import {
  Article,
  Category,
  Comment,
  CommentLike,
  DailyArticleStat,
  DailySiteStat,
  LearningPath,
  Reaction,
  User,
} from '../src/models/index.js';
import { createDemo, removeDemo } from '../seed/demoLib.js';
import { runSeed } from '../seed/lib.js';
import { makeApp, resetDb, sessionCookie, useTestDb } from './helpers.js';

useTestDb();
const dataDir = path.join(path.dirname(fileURLToPath(import.meta.url)), 'fixtures', 'seed-data');
const admin = { email: 'Admin@TKH.local', password: 'mat-khau-admin-1', name: 'Quản trị viên' };
const quiet = () => undefined;

async function counts() {
  return {
    users: await User.countDocuments(),
    categories: await Category.countDocuments(),
    articles: await Article.countDocuments(),
    paths: await LearningPath.countDocuments(),
  };
}

describe('seed', () => {
  beforeEach(resetDb);

  it('chạy hai lần không nhân đôi, giữ publishedAt và mật khẩu admin, tính lại bộ đếm', async () => {
    const first = await runSeed({ dataDir, admin, log: quiet });
    expect(first).toMatchObject({ adminCreated: true, categories: 2, learningPaths: 1, articles: { created: 2, updated: 0, skipped: 1 } });
    const c1 = await counts();
    expect(c1).toEqual({ users: 1, categories: 2, articles: 2, paths: 1 });
    const art1 = await Article.findOne({ slug: 'chi-muc-co-ban' }).lean();
    expect(art1).toMatchObject({ status: 'published', tags: ['index', 'mongodb'], readingMinutes: 1 });
    const adminDoc = await User.findOne({ email: 'admin@tkh.local' }).lean();
    expect(adminDoc!.role).toBe('admin');
    expect(art1!.authorId.equals(adminDoc!._id)).toBe(true);
    expect((await Category.findOne({ slug: 'database' }).lean())!.articleCount).toBe(2);
    expect((await Category.findOne({ slug: 'database' }).lean())!.color).toBe('#2563EB');
    const path1 = await LearningPath.findOne({ slug: 'nhap-mon-csdl' }).lean();
    expect(path1!.articleIds.map(String)).toEqual([art1!._id.toString(), (await Article.findOne({ slug: 'giao-dich' }).lean())!._id.toString()]);
    expect((await Article.findById(art1!._id).lean())!.learningPathId!.toString()).toBe(path1!._id.toString());

    await Article.updateOne({ _id: art1!._id }, { $inc: { 'stats.views': 5 } });
    const second = await runSeed({ dataDir, admin: { ...admin, password: 'mat-khau-khac-2' }, log: quiet });
    expect(second).toMatchObject({ adminCreated: false, articles: { created: 0, updated: 2, skipped: 1 } });
    expect(await counts()).toEqual(c1);
    const art2 = await Article.findById(art1!._id).lean();
    expect(art2!.publishedAt!.toISOString()).toBe(art1!.publishedAt!.toISOString());
    expect(art2!.stats.views).toBe(5);
    const adminAfter = await User.findOne({ email: 'admin@tkh.local' }).lean();
    expect(await bcrypt.compare(admin.password, adminAfter!.passwordHash)).toBe(true);
    expect((await Category.findOne({ slug: 'database' }).lean())!.articleCount).toBe(2);

    const app = makeApp();
    const summary = await request(app).get('/api/v1/site/summary').expect(200);
    expect(summary.body).toMatchObject({ articleCount: 2, categoryCount: 2, learningPathCount: 1 });
  });

  it('bài đã sửa trong CMS: seed giữ nguyên + log, --force-content ghi đè lại; bài chưa sửa vẫn cập nhật', async () => {
    await runSeed({ dataDir, admin, log: quiet });
    const adminDoc = await User.findOne({ email: 'admin@tkh.local' }).lean();
    const edited = await Article.findOne({ slug: 'chi-muc-co-ban' }).lean();
    const untouched = await Article.findOne({ slug: 'giao-dich' }).lean();
    const app = makeApp();
    const c = sessionCookie(adminDoc!);
    await request(app)
      .put(`/api/v1/admin/articles/${edited!._id}`)
      .set('Cookie', c)
      .send({
        title: 'Tiêu đề sửa trong CMS',
        summary: 'Tóm tắt CMS',
        content: 'Nội dung viết lại trong CMS.',
        language: 'vi',
        categoryId: edited!.categoryId.toString(),
        level: 'advanced',
        tags: ['cms'],
      })
      .expect(200);
    await request(app).post(`/api/v1/admin/articles/${edited!._id}/unpublish`).set('Cookie', c).send({}).expect(200);
    // Bài chưa sửa trong CMS nhưng bị đổi trực tiếp trong DB ⇒ seed vẫn đưa về tệp .md.
    await Article.updateOne({ _id: untouched!._id }, { $set: { title: 'đổi tay' } });

    const logs: string[] = [];
    const again = await runSeed({ dataDir, admin, log: (m) => logs.push(m) });
    expect(again.articles).toMatchObject({ created: 0, updated: 1, keptCms: 1, skipped: 1 });
    expect(logs.some((l) => l.includes('chi-muc-co-ban') && l.includes('bỏ qua (đã sửa trong CMS)'))).toBe(true);
    const kept = await Article.findById(edited!._id).lean();
    expect(kept).toMatchObject({ title: 'Tiêu đề sửa trong CMS', content: 'Nội dung viết lại trong CMS.', level: 'advanced', tags: ['cms'], status: 'draft', editedInCms: true });
    expect((await Article.findById(untouched!._id).lean())!.title).toBe(untouched!.title);

    const forced = await runSeed({ dataDir, admin, log: quiet, forceContent: true });
    expect(forced.articles).toMatchObject({ created: 0, updated: 2, keptCms: 0 });
    const reset = await Article.findById(edited!._id).lean();
    expect(reset).toMatchObject({ title: edited!.title, content: edited!.content, level: edited!.level, tags: edited!.tags, status: 'published' });
    expect(reset!.editedInCms).toBeUndefined();
    expect(reset!.publishedAt!.toISOString()).toBe(edited!.publishedAt!.toISOString());
  });

  it('thiếu tệp dữ liệu ⇒ ghi log rõ ràng, vẫn tạo admin', async () => {
    const logs: string[] = [];
    await runSeed({ dataDir: path.join(dataDir, 'khong-co'), admin, log: (m) => logs.push(m) });
    expect(logs.some((l) => l.includes('categories.json'))).toBe(true);
    expect(logs.some((l) => l.includes('learning-paths.json'))).toBe(true);
    expect(await User.countDocuments()).toBe(1);
  });

  it('seed:demo tạo hoạt động nhận diện được; --remove gỡ sạch và trả bộ đếm về như cũ', async () => {
    await runSeed({ dataDir, admin, log: quiet });
    // Hoạt động thật có trước demo phải được giữ nguyên.
    const realUser = await User.findOne({ email: 'admin@tkh.local' }).lean();
    const art = await Article.findOne({ slug: 'chi-muc-co-ban' }).lean();
    const app = makeApp();
    const real = await request(app).post(`/api/v1/articles/${art!.slug}/comments`).set('Cookie', sessionCookie(realUser!)).send({ body: 'thật' }).expect(201);
    await request(app).put(`/api/v1/articles/${art!.slug}/reactions/love`).set('Cookie', sessionCookie(realUser!)).send({}).expect(200);
    await Article.updateOne({ _id: art!._id }, { $inc: { 'stats.views': 3 } });
    const before = await Article.find({}).sort({ slug: 1 }).lean();

    await createDemo(quiet);
    await createDemo(quiet); // chạy lại không nhân đôi
    expect(await User.countDocuments({ isDemo: true })).toBe(8);
    expect(await User.countDocuments({ email: /@demo\.local$/, isDemo: { $ne: true } })).toBe(0);
    const demoComments = await Comment.countDocuments({ isDemo: true });
    expect(demoComments).toBeGreaterThan(0);
    expect(await Reaction.countDocuments({ isDemo: true })).toBeGreaterThan(0);
    expect(await DailySiteStat.countDocuments({ isDemo: true })).toBe(30);
    const withDemo = await Article.findById(art!._id).lean();
    expect(withDemo!.stats.views).toBeGreaterThan(3);
    expect(withDemo!.stats.comments).toBe(await Comment.countDocuments({ articleId: art!._id, status: 'visible' }));
    const anyDemo = await Comment.findOne({ isDemo: true }).lean();
    expect(anyDemo!.likeCount).toBe(await CommentLike.countDocuments({ commentId: anyDemo!._id }));
    const overview = await request(app).get('/api/v1/admin/stats/overview?days=30').set('Cookie', sessionCookie(realUser!)).expect(200);
    expect(overview.body.totals.visits).toBeGreaterThan(0);
    expect(overview.body.topArticles.length).toBeGreaterThan(0);

    await removeDemo(quiet);
    expect(await User.countDocuments({ isDemo: true })).toBe(0);
    expect(await Comment.countDocuments({ isDemo: true })).toBe(0);
    expect(await Reaction.countDocuments({ isDemo: true })).toBe(0);
    expect(await DailySiteStat.countDocuments({ isDemo: true })).toBe(0);
    expect(await DailyArticleStat.countDocuments({ isDemo: true })).toBe(0);
    expect(await Comment.findById(real.body.id).lean()).toBeTruthy();
    const after = await Article.find({}).sort({ slug: 1 }).lean();
    expect(after.map((a) => a.stats)).toEqual(before.map((a) => a.stats));
  });
});
