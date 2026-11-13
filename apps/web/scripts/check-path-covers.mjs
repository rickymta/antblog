/** Verify every seeded learning path has distinct, correctly sized, packaged artwork. */
import { readFile, access } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import sharp from 'sharp';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const covers = JSON.parse(await readFile(path.join(root, 'src/lib/path-covers.json'), 'utf8'));
let paths;
try {
  paths = JSON.parse(await readFile(path.resolve(root, '../../api/seed/learning-paths.json'), 'utf8'));
} catch (error) {
  if (error.code !== 'ENOENT') throw error;
  paths = Object.keys(covers).map(slug => ({ slug }));
}
if (!paths.length) throw new Error('No learning paths to validate.');
const hashes = new Set();
for (const { slug } of paths) {
  if (!/^[a-z0-9-]+$/.test(slug) || !Object.hasOwn(covers, slug)) throw new Error('Missing cover mapping: ' + slug);
  const stem = path.join(root, 'public/images/learning-paths', slug);
  await access(stem + '.svg');
  const bytes = await readFile(stem + '.webp');
  const meta = await sharp(bytes).metadata();
  if (meta.format !== 'webp' || meta.width !== 1200 || meta.height !== 675) throw new Error('Invalid cover: ' + slug);
  const hash = createHash('sha256').update(bytes).digest('hex');
  if (hashes.has(hash)) throw new Error('Duplicate artwork: ' + slug);
  hashes.add(hash);
}
console.log(`Validated ${paths.length} distinct learning-path covers (1200 × 675).`);
