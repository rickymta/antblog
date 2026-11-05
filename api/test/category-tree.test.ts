import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { Types } from 'mongoose';
import { Category } from '../src/models/category.js';
import { createArticle, createCategory, createUser, makeApp, resetDb, sessionCookie, useTestDb } from './helpers.js';

useTestDb();
const app = makeApp();
const base = '/api/v1/admin/categories';
const input = (slug: string, parentId?: string | null) => ({ slug, parentId, name: {vi:slug,en:slug}, description:{vi:'',en:''}, icon:'code', color:'#2563EB', order:1 });

describe('category hierarchy (maximum three levels)', () => {
  beforeEach(resetDb);

  it('accepts three levels, rejects a fourth level, cycles, self-parent and missing parents', async () => {
    const cookie = sessionCookie(await createUser('admin'));
    const root = (await request(app).post(base).set('Cookie',cookie).send(input('root')).expect(201)).body;
    const child = (await request(app).post(base).set('Cookie',cookie).send(input('child',root.id)).expect(201)).body;
    const leaf = (await request(app).post(base).set('Cookie',cookie).send(input('leaf',child.id)).expect(201)).body;
    expect(leaf.parentId).toBe(child.id);
    for (const [slug,parent] of [['fourth',leaf.id],['orphan',String(new Types.ObjectId())]]) {
      const response = await request(app).post(base).set('Cookie',cookie).send(input(slug!,parent)).expect(400);
      expect(response.body.error.details.fields.parentId).toBeTruthy();
    }
    await request(app).put(`${base}/${root.id}`).set('Cookie',cookie).send(input('root',leaf.id)).expect(400);
    await request(app).put(`${base}/${child.id}`).set('Cookie',cookie).send(input('child',child.id)).expect(400);
    expect(await Category.countDocuments()).toBe(3);
  });

  it('validates moved descendants; omitted parent preserves position, null moves to root', async () => {
    const cookie = sessionCookie(await createUser('admin'));
    const a = await createCategory();
    const b = await createCategory({parentId:a._id});
    const c = await createCategory({parentId:b._id});
    const other = await createCategory();
    await request(app).put(`${base}/${a._id}`).set('Cookie',cookie).send(input(a.slug,String(other._id))).expect(400);
    const preserved = await request(app).put(`${base}/${b._id}`).set('Cookie',cookie).send(input(b.slug)).expect(200);
    expect(preserved.body.parentId).toBe(String(a._id));
    await request(app).put(`${base}/${b._id}`).set('Cookie',cookie).send(input(b.slug,null)).expect(200);
    expect((await Category.findById(b._id))!.parentId).toBeNull();
    expect(String((await Category.findById(c._id))!.parentId)).toBe(String(b._id));
  });

  it('blocks deletion of a parent even when it has no direct articles', async () => {
    const cookie = sessionCookie(await createUser('admin'));
    const a = await createCategory();
    await createCategory({parentId:a._id});
    const r = await request(app).delete(`${base}/${a._id}`).set('Cookie',cookie).send({}).expect(409);
    expect(r.body.error.code).toBe('CATEGORY_HAS_CHILDREN');
    expect(await Category.exists({_id:a._id})).toBeTruthy();
  });

  it('aggregates public descendants once and excludes drafts and scheduled articles', async () => {
    const author = await createUser('admin');
    const a = await createCategory();
    const b = await createCategory({parentId:a._id});
    const c = await createCategory({parentId:b._id});
    const outside = await createCategory();
    await createArticle(a,author);
    await createArticle(b,author);
    await createArticle(c,author);
    await createArticle(c,author,{status:'draft'});
    await createArticle(c,author,{publishedAt:new Date(Date.now()+86_400_000)});
    await createArticle(outside,author);
    const list = await request(app).get('/api/v1/categories').expect(200);
    expect(list.body.find((n: {id:string})=>n.id===String(a._id)).articleCount).toBe(3);
    const detail = await request(app).get(`/api/v1/categories/${b.slug}`).expect(200);
    expect(detail.body.articleCount).toBe(2);
    for (const sort of ['latest','popular','trending']) {
      const result = await request(app).get(`/api/v1/articles?category=${a.slug}&sort=${sort}`).expect(200);
      expect(result.body.total).toBe(3);
      expect(new Set(result.body.items.map((n:{id:string})=>n.id)).size).toBe(3);
    }
  });
});
