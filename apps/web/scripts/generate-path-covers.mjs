/** Compose original lesson vectors into dedicated learning-path artwork. No network or DB. */
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import sharp from 'sharp';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const paths = JSON.parse(await readFile(path.resolve(root, '../../api/seed/learning-paths.json'), 'utf8'));
const categories = JSON.parse(await readFile(path.resolve(root, '../../api/seed/categories.json'), 'utf8'));
const briefs = JSON.parse(await readFile(path.join(root, 'src/lib/article-covers.json'), 'utf8'));
const output = path.join(root, 'public/images/learning-paths');
await mkdir(output, { recursive: true });
const escape = s => String(s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&apos;' }[c]));
const manifest = {};
// Same order as the reader site (comparePathOrder in @tkh/shared): priority → level → slug.
const LEVELS = ['beginner', 'intermediate', 'advanced'];
const rankPriority = p => (Number.isInteger(p.priority) ? p.priority : Number.POSITIVE_INFINITY);
const comparePathOrder = (a, b) => {
  const pa = rankPriority(a);
  const pb = rankPriority(b);
  return (pa === pb ? 0 : pa < pb ? -1 : 1)
    || LEVELS.indexOf(a.level) - LEVELS.indexOf(b.level)
    || a.slug.localeCompare(b.slug, 'en', { numeric: true });
};

for (const p of paths) {
  if (!/^[a-z0-9-]+$/.test(p.slug)) throw new Error('Invalid path slug: ' + p.slug);
  const category = categories.find(c => c.slug === p.category);
  const siblings = paths.filter(x => x.category === p.category).sort(comparePathOrder);
  const number = siblings.indexOf(p) + 1;
  const color = category?.color ?? '#2563EB';
  const source = p.articles.find(slug => briefs[slug]);
  let artwork;
  if (source) {
    // Crop the diagram only: omit the article's masthead, sequence number and title band.
    const svg = await readFile(path.join(root, `public/images/articles/${source}.svg`), 'utf8');
    artwork = `<svg x="0" y="115" width="1200" height="420" viewBox="0 85 1200 420">${svg.replace(/^<svg[^>]*>/, '').replace(/<\/svg>\s*$/, '')}</svg>`;
  } else {
    // Data analysis: a question becomes measured evidence and a decision.
    artwork = `<g fill="none" stroke="${color}" stroke-width="4">
      <rect x="90" y="165" width="250" height="300" rx="22" fill="white"/>
      <path d="M130 230h165m-165 40h120m-120 40h145" stroke-opacity=".35"/>
      <circle cx="215" cy="385" r="35"/><path d="m200 385 12 12 22-27"/>
      <path d="M360 320h90m-15-12 15 12-15 12"/>
      <rect x="470" y="165" width="630" height="300" rx="22" fill="white"/>
      <path d="M520 215v200h520" stroke-opacity=".3"/>
      <rect x="560" y="325" width="70" height="90" rx="6" fill="${color}" fill-opacity=".3"/>
      <rect x="670" y="280" width="70" height="135" rx="6" fill="${color}" fill-opacity=".5"/>
      <rect x="780" y="300" width="70" height="115" rx="6" fill="${color}" fill-opacity=".7"/>
      <rect x="890" y="225" width="70" height="190" rx="6" fill="${color}"/>
      <path d="m570 280 130-45 125 20 130-65" stroke="#0f172a" stroke-dasharray="7 9"/>
    </g>`;
  }
  const dots = siblings.map((_, i) => {
    const x = 75 + i * Math.min(90, 900 / Math.max(1, siblings.length - 1));
    return `<circle cx="${x}" cy="610" r="${i + 1 === number ? 15 : 8}" fill="${i + 1 === number ? '#5eead4' : '#64748b'}"/>`;
  }).join('');
  const label = category?.name.en ?? p.category;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="675" viewBox="0 0 1200 675">
    <title>${escape(p.title)}</title><desc>${escape(p.description)}</desc>
    <rect width="1200" height="675" fill="#edf2f7"/>
    ${artwork}
    <rect width="1200" height="115" fill="#0f172a"/>
    <rect x="40" y="37" width="5" height="40" rx="2" fill="${color}"/>
    <text x="65" y="65" font-family="Arial,sans-serif" font-size="26" font-weight="700" fill="white">${escape(label.toUpperCase())}</text>
    <text x="1110" y="79" text-anchor="middle" font-family="Arial,sans-serif" font-size="62" font-weight="700" fill="#99f6e4">${String(number).padStart(2, '0')}</text>
    <rect y="535" width="1200" height="140" fill="#0f172a"/>
    <path d="M75 610h900" stroke="#334155" stroke-width="3" stroke-dasharray="6 10"/>
    ${dots}
    <path d="m1080 590 20 20-20 20m-24-20h44" fill="none" stroke="#5eead4" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>
  </svg>`;
  await writeFile(path.join(output, `${p.slug}.svg`), svg);
  await sharp(Buffer.from(svg)).webp({ quality: 86 }).toFile(path.join(output, `${p.slug}.webp`));
  manifest[p.slug] = { source: source ?? null, topic: p.category, number };
}
await writeFile(path.join(root, 'src/lib/path-covers.json'), JSON.stringify(manifest, null, 2) + '\n');
console.log(`Generated ${paths.length} dedicated learning-path covers (1200 × 675, SVG + WebP).`);
