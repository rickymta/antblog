// Thư viện media (hợp đồng §8.2): tải ảnh lên (multipart — ngoại lệ DUY NHẤT của luật "ghi phải là JSON"),
// liệt kê/tìm, sửa alt, xoá. Admin thấy mọi ảnh; author chỉ ảnh do mình tải lên.
import { randomBytes } from 'node:crypto';
import { mkdir, unlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { Router, type NextFunction, type Request, type Response } from 'express';
import { imageSize } from 'image-size';
import { Types } from 'mongoose';
import multer from 'multer';
import { z } from 'zod';
import { getConfig } from '../config.js';
import { AppError, forbidden, notFound, validation } from '../lib/errors.js';
import { escapeRegex } from '../lib/text.js';
import { isObjectId, optionalQueryString, pagingSchema, parse } from '../lib/validate.js';
import { currentUser } from '../middleware/auth.js';
import { Media, MEDIA_MIME_TYPES, type IMedia, type MediaMimeType } from '../models/index.js';
import { toMediaItems } from '../services/serializers.js';

export const MEDIA_MAX_BYTES = 5 * 1024 * 1024;
const ALT_MAX = 200;

const EXT: Record<MediaMimeType, string> = {
  'image/png': 'png',
  'image/jpeg': 'jpg',
  'image/webp': 'webp',
  'image/gif': 'gif',
};

/** Loại ảnh thật theo magic bytes — không tin Content-Type hay đuôi tệp do trình duyệt gửi. */
export function sniffImageType(buf: Buffer): MediaMimeType | undefined {
  if (buf.length >= 8 && buf.subarray(0, 8).equals(Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]))) return 'image/png';
  if (buf.length >= 3 && buf[0] === 0xff && buf[1] === 0xd8 && buf[2] === 0xff) return 'image/jpeg';
  if (buf.length >= 6) {
    const head = buf.subarray(0, 6).toString('latin1');
    if (head === 'GIF87a' || head === 'GIF89a') return 'image/gif';
  }
  if (buf.length >= 12 && buf.subarray(0, 4).toString('latin1') === 'RIFF' && buf.subarray(8, 12).toString('latin1') === 'WEBP') {
    return 'image/webp';
  }
  return undefined;
}

/** Content-Type trình duyệt khai báo phải thuộc danh sách cho phép (Blob không có loại ⇒ octet-stream). */
const DECLARED_OK = new Set<string>([...MEDIA_MIME_TYPES, 'image/jpg', 'image/pjpeg', 'application/octet-stream', '']);

/** Tên tệp gốc để hiển thị: bỏ thư mục, ký tự điều khiển, cắt 200 ký tự. */
function cleanFileName(name: string | undefined, ext: string): string {
  // multer 2 giải mã tên tệp theo latin1 — đổi lại sang UTF-8 cho tên tiếng Việt.
  let n = name ?? '';
  try {
    const utf8 = Buffer.from(n, 'latin1').toString('utf8');
    if (!utf8.includes('�')) n = utf8;
  } catch {
    /* giữ nguyên */
  }
  n = path.basename(n.replace(/\\/g, '/')).replace(/[\u0000-\u001f\u007f]/g, '').trim();
  if (!n || n === 'blob') n = `anh.${ext}`;
  return n.slice(0, 200);
}

const upload = multer({
  storage: multer.memoryStorage(),
  limits: { fileSize: MEDIA_MAX_BYTES, files: 1, fields: 5, fieldSize: 4096, parts: 8 },
});

/** Chạy multer, đổi lỗi của nó thành lỗi theo hợp đồng (413 / 400). */
function receiveFile(req: Request, res: Response, next: NextFunction): void {
  upload.single('file')(req, res, (err: unknown) => {
    if (!err) return next();
    if (err instanceof multer.MulterError) {
      if (err.code === 'LIMIT_FILE_SIZE') {
        return next(new AppError(413, 'PAYLOAD_TOO_LARGE', 'Ảnh vượt quá 5 MB.'));
      }
      if (err.code === 'LIMIT_UNEXPECTED_FILE' || err.code === 'LIMIT_FILE_COUNT') {
        return next(validation({ file: 'Chỉ gửi một tệp, trong trường "file".' }));
      }
      return next(validation({ file: 'Dữ liệu tải lên không hợp lệ.' }));
    }
    // Thân multipart hỏng (busboy) ⇒ 400.
    return next(validation({ file: 'Dữ liệu tải lên không hợp lệ.' }));
  });
}

/** Chặn tải lên từ trang khác (form multipart vượt được luật 415) — trình duyệt hiện đại gửi Sec-Fetch-Site. */
function sameSiteOnly(req: Request, _res: Response, next: NextFunction): void {
  if (req.get('sec-fetch-site') === 'cross-site') return next(forbidden('Không nhận tải lên từ trang khác.'));
  next();
}

function absolutePath(storagePath: string): string {
  const root = getConfig().uploadDir;
  const abs = path.resolve(root, storagePath);
  if (!abs.startsWith(root + path.sep)) throw new Error(`Đường dẫn tệp nằm ngoài UPLOAD_DIR: ${storagePath}`);
  return abs;
}

async function removeFile(storagePath: string): Promise<void> {
  try {
    await unlink(absolutePath(storagePath));
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code !== 'ENOENT') throw err;
  }
}

export function mediaRouter(): Router {
  const r = Router();

  async function loadMediaFor(req: Request): Promise<IMedia> {
    const id = String(req.params.id);
    if (!isObjectId(id)) throw notFound('Không tìm thấy ảnh.');
    const media = await Media.findById(id).lean<IMedia>();
    if (!media) throw notFound('Không tìm thấy ảnh.');
    const user = currentUser(req);
    if (user.role !== 'admin' && !media.uploadedBy.equals(user._id)) {
      throw forbidden('Bạn chỉ quản lý được ảnh do mình tải lên.');
    }
    return media;
  }

  async function one(id: Types.ObjectId) {
    const m = await Media.findById(id).lean<IMedia>();
    if (!m) throw notFound('Không tìm thấy ảnh.');
    const [out] = await toMediaItems([m]);
    return out;
  }

  r.get('/', async (req, res) => {
    const q = parse(z.object({ q: optionalQueryString(200), ...pagingSchema(12) }), req.query);
    const user = currentUser(req);
    const filter: Record<string, unknown> = {};
    if (user.role !== 'admin') filter.uploadedBy = user._id;
    if (q.q) {
      const rx = new RegExp(escapeRegex(q.q), 'i');
      filter.$or = [{ fileName: rx }, { alt: rx }];
    }
    const [items, total] = await Promise.all([
      Media.find(filter)
        .sort({ createdAt: -1, _id: -1 })
        .skip((q.page - 1) * q.pageSize)
        .limit(q.pageSize)
        .lean<IMedia[]>(),
      Media.countDocuments(filter),
    ]);
    res.json({ items: await toMediaItems(items), total, page: q.page, pageSize: q.pageSize });
  });

  r.post('/', sameSiteOnly, receiveFile, async (req, res) => {
    const file = req.file;
    if (!file || !file.buffer?.length) throw validation({ file: 'Chọn một tệp ảnh để tải lên.' });
    const declared = (file.mimetype ?? '').toLowerCase();
    const mimeType = sniffImageType(file.buffer);
    if (!DECLARED_OK.has(declared) || !mimeType) {
      throw new AppError(415, 'UNSUPPORTED_MEDIA_TYPE', 'Chỉ nhận ảnh PNG, JPEG, WebP hoặc GIF (không nhận SVG).');
    }
    const { alt } = parse(
      z.object({ alt: z.preprocess((v) => (typeof v === 'string' && v.trim() === '' ? undefined : v), z.string().trim().max(ALT_MAX).optional()) }),
      req.body ?? {},
    );

    let width: number | undefined;
    let height: number | undefined;
    try {
      const dim = imageSize(file.buffer);
      width = dim.width;
      height = dim.height;
    } catch {
      // Đúng magic bytes nhưng phần còn lại hỏng ⇒ không phải ảnh dùng được.
      throw new AppError(415, 'UNSUPPORTED_MEDIA_TYPE', 'Tệp ảnh bị hỏng hoặc không đọc được kích thước.');
    }

    const user = currentUser(req);
    const now = new Date();
    const ext = EXT[mimeType];
    const dir = `${now.getUTCFullYear()}/${String(now.getUTCMonth() + 1).padStart(2, '0')}`;
    const storagePath = `${dir}/${randomBytes(12).toString('hex')}.${ext}`;
    const abs = absolutePath(storagePath);
    await mkdir(path.dirname(abs), { recursive: true });
    await writeFile(abs, file.buffer, { flag: 'wx' });

    let created;
    try {
      created = await Media.create({
        fileName: cleanFileName(file.originalname, ext),
        storagePath,
        mimeType,
        size: file.size,
        width,
        height,
        alt,
        uploadedBy: user._id,
      });
    } catch (err) {
      await removeFile(storagePath).catch(() => undefined);
      throw err;
    }
    res.status(201).json(await one(created._id));
  });

  r.patch('/:id', async (req, res) => {
    const media = await loadMediaFor(req);
    const { alt } = parse(z.object({ alt: z.string().trim().max(ALT_MAX) }), req.body);
    await Media.updateOne({ _id: media._id }, alt ? { $set: { alt } } : { $unset: { alt: 1 } });
    res.json(await one(media._id));
  });

  r.delete('/:id', async (req, res) => {
    const media = await loadMediaFor(req);
    await Media.deleteOne({ _id: media._id });
    await removeFile(media.storagePath);
    res.status(204).end();
  });

  return r;
}
