import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { Article, Comment } from '../src/models/index.js';
import { assertCleanJson, createArticle, createCategory, createUser, makeApp, resetDb, sessionCookie, useTestDb } from './helpers.js';

useTestDb();
const app = makeApp();

async function setup() {
  const admin = await createUser('admin');
  const alice = await createUser('reader', { displayName: 'Alice' });
  const bob = await createUser('reader', { displayName: 'Bob' });
  const cat = await createCategory();
  const art = await createArticle(cat, admin);
  return { admin, alice, bob, art, url: `/api/v1/articles/${art.slug}/comments` };
}

const statsComments = async (id: unknown) => (await Article.findById(id).lean())!.stats.comments;

describe('bình luận 2 cấp', () => {
  beforeEach(resetDb);

  it('trả lời một phản hồi ⇒ gắn vào gốc, replyTo = tác giả phản hồi; danh sách theo gốc', async () => {
    const { alice, bob, art, url } = await setup();
    await request(app).post(url).send({ body: 'x' }).expect(401);
    const root = await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: '  Gốc  ' }).expect(201);
    expect(root.body).toMatchObject({ articleId: art._id.toString(), body: 'Gốc', status: 'visible', likeCount: 0, likedByMe: false });
    expect(root.body).not.toHaveProperty('parentId');
    expect(root.body).not.toHaveProperty('replyTo');
    expect(root.body.author).toEqual({ id: alice._id.toString(), displayName: 'Alice', role: 'reader' });
    assertCleanJson(root.body);

    const reply = await request(app).post(url).set('Cookie', sessionCookie(bob)).send({ body: 'Phản hồi', parentId: root.body.id }).expect(201);
    expect(reply.body.parentId).toBe(root.body.id);
    expect(reply.body).not.toHaveProperty('replyTo');

    const nested = await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'Trả lời Bob', parentId: reply.body.id }).expect(201);
    expect(nested.body.parentId).toBe(root.body.id);
    expect(nested.body.replyTo).toEqual({ id: bob._id.toString(), displayName: 'Bob', role: 'reader' });

    await request(app).post(url).set('Cookie', sessionCookie(bob)).send({ body: 'Gốc thứ hai' }).expect(201);
    expect(await statsComments(art._id)).toBe(4);

    const list = await request(app).get(`${url}?sort=oldest`).expect(200);
    expect(list.body).toMatchObject({ total: 2, page: 1, pageSize: 10, commentCount: 4 });
    expect(list.body.items[0].id).toBe(root.body.id);
    expect(list.body.items[0].replyCount).toBe(2);
    expect(list.body.items[0].replies.map((r: { id: string }) => r.id)).toEqual([reply.body.id, nested.body.id]);
    expect(list.body.items[1].replies).toEqual([]);
    assertCleanJson(list.body);
    const newest = await request(app).get(`${url}?pageSize=1`).expect(200);
    expect(newest.body.total).toBe(2);
    expect(newest.body.items[0].body).toBe('Gốc thứ hai');

    const empty = await request(app).post(url).set('Cookie', sessionCookie(bob)).send({ body: '   ' }).expect(400);
    expect(empty.body.error.details.fields.body).toBeTruthy();
    await request(app).post(url).set('Cookie', sessionCookie(bob)).send({ body: 'x'.repeat(2001) }).expect(400);
  });

  it('parentId không thuộc bài ⇒ 400; gốc đã xoá/ẩn ⇒ 409 PARENT_UNAVAILABLE', async () => {
    const { admin, alice, url } = await setup();
    const cat = await createCategory();
    const other = await createArticle(cat, admin);
    const foreign = await request(app).post(`/api/v1/articles/${other.slug}/comments`).set('Cookie', sessionCookie(alice)).send({ body: 'khác' }).expect(201);
    const bad = await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'x', parentId: foreign.body.id }).expect(400);
    expect(bad.body.error.details.fields.parentId).toBeTruthy();
    await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'x', parentId: 'khong-hop-le' }).expect(400);

    const root = await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'gốc' }).expect(201);
    const reply = await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'rep', parentId: root.body.id }).expect(201);
    await request(app).delete(`/api/v1/comments/${root.body.id}`).set('Cookie', sessionCookie(alice)).send({}).expect(204);
    const c1 = await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'x', parentId: root.body.id }).expect(409);
    expect(c1.body.error.code).toBe('PARENT_UNAVAILABLE');
    await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'x', parentId: reply.body.id }).expect(409);
  });

  it('xoá mềm: chủ hoặc admin; gốc đã xoá còn phản hồi ⇒ trả status deleted, body rỗng, giữ author', async () => {
    const { admin, alice, bob, art, url } = await setup();
    const root = await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'gốc' }).expect(201);
    const reply = await request(app).post(url).set('Cookie', sessionCookie(bob)).send({ body: 'rep', parentId: root.body.id }).expect(201);
    const lonely = await request(app).post(url).set('Cookie', sessionCookie(bob)).send({ body: 'một mình' }).expect(201);
    expect(await statsComments(art._id)).toBe(3);

    await request(app).delete(`/api/v1/comments/${root.body.id}`).set('Cookie', sessionCookie(bob)).send({}).expect(403);
    await request(app).delete(`/api/v1/comments/${root.body.id}`).set('Cookie', sessionCookie(alice)).send({}).expect(204);
    await request(app).delete(`/api/v1/comments/${root.body.id}`).set('Cookie', sessionCookie(alice)).send({}).expect(204);
    await request(app).delete(`/api/v1/comments/${lonely.body.id}`).set('Cookie', sessionCookie(admin)).send({}).expect(204);
    expect(await statsComments(art._id)).toBe(1);
    expect((await Comment.findById(root.body.id).lean())!.body).toBe('');

    const list = await request(app).get(url).expect(200);
    expect(list.body.total).toBe(1);
    expect(list.body.commentCount).toBe(1);
    const thread = list.body.items[0];
    expect(thread).toMatchObject({ id: root.body.id, status: 'deleted', body: '', replyCount: 1 });
    expect(thread.author.id).toBe(alice._id.toString());
    expect(thread.replies[0].id).toBe(reply.body.id);

    // Phản hồi đã xoá hiện như chỗ giữ chỗ dưới gốc còn hiện.
    const root2 = await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'gốc 2' }).expect(201);
    const rep2 = await request(app).post(url).set('Cookie', sessionCookie(bob)).send({ body: 'rep 2', parentId: root2.body.id }).expect(201);
    await request(app).delete(`/api/v1/comments/${rep2.body.id}`).set('Cookie', sessionCookie(bob)).send({}).expect(204);
    const list2 = await request(app).get(url).expect(200);
    const t2 = list2.body.items.find((t: { id: string }) => t.id === root2.body.id);
    expect(t2.replyCount).toBe(0);
    expect(t2.replies).toHaveLength(1);
    expect(t2.replies[0]).toMatchObject({ status: 'deleted', body: '' });
  });

  it('bình luận bị ẩn không bao giờ lộ; gốc ẩn còn phản hồi ⇒ như deleted; stats.comments cập nhật', async () => {
    const { admin, alice, bob, art, url } = await setup();
    const root = await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'nội dung nhạy cảm' }).expect(201);
    const rep = await request(app).post(url).set('Cookie', sessionCookie(bob)).send({ body: 'rep', parentId: root.body.id }).expect(201);
    const lone = await request(app).post(url).set('Cookie', sessionCookie(bob)).send({ body: 'spam' }).expect(201);
    const repHidden = await request(app).post(url).set('Cookie', sessionCookie(bob)).send({ body: 'rep spam', parentId: root.body.id }).expect(201);
    const adm = sessionCookie(admin);
    for (const id of [root.body.id, lone.body.id, repHidden.body.id]) {
      const r = await request(app).patch(`/api/v1/admin/comments/${id}`).set('Cookie', adm).send({ status: 'hidden' }).expect(200);
      expect(r.body.status).toBe('hidden');
    }
    // Ẩn lại lần nữa không trừ đôi.
    await request(app).patch(`/api/v1/admin/comments/${lone.body.id}`).set('Cookie', adm).send({ status: 'hidden' }).expect(200);
    expect(await statsComments(art._id)).toBe(1);

    const list = await request(app).get(url).expect(200);
    expect(JSON.stringify(list.body)).not.toContain('nhạy cảm');
    expect(JSON.stringify(list.body)).not.toContain('spam');
    expect(list.body.total).toBe(1);
    expect(list.body.items[0]).toMatchObject({ id: root.body.id, status: 'deleted', body: '', replyCount: 1 });
    expect(list.body.items[0].replies.map((r: { id: string }) => r.id)).toEqual([rep.body.id]);

    await request(app).patch(`/api/v1/admin/comments/${root.body.id}`).set('Cookie', adm).send({ status: 'visible' }).expect(200);
    expect(await statsComments(art._id)).toBe(2);
    await request(app).delete(`/api/v1/comments/${rep.body.id}`).set('Cookie', sessionCookie(bob)).send({}).expect(204);
    const del = await request(app).patch(`/api/v1/admin/comments/${rep.body.id}`).set('Cookie', adm).send({ status: 'visible' }).expect(409);
    expect(del.body.error.code).toBe('COMMENT_DELETED');
    expect(await statsComments(art._id)).toBe(1);
  });

  it('sửa: chỉ chủ bình luận, đặt editedAt', async () => {
    const { alice, bob, url } = await setup();
    const c = await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'cũ' }).expect(201);
    await request(app).patch(`/api/v1/comments/${c.body.id}`).set('Cookie', sessionCookie(bob)).send({ body: 'hack' }).expect(403);
    const ok = await request(app).patch(`/api/v1/comments/${c.body.id}`).set('Cookie', sessionCookie(alice)).send({ body: 'mới' }).expect(200);
    expect(ok.body.body).toBe('mới');
    expect(ok.body.editedAt).toMatch(/Z$/);
    await request(app).patch('/api/v1/comments/000000000000000000000000').set('Cookie', sessionCookie(alice)).send({ body: 'x' }).expect(404);
  });

  it('thích bình luận: idempotent, chống bấm đúp, likedByMe', async () => {
    const { alice, bob, url } = await setup();
    const c = await request(app).post(url).set('Cookie', sessionCookie(alice)).send({ body: 'hay' }).expect(201);
    const likeUrl = `/api/v1/comments/${c.body.id}/like`;
    await request(app).put(likeUrl).send({}).expect(401);
    const burst = await Promise.all(Array.from({ length: 5 }, () => request(app).put(likeUrl).set('Cookie', sessionCookie(bob)).send({})));
    expect(burst.every((r) => r.status === 200 && r.body.likedByMe === true)).toBe(true);
    expect((await request(app).put(likeUrl).set('Cookie', sessionCookie(bob)).send({}).expect(200)).body).toEqual({ likeCount: 1, likedByMe: true });
    expect((await request(app).put(likeUrl).set('Cookie', sessionCookie(alice)).send({}).expect(200)).body).toEqual({ likeCount: 2, likedByMe: true });

    const list = await request(app).get(url).set('Cookie', sessionCookie(bob)).expect(200);
    expect(list.body.items[0]).toMatchObject({ likeCount: 2, likedByMe: true });
    const anon = await request(app).get(url).expect(200);
    expect(anon.body.items[0].likedByMe).toBe(false);

    expect((await request(app).delete(likeUrl).set('Cookie', sessionCookie(bob)).send({}).expect(200)).body).toEqual({ likeCount: 1, likedByMe: false });
    expect((await request(app).delete(likeUrl).set('Cookie', sessionCookie(bob)).send({}).expect(200)).body).toEqual({ likeCount: 1, likedByMe: false });

    const top = await request(app).post(url).set('Cookie', sessionCookie(bob)).send({ body: 'mới hơn' }).expect(201);
    const sorted = await request(app).get(`${url}?sort=top`).expect(200);
    expect(sorted.body.items[0].id).toBe(c.body.id);
    expect(sorted.body.items[1].id).toBe(top.body.id);
  });

  it('giới hạn 10 bình luận / phút / người ⇒ 429', async () => {
    const limited = makeApp({ commentRateLimit: 2 });
    const { alice, bob, url } = await setup();
    await request(limited).post(url).set('Cookie', sessionCookie(alice)).send({ body: '1' }).expect(201);
    await request(limited).post(url).set('Cookie', sessionCookie(alice)).send({ body: '2' }).expect(201);
    const res = await request(limited).post(url).set('Cookie', sessionCookie(alice)).send({ body: '3' }).expect(429);
    expect(res.body.error.code).toBe('RATE_LIMITED');
    await request(limited).post(url).set('Cookie', sessionCookie(bob)).send({ body: 'người khác' }).expect(201);
  });
});
