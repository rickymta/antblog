import { existsSync, readdirSync, rmSync } from 'node:fs';
import path from 'node:path';
import request from 'supertest';
import { beforeEach, describe, expect, it } from 'vitest';
import { getConfig } from '../src/config.js';
import { Media } from '../src/models/index.js';
import { assertCleanJson, createUser, makeApp, resetDb, sessionCookie, useTestDb } from './helpers.js';

useTestDb();
const app = makeApp();

/** PNG 1×1 hợp lệ. */
const PNG = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==', 'base64');
/** GIF 1×1 hợp lệ. */
const GIF = Buffer.from('R0lGODlhAQABAIAAAP///wAAACH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==', 'base64');
const SVG = Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>');

function uploadedFiles(): string[] {
  const root = getConfig().uploadDir;
  const out: string[] = [];
  const walk = (dir: string) => {
    if (!existsSync(dir)) return;
    for (const e of readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, e.name);
      if (e.isDirectory()) walk(p);
      else out.push(path.relative(root, p).split(path.sep).join('/'));
    }
  };
  walk(root);
  return out;
}

describe('thư viện media', () => {
  beforeEach(async () => {
    await resetDb();
    // Mỗi ca bắt đầu với thư mục ảnh rỗng (thư mục tạm của lượt test — xem test/setup.ts).
    for (const e of readdirSync(getConfig().uploadDir)) rmSync(path.join(getConfig().uploadDir, e), { recursive: true, force: true });
  });

  it('tải PNG lên: 201 MediaItem, lưu YYYY/MM/<ngẫu nhiên>.png, phục vụ /uploads với nosniff + cache dài', async () => {
    const admin = await createUser('admin');
    const res = await request(app)
      .post('/api/v1/admin/media')
      .set('Cookie', sessionCookie(admin))
      .field('alt', 'Sơ đồ chỉ mục')
      .attach('file', PNG, { filename: 'sơ-đồ.png', contentType: 'image/png' })
      .expect(201);
    assertCleanJson(res.body);
    expect(res.body).toMatchObject({
      fileName: 'sơ-đồ.png',
      mimeType: 'image/png',
      size: PNG.length,
      width: 1,
      height: 1,
      alt: 'Sơ đồ chỉ mục',
      uploadedBy: { id: admin._id.toString(), displayName: admin.displayName, role: 'admin' },
    });
    expect(res.body.url).toMatch(/^\/uploads\/\d{4}\/\d{2}\/[a-f0-9]{24}\.png$/);
    expect(uploadedFiles()).toEqual([res.body.url.replace('/uploads/', '')]);

    const file = await request(app).get(res.body.url).expect(200);
    expect(file.headers['content-type']).toBe('image/png');
    expect(file.headers['x-content-type-options']).toBe('nosniff');
    expect(file.headers['cache-control']).toContain('immutable');
    expect(file.headers['cache-control']).toContain('max-age=31536000');
    expect(file.headers['content-security-policy']).toContain("default-src 'none'");
    expect(Buffer.compare(file.body as Buffer, PNG)).toBe(0);
    // Không liệt kê thư mục, không thoát ra ngoài UPLOAD_DIR.
    await request(app).get(res.body.url.replace(/\/[^/]+$/, '/')).expect(404);
    await request(app).get('/uploads/').expect(404);
    await request(app).get('/uploads/..%2f..%2fpackage.json').expect((r) => expect([400, 403, 404]).toContain(r.status));

    // GIF cũng nhận; Blob không tên/không loại ⇒ đoán theo magic bytes.
    const gif = await request(app)
      .post('/api/v1/admin/media')
      .set('Cookie', sessionCookie(admin))
      .attach('file', GIF, { filename: 'blob', contentType: 'application/octet-stream' })
      .expect(201);
    expect(gif.body).toMatchObject({ mimeType: 'image/gif', fileName: 'anh.gif' });
    expect(gif.body.url).toMatch(/\.gif$/);
  });

  it('từ chối SVG, đuôi giả, quá 5 MB, thiếu tệp; request JSON thường vẫn bị 415 ở chỗ khác', async () => {
    const admin = await createUser('admin');
    const c = sessionCookie(admin);
    const svg = await request(app).post('/api/v1/admin/media').set('Cookie', c).attach('file', SVG, { filename: 'a.svg', contentType: 'image/svg+xml' }).expect(415);
    expect(svg.body.error.code).toBe('UNSUPPORTED_MEDIA_TYPE');
    // Đuôi .png + Content-Type image/png nhưng nội dung là SVG/HTML.
    const fake = await request(app).post('/api/v1/admin/media').set('Cookie', c).attach('file', SVG, { filename: 'anh.png', contentType: 'image/png' }).expect(415);
    expect(fake.body.error.code).toBe('UNSUPPORTED_MEDIA_TYPE');
    // Nội dung PNG thật nhưng khai báo loại lạ.
    await request(app).post('/api/v1/admin/media').set('Cookie', c).attach('file', PNG, { filename: 'a.html', contentType: 'text/html' }).expect(415);
    const big = Buffer.concat([PNG, Buffer.alloc(5 * 1024 * 1024)]);
    const tooBig = await request(app).post('/api/v1/admin/media').set('Cookie', c).attach('file', big, { filename: 'to.png', contentType: 'image/png' }).expect(413);
    expect(tooBig.body.error.code).toBe('PAYLOAD_TOO_LARGE');
    const none = await request(app).post('/api/v1/admin/media').set('Cookie', c).field('alt', 'x').expect(400);
    expect(none.body.error.details.fields.file).toBeTruthy();
    const longAlt = await request(app).post('/api/v1/admin/media').set('Cookie', c).field('alt', 'x'.repeat(201)).attach('file', PNG, 'a.png').expect(400);
    expect(longAlt.body.error.details.fields.alt).toBeTruthy();
    // Trang khác gửi form multipart ⇒ chặn.
    await request(app).post('/api/v1/admin/media').set('Cookie', c).set('Sec-Fetch-Site', 'cross-site').attach('file', PNG, 'a.png').expect(403);
    // Multipart chỉ được phép đúng ở POST /admin/media.
    await request(app).post('/api/v1/admin/articles').set('Cookie', c).attach('file', PNG, 'a.png').expect(415);
    await request(app).patch('/api/v1/admin/media/000000000000000000000000').set('Cookie', c).attach('file', PNG, 'a.png').expect(415);
    expect(await Media.countDocuments()).toBe(0);
    expect(uploadedFiles()).toEqual([]);
  });

  it('phân quyền: reader 403; author chỉ thấy/sửa/xoá ảnh của mình; admin thấy tất cả; xoá gỡ cả tệp', async () => {
    const admin = await createUser('admin');
    const author = await createUser('author');
    const other = await createUser('author');
    const reader = await createUser('reader');
    await request(app).post('/api/v1/admin/media').set('Cookie', sessionCookie(reader)).attach('file', PNG, 'a.png').expect(403);
    await request(app).post('/api/v1/admin/media').attach('file', PNG, 'a.png').expect(401);

    const mine = await request(app).post('/api/v1/admin/media').set('Cookie', sessionCookie(author)).attach('file', PNG, { filename: 'cua-toi.png', contentType: 'image/png' }).expect(201);
    const theirs = await request(app).post('/api/v1/admin/media').set('Cookie', sessionCookie(other)).attach('file', PNG, { filename: 'cua-ban.png', contentType: 'image/png' }).expect(201);
    const a = sessionCookie(author);

    const list = await request(app).get('/api/v1/admin/media').set('Cookie', a).expect(200);
    expect(list.body).toMatchObject({ total: 1, page: 1, pageSize: 12 });
    expect(list.body.items.map((m: { id: string }) => m.id)).toEqual([mine.body.id]);
    const all = await request(app).get('/api/v1/admin/media?pageSize=1&page=2').set('Cookie', sessionCookie(admin)).expect(200);
    expect(all.body).toMatchObject({ total: 2, page: 2, pageSize: 1 });
    expect(all.body.items.map((m: { id: string }) => m.id)).toEqual([mine.body.id]); // mới nhất trước ⇒ trang 2 là ảnh cũ hơn

    await request(app).patch(`/api/v1/admin/media/${theirs.body.id}`).set('Cookie', a).send({ alt: 'x' }).expect(403);
    await request(app).delete(`/api/v1/admin/media/${theirs.body.id}`).set('Cookie', a).send({}).expect(403);
    const patched = await request(app).patch(`/api/v1/admin/media/${mine.body.id}`).set('Cookie', a).send({ alt: '  Ảnh minh hoạ ' }).expect(200);
    expect(patched.body.alt).toBe('Ảnh minh hoạ');
    const search = await request(app).get('/api/v1/admin/media?q=minh%20ho%E1%BA%A1').set('Cookie', sessionCookie(admin)).expect(200);
    expect(search.body.items.map((m: { id: string }) => m.id)).toEqual([mine.body.id]);
    const byName = await request(app).get('/api/v1/admin/media?q=cua-ban').set('Cookie', sessionCookie(admin)).expect(200);
    expect(byName.body.items.map((m: { id: string }) => m.id)).toEqual([theirs.body.id]);
    const cleared = await request(app).patch(`/api/v1/admin/media/${mine.body.id}`).set('Cookie', a).send({ alt: '' }).expect(200);
    expect(cleared.body).not.toHaveProperty('alt');

    const filePath = mine.body.url.replace('/uploads/', '');
    expect(uploadedFiles()).toContain(filePath);
    await request(app).delete(`/api/v1/admin/media/${mine.body.id}`).set('Cookie', a).send({}).expect(204);
    expect(uploadedFiles()).not.toContain(filePath);
    expect(await Media.findById(mine.body.id)).toBeNull();
    await request(app).get(mine.body.url).expect(404);
    await request(app).delete(`/api/v1/admin/media/${mine.body.id}`).set('Cookie', a).send({}).expect(404);
    // Admin xoá ảnh của người khác được.
    await request(app).delete(`/api/v1/admin/media/${theirs.body.id}`).set('Cookie', sessionCookie(admin)).send({}).expect(204);
    expect(uploadedFiles()).toEqual([]);
  });
});
