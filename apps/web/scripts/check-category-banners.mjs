import { readFile, access, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import sharp from 'sharp';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const dir = path.join(root, 'public/images/categories');
const WIDTH = 1600;
const HEIGHT = 400;

// Kiểm đủ banner cho mọi chủ đề trong seed. Ngữ cảnh build chỉ có web (Docker) không có api/ ⇒ bỏ kiểm nguồn,
// chỉ kiểm tính hợp lệ của các tệp banner đang có (giống check-covers.mjs).
const seedFile = path.resolve(root, '../../api/seed/categories.json');
let slugs;
try {
  slugs = JSON.parse(await readFile(seedFile, 'utf8')).map(c => c.slug);
} catch (error) {
  if (error.code !== 'ENOENT') throw error;
  console.log('Category seed is not included in this checkout/build context; checking banner assets only.');
  slugs = (await readdir(dir)).filter(f => f.endsWith('.webp')).map(f => f.slice(0, -'.webp'.length));
}
if (!slugs.length) throw new Error('No category banners to validate.');

for (const slug of slugs) {
  if (!/^[a-z0-9-]+$/.test(slug)) throw new Error('Invalid slug: ' + slug);
  await access(path.join(dir, slug + '.svg')).catch(() => { throw new Error('Missing banner SVG: ' + slug); });
  const meta = await sharp(path.join(dir, slug + '.webp')).metadata().catch(() => { throw new Error('Missing or unreadable banner WebP: ' + slug); });
  if (meta.format !== 'webp' || meta.width !== WIDTH || meta.height !== HEIGHT) throw new Error('Unexpected banner dimensions: ' + slug);
}
console.log(slugs.length + ` category banners validated (${WIDTH} × ${HEIGHT}).`);
