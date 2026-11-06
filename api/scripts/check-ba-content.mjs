// Offline editorial validation; does not seed or publish content.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import matter from 'gray-matter';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const read = p => fs.readFileSync(path.join(root, p), 'utf8');
const all = fs.readdirSync(path.join(root, 'api/seed/content')).filter(f => f.endsWith('.md')).map(file => ({ file, ...matter(read('api/seed/content/' + file)) }));
const ba = all.filter(a => a.data.category === 'business-analysis');
const slugs = new Set(all.map(a => a.data.slug));
const covers = JSON.parse(read('apps/web/src/lib/article-covers.json'));
const paths = JSON.parse(read('api/seed/learning-paths.json')).filter(p => p.category === 'business-analysis');
const fail = message => { throw new Error(message); };
for (const a of ba) {
  const d = a.data;
  if (d.slug + '.md' !== a.file || !d.title || !d.summary || d.summary.length > 400 || d.language !== 'vi' || !Array.isArray(d.tags) || d.tags.length > 10) fail('Invalid metadata: ' + a.file);
  if ((a.content.match(/^## \d\./gm) || []).length !== 8) fail('Expected 8 sections: ' + a.file);
  if (!Object.hasOwn(covers, d.slug)) fail('Missing cover brief: ' + d.slug);
  for (const m of a.content.matchAll(/(?:\(|href=")\/articles\/([a-z0-9-]+)/g)) if (!slugs.has(m[1])) fail('Missing article: ' + m[1]);
}
const linked = [];
for (const p of paths) {
  if (p.articles.length !== 8 || new Set(p.articles).size !== 8) fail('Expected 8 unique articles: ' + p.slug);
  const counts = { beginner: 0, intermediate: 0, advanced: 0 };
  for (const slug of p.articles) {
    const a = ba.find(a => a.data.slug === slug);
    if (!a || !(a.data.level in counts)) fail('Invalid path member: ' + slug);
    counts[a.data.level]++;
    linked.push(slug);
  }
  if (counts.beginner !== 3 || counts.intermediate !== 3 || counts.advanced !== 2) fail('Expected 3/3/2 levels: ' + p.slug);
}
if (new Set(linked).size !== ba.length || linked.length !== ba.length) fail('BA articles must belong to exactly one path');
console.log(`Validated ${ba.length} BA articles in ${paths.length} parts: metadata, sections, internal links, cover briefs and 3/3/2 levels.`);
