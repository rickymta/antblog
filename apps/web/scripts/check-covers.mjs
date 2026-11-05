import { readFile, access } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import sharp from 'sharp';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const covers = JSON.parse(await readFile(path.join(root, 'src/lib/article-covers.json'), 'utf8'));
for (const slug of Object.keys(covers)) {
  if (!/^[a-z0-9-]+$/.test(slug)) throw new Error('Invalid slug: ' + slug);
  await access(path.join(root, 'public/images/articles', slug + '.svg'));
  const meta = await sharp(path.join(root, 'public/images/articles', slug + '.webp')).metadata();
  if (meta.width !== 1200 || meta.height !== 675) throw new Error('Unexpected dimensions: ' + slug);
}
console.log(Object.keys(covers).length + ' covers validated (1200 × 675).');
