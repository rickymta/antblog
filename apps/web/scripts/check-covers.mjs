import { readFile, access, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import sharp from 'sharp';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const covers = JSON.parse(await readFile(path.join(root, 'src/lib/article-covers.json'), 'utf8'));
// Audit source coverage too: validating only manifest entries misses new articles.
const contentDir = path.resolve(root, '../../api/seed/content');
for (const file of (await readdir(contentDir)).filter(file => file.startsWith('ba-') && file.endsWith('.md'))) {
  const source = await readFile(path.join(contentDir, file), 'utf8');
  const slug = source.match(/^slug:\s*([a-z0-9-]+)\s*$/m)?.[1];
  if (!slug || !Object.hasOwn(covers, slug)) throw new Error('Missing BA cover brief: ' + file);
}
const baseUrlIndex = process.argv.indexOf('--base-url');
const baseUrl = baseUrlIndex === -1 ? undefined : process.argv[baseUrlIndex + 1];
if (baseUrlIndex !== -1 && !baseUrl) throw new Error('--base-url requires an HTTP(S) origin.');
if (baseUrl && !/^https?:\/\//.test(baseUrl)) throw new Error('Expected an HTTP(S) origin.');
for (const slug of Object.keys(covers)) {
  if (!/^[a-z0-9-]+$/.test(slug)) throw new Error('Invalid slug: ' + slug);
  await access(path.join(root, 'public/images/articles', slug + '.svg'));
  const meta = await sharp(path.join(root, 'public/images/articles', slug + '.webp')).metadata();
  if (meta.width !== 1200 || meta.height !== 675) throw new Error('Unexpected dimensions: ' + slug);
  if (baseUrl) {
    const response = await fetch(new URL('/images/articles/' + slug + '.webp', baseUrl), { signal: AbortSignal.timeout(10000) });
    if (!response.ok || !response.headers.get('content-type')?.startsWith('image/webp')) {
      throw new Error(`Cover unavailable: ${slug} (HTTP ${response.status}, ${response.headers.get('content-type')})`);
    }
    const served = await sharp(Buffer.from(await response.arrayBuffer())).metadata();
    if (served.width !== 1200 || served.height !== 675) throw new Error('Served cover dimensions differ: ' + slug);
  }
}
console.log(Object.keys(covers).length + ' covers validated (1200 × 675)' + (baseUrl ? ' and served correctly by ' + baseUrl : '') + '.');
