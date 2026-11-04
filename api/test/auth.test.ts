import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { User } from '../src/models/index.js';
import { assertCleanJson, createUser, makeApp, resetDb, sessionCookie, TEST_PASSWORD, useTestDb } from './helpers.js';

useTestDb();
const app = makeApp();

function cookieNamed(res: request.Response, name: string): string | undefined {
  const raw = res.headers['set-cookie'] as unknown as string[] | undefined;
  return raw?.find((c) => c.startsWith(`${name}=`));
}

describe('xác thực', () => {
  beforeEach(resetDb);

  it('GET /auth/session luôn 200: khách ⇒ user null, đã đăng nhập ⇒ user; GET không dùng no-store', async () => {
    const anon = await request(app).get('/api/v1/auth/session').expect(200);
    expect(anon.body).toEqual({ user: null });
    expect(anon.headers['cache-control']).toBe('private, no-cache');
    const user = await createUser('reader', { email: 'sess@example.com' });
    const cookie = sessionCookie(user);
    const me = await request(app).get('/api/v1/auth/session').set('Cookie', cookie).expect(200);
    expect(me.body.user).toMatchObject({ email: 'sess@example.com' });
    const write = await request(app).post('/api/v1/auth/logout').set('Cookie', cookie).send({}).expect(204);
    expect(write.headers['cache-control']).toBe('no-store');
  });

  it('đăng ký ⇒ 201, đặt cookie httpOnly SameSite=Lax, me/logout hoạt động', async () => {
    const agent = request.agent(app);
    const res = await agent
      .post('/api/v1/auth/register')
      .send({ email: '  Alice@Example.COM ', password: 'mat-khau-123', displayName: 'Alice' })
      .expect(201);
    expect(res.body.user).toMatchObject({ email: 'alice@example.com', displayName: 'Alice', role: 'reader', status: 'active' });
    expect(res.body.user.id).toMatch(/^[a-f0-9]{24}$/);
    expect(res.body.user).not.toHaveProperty('avatarUrl');
    assertCleanJson(res.body);
    const cookie = cookieNamed(res, 'tkh_session')!;
    expect(cookie).toMatch(/HttpOnly/i);
    expect(cookie).toMatch(/SameSite=Lax/i);
    expect(cookie).not.toMatch(/Secure/i);
    expect(cookie).toMatch(/Max-Age=604800/);

    const stored = await User.findOne({ email: 'alice@example.com' }).lean();
    expect(stored!.passwordHash).toMatch(/^\$2[aby]\$12\$/);

    const me = await agent.get('/api/v1/auth/me').expect(200);
    expect(me.body.user.email).toBe('alice@example.com');

    const out = await agent.post('/api/v1/auth/logout').send({}).expect(204);
    expect(cookieNamed(out, 'tkh_session')).toMatch(/tkh_session=;/);
    await agent.get('/api/v1/auth/me').expect(401);
  });

  it('email trùng ⇒ 409 EMAIL_TAKEN; dữ liệu sai ⇒ 400 VALIDATION có details.fields', async () => {
    await request(app).post('/api/v1/auth/register').send({ email: 'b@x.vn', password: 'mat-khau-123', displayName: 'Bê' }).expect(201);
    const dup = await request(app)
      .post('/api/v1/auth/register')
      .send({ email: 'B@X.vn', password: 'mat-khau-123', displayName: 'Bê' })
      .expect(409);
    expect(dup.body.error.code).toBe('EMAIL_TAKEN');
    const bad = await request(app).post('/api/v1/auth/register').send({ email: 'khong-phai-email', password: '123', displayName: 'X' }).expect(400);
    expect(bad.body.error.code).toBe('VALIDATION');
    expect(Object.keys(bad.body.error.details.fields).sort()).toEqual(['displayName', 'email', 'password']);
    expect(typeof bad.body.error.message).toBe('string');
  });

  it('đăng nhập: sai ⇒ 401 INVALID_CREDENTIALS, đúng ⇒ 200 + lastLoginAt; bị khoá ⇒ 403 ACCOUNT_LOCKED', async () => {
    const user = await createUser('reader');
    const wrong = await request(app).post('/api/v1/auth/login').send({ email: user.email, password: 'sai-mat-khau' }).expect(401);
    expect(wrong.body.error.code).toBe('INVALID_CREDENTIALS');
    await request(app).post('/api/v1/auth/login').send({ email: 'khong-co@x.vn', password: 'sai-mat-khau' }).expect(401);
    const ok = await request(app).post('/api/v1/auth/login').send({ email: user.email.toUpperCase(), password: TEST_PASSWORD }).expect(200);
    expect(ok.body.user.id).toBe(user._id.toString());
    expect((await User.findById(user._id).lean())!.lastLoginAt).toBeInstanceOf(Date);

    await User.updateOne({ _id: user._id }, { $set: { status: 'locked' } });
    const locked = await request(app).post('/api/v1/auth/login').send({ email: user.email, password: TEST_PASSWORD }).expect(403);
    expect(locked.body.error.code).toBe('ACCOUNT_LOCKED');
  });

  it('phiên của user bị khoá ⇒ coi như chưa đăng nhập và xoá cookie', async () => {
    const user = await createUser('reader');
    const cookie = sessionCookie(user);
    await request(app).get('/api/v1/auth/me').set('Cookie', cookie).expect(200);
    await User.updateOne({ _id: user._id }, { $set: { status: 'locked' } });
    const res = await request(app).get('/api/v1/auth/me').set('Cookie', cookie).expect(401);
    expect(res.body.error.code).toBe('UNAUTHENTICATED');
    expect(cookieNamed(res, 'tkh_session')).toMatch(/tkh_session=;/);
  });

  it('cookie giả mạo bị bỏ qua', async () => {
    await request(app).get('/api/v1/auth/me').set('Cookie', 'tkh_session=abc.def.ghi').expect(401);
  });

  it('PATCH /auth/me cập nhật hồ sơ; chuỗi rỗng xoá trường', async () => {
    const user = await createUser('reader');
    const cookie = sessionCookie(user);
    const res = await request(app)
      .patch('/api/v1/auth/me')
      .set('Cookie', cookie)
      .send({ displayName: 'Tên mới', avatarUrl: 'https://cdn.example.com/a.png', bio: 'Xin chào' })
      .expect(200);
    expect(res.body.user).toMatchObject({ displayName: 'Tên mới', avatarUrl: 'https://cdn.example.com/a.png', bio: 'Xin chào' });
    const bad = await request(app).patch('/api/v1/auth/me').set('Cookie', cookie).send({ avatarUrl: 'javascript:alert(1)' }).expect(400);
    expect(bad.body.error.details.fields.avatarUrl).toBeTruthy();
    const cleared = await request(app).patch('/api/v1/auth/me').set('Cookie', cookie).send({ bio: '' }).expect(200);
    expect(cleared.body.user).not.toHaveProperty('bio');
    await request(app).patch('/api/v1/auth/me').send({ bio: 'x' }).expect(401);
  });

  it('request ghi thiếu Content-Type JSON ⇒ 415 (kể cả thân rỗng); GET thì không cần', async () => {
    const form = await request(app)
      .post('/api/v1/auth/login')
      .type('form')
      .send('email=a@b.vn&password=12345678')
      .expect(415);
    expect(form.body.error.code).toBe('UNSUPPORTED_MEDIA_TYPE');
    await request(app).post('/api/v1/auth/logout').expect(415);
    await request(app).post('/api/v1/auth/logout').set('Content-Type', 'text/plain').send('{}').expect(415);
    await request(app).post('/api/v1/auth/logout').set('Content-Type', 'application/json; charset=utf-8').send('{}').expect(204);
    await request(app).get('/api/v1/site/summary').expect(200);
  });

  it('JSON hỏng ⇒ 400 VALIDATION; đường dẫn lạ ⇒ 404 NOT_FOUND', async () => {
    const res = await request(app).post('/api/v1/auth/login').set('Content-Type', 'application/json').send('{"email":').expect(400);
    expect(res.body.error.code).toBe('VALIDATION');
    const nf = await request(app).get('/api/v1/khong-ton-tai').expect(404);
    expect(nf.body.error.code).toBe('NOT_FOUND');
  });

  it('giới hạn tần suất đăng nhập + đăng ký dùng chung theo IP ⇒ 429 RATE_LIMITED', async () => {
    const limited = makeApp({ authRateLimit: 3 });
    for (let i = 0; i < 2; i++) {
      await request(limited).post('/api/v1/auth/login').send({ email: 'x@y.vn', password: 'sai-mat-khau' }).expect(401);
    }
    await request(limited).post('/api/v1/auth/register').send({ email: 'r@y.vn', password: 'mat-khau-123', displayName: 'Rờ' }).expect(201);
    const res = await request(limited).post('/api/v1/auth/login').send({ email: 'x@y.vn', password: 'sai-mat-khau' }).expect(429);
    expect(res.body.error.code).toBe('RATE_LIMITED');
  });

  it('request đầu tiên nhận cookie khách tkh_vid', async () => {
    const res = await request(app).get('/api/v1/categories').expect(200);
    const vid = cookieNamed(res, 'tkh_vid')!;
    expect(vid).toMatch(/tkh_vid=[0-9a-f-]{36};/);
    expect(vid).toMatch(/HttpOnly/i);
    const again = await request(app).get('/api/v1/categories').set('Cookie', vid.split(';')[0]!).expect(200);
    expect(cookieNamed(again, 'tkh_vid')).toBeUndefined();
  });
});
