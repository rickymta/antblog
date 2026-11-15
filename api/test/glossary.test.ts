import { cpSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { runSeed } from '../seed/lib.js';
import { Term } from '../src/models/index.js';
import { assertCleanJson, createArticle, createCategory, createUser, makeApp, resetDb, sessionCookie, useTestDb } from './helpers.js';

useTestDb();
const app = makeApp();
const fixtures = path.join(path.dirname(fileURLToPath(import.meta.url)), 'fixtures', 'seed-data');
const admin = { email: 'Admin@TKH.local', password: 'mat-khau-admin-1', name: 'Quản trị viên' };
const quiet = () => undefined;

const termInput = (extra: Record<string, unknown> = {}) => ({
  slug: 'smoke-test',
  term: 'smoke test',
  aliases: ['smoke testing', 'smoke testing', ' '],
  shortDefinition: 'Bộ kiểm thử nhỏ chạy nhanh trên luồng sống còn.',
  ...extra,
});

describe('từ điển keyword — API công khai', () => {
  beforeEach(resetDb);

  it('chỉ trả thuật ngữ đã xuất bản của đúng ngôn ngữ, không kèm body hay trường nội bộ', async () => {
    await Term.create([
      { slug: 'smoke-test', language: 'vi', term: 'smoke test', shortDefinition: 'A', body: '# Dài', editedInCms: true },
      { slug: 'khu-trung-lap', language: 'vi', term: 'khử trùng lặp', english: 'deduplication', aliases: ['deduplication'], shortDefinition: 'B' },
      { slug: 'nhap', language: 'vi', term: 'bản nháp', shortDefinition: 'C', status: 'draft' },
      { slug: 'smoke-test', language: 'en', term: 'smoke test', shortDefinition: 'EN' },
    ]);
    const list = await request(app).get('/api/v1/glossary').expect(200);
    assertCleanJson(list.body);
    expect(list.body).toEqual([
      { slug: 'khu-trung-lap', term: 'khử trùng lặp', english: 'deduplication', aliases: ['deduplication'], shortDefinition: 'B', caseSensitive: false, autoLink: true },
      { slug: 'smoke-test', term: 'smoke test', aliases: [], shortDefinition: 'A', caseSensitive: false, autoLink: true },
    ]);
    const en = await request(app).get('/api/v1/glossary?language=en').expect(200);
    expect(en.body.map((t: { shortDefinition: string }) => t.shortDefinition)).toEqual(['EN']);
    await request(app).get('/api/v1/glossary?language=fr').expect(400);

    const detail = await request(app).get('/api/v1/glossary/smoke-test').expect(200);
    expect(detail.body.body).toBe('# Dài');
    await request(app).get('/api/v1/glossary/nhap').expect(404);
  });
});

describe('từ điển keyword — quản trị', () => {
  beforeEach(resetDb);

  it('chỉ admin được dùng; reader và author nhận 403', async () => {
    await request(app).get('/api/v1/admin/terms').expect(401);
    for (const role of ['reader', 'author'] as const) {
      const u = await createUser(role);
      await request(app).get('/api/v1/admin/terms').set('Cookie', sessionCookie(u)).expect(403);
    }
  });

  it('tạo, sửa, xoá; chuẩn hoá alias; chặn slug trùng trong cùng ngôn ngữ; đánh dấu editedInCms', async () => {
    const cookie = sessionCookie(await createUser('admin'));
    const created = await request(app).post('/api/v1/admin/terms').set('Cookie', cookie).send(termInput()).expect(400);
    expect(created.body.error.code).toBe('VALIDATION');

    const ok = await request(app).post('/api/v1/admin/terms').set('Cookie', cookie).send(termInput({ aliases: ['smoke testing', 'smoke testing'] })).expect(201);
    assertCleanJson(ok.body);
    expect(ok.body).toMatchObject({ slug: 'smoke-test', language: 'vi', aliases: ['smoke testing'], status: 'published', editedInCms: true });

    const dup = await request(app).post('/api/v1/admin/terms').set('Cookie', cookie).send(termInput({ aliases: [] })).expect(409);
    expect(dup.body.error.code).toBe('SLUG_TAKEN');
    await request(app).post('/api/v1/admin/terms').set('Cookie', cookie).send(termInput({ aliases: [], language: 'en' })).expect(201);

    const tooLong = await request(app).post('/api/v1/admin/terms').set('Cookie', cookie).send(termInput({ slug: 'x', aliases: [], shortDefinition: 'a'.repeat(301) })).expect(400);
    expect(tooLong.body.error.code).toBe('VALIDATION');

    const updated = await request(app)
      .put(`/api/v1/admin/terms/${ok.body.id}`)
      .set('Cookie', cookie)
      .send(termInput({ aliases: [], english: 'smoke test', body: 'Chi tiết', caseSensitive: true }))
      .expect(200);
    expect(updated.body).toMatchObject({ english: 'smoke test', body: 'Chi tiết', caseSensitive: true, aliases: [] });
    const cleared = await request(app).put(`/api/v1/admin/terms/${ok.body.id}`).set('Cookie', cookie).send(termInput({ aliases: [], english: '', body: '' })).expect(200);
    expect(cleared.body.english).toBeUndefined();
    expect(cleared.body.body).toBeUndefined();

    await request(app).put('/api/v1/admin/terms/khong-phai-id').set('Cookie', cookie).send(termInput({ aliases: [] })).expect(404);
    await request(app).delete(`/api/v1/admin/terms/${ok.body.id}`).set('Cookie', cookie).send({}).expect(204);
    await request(app).delete(`/api/v1/admin/terms/${ok.body.id}`).set('Cookie', cookie).send({}).expect(404);
    const list = await request(app).get('/api/v1/admin/terms').set('Cookie', cookie).expect(200);
    expect(list.body.map((t: { language: string }) => t.language)).toEqual(['en']);
  });
});

describe('từ điển keyword — đếm số bài chứa keyword', () => {
  beforeEach(resetDb);

  it('đếm theo tên và alias trong văn xuôi, bỏ qua code; trả mẫu theo số lần xuất hiện; chỉ admin', async () => {
    const adminUser = await createUser('admin');
    const cookie = sessionCookie(adminUser);
    const cat = await createCategory();
    await createArticle(cat, adminUser, {
      slug: 'nhieu',
      content: ['Chạy smoke test. Smoke testing nữa. Lại smoke test.', '', '```', 'smoke test', '```'].join('\n'),
    });
    await createArticle(cat, adminUser, { slug: 'mot', status: 'draft', publishedAt: undefined, content: 'Một smoke test thôi, kèm `smoke test` trong code.' });
    await createArticle(cat, adminUser, { slug: 'khong', content: 'Không có gì: smoketest dính liền.' });
    await createArticle(cat, adminUser, { slug: 'tieng-anh', language: 'en', content: 'English smoke test.' });

    const r = await request(app)
      .post('/api/v1/admin/terms/usage')
      .set('Cookie', cookie)
      .send({ term: 'smoke test', aliases: ['smoke testing'] })
      .expect(200);
    expect(r.body).toMatchObject({ articles: 2, occurrences: 4 });
    expect(r.body.samples.map((x: { slug: string; occurrences: number; status: string }) => `${x.slug}:${x.occurrences}:${x.status}`)).toEqual([
      'nhieu:3:published',
      'mot:1:draft',
    ]);
    expect(r.body.samples[0].excerpt).toContain('smoke test');

    // Phân biệt hoa thường: "Smoke test" không khớp bên trong "Smoke testing" (ranh giới từ); "Smoke testing" khớp đúng 1 chỗ.
    const cs = await request(app).post('/api/v1/admin/terms/usage').set('Cookie', cookie).send({ term: 'Smoke test', caseSensitive: true }).expect(200);
    expect(cs.body).toMatchObject({ articles: 0, occurrences: 0 });
    const csLong = await request(app).post('/api/v1/admin/terms/usage').set('Cookie', cookie).send({ term: 'Smoke testing', caseSensitive: true }).expect(200);
    expect(csLong.body).toMatchObject({ articles: 1, occurrences: 1 });

    const author = sessionCookie(await createUser('author'));
    await request(app).post('/api/v1/admin/terms/usage').set('Cookie', author).send({ term: 'smoke test' }).expect(403);
    await request(app).post('/api/v1/admin/terms/usage').set('Cookie', cookie).send({ term: '' }).expect(400);
  });
});

describe('từ điển keyword — seed từ terminology.json', () => {
  beforeEach(resetDb);

  it('nạp mục glossary có definition, alias tên tiếng Anh, giữ thuật ngữ đã sửa trong CMS', async () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'tkh-seed-terms-'));
    try {
      cpSync(fixtures, dir, { recursive: true });
      writeFileSync(
        path.join(dir, 'terminology.json'),
        JSON.stringify({
          version: 1,
          terms: [
            { id: 'smoke-test', use: 'smoke test', keepEnglish: true, glossary: true, definition: 'Bộ kiểm thử nhỏ.' },
            { id: 'deduplication', use: 'khử trùng lặp', english: 'deduplication', glossary: true, definition: 'Loại bỏ bản ghi lặp.' },
            { id: 'no-definition', use: 'x', glossary: true },
            { id: 'not-glossary', use: 'y', definition: 'Không vào từ điển.' },
          ],
          patterns: [],
        }),
      );
      const first = await runSeed({ dataDir: dir, admin, log: quiet });
      expect(first.terms).toEqual({ upserted: 2, keptCms: 0 });
      const dedup = await Term.findOne({ slug: 'deduplication' }).lean();
      expect(dedup).toMatchObject({ term: 'khử trùng lặp', english: 'deduplication', aliases: ['deduplication'], autoLink: true, status: 'published' });
      expect((await Term.findOne({ slug: 'smoke-test' }).lean())?.english).toBeUndefined();

      await Term.updateOne({ slug: 'smoke-test' }, { $set: { shortDefinition: 'Sửa trong CMS', editedInCms: true } });
      const second = await runSeed({ dataDir: dir, admin, log: quiet });
      expect(second.terms).toEqual({ upserted: 1, keptCms: 1 });
      expect((await Term.findOne({ slug: 'smoke-test' }).lean())?.shortDefinition).toBe('Sửa trong CMS');

      await runSeed({ dataDir: dir, admin, log: quiet, forceContent: true });
      expect((await Term.findOne({ slug: 'smoke-test' }).lean())?.shortDefinition).toBe('Bộ kiểm thử nhỏ.');
      expect(await Term.countDocuments()).toBe(2);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});
