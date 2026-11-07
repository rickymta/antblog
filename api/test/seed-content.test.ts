import { readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import matter from 'gray-matter';
import { describe, expect, it } from 'vitest';

// Kiểm tra dữ liệu seed THẬT (không cần DB): cây chủ đề hợp lệ và mọi bài/lộ trình trỏ đúng chủ đề.
// Chốt chặn cho các nhánh loạt bài merge sau này còn dùng `category: programming` (chủ đề cha).
const seedDir = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'seed');
const MAX_DEPTH = 3;

interface Cat {
  slug: string;
  parent?: string;
}
const categories = JSON.parse(readFileSync(path.join(seedDir, 'categories.json'), 'utf8')) as Cat[];
const bySlug = new Map(categories.map((c) => [c.slug, c]));
const parentSlugs = new Set(categories.flatMap((c) => (c.parent ? [c.parent] : [])));
const contentDir = path.join(seedDir, 'content');
const files = readdirSync(contentDir).filter((f) => f.endsWith('.md')).sort();
const paths = JSON.parse(readFileSync(path.join(seedDir, 'learning-paths.json'), 'utf8')) as Array<{ slug: string; category?: string }>;

function depth(slug: string): number {
  let d = 1;
  for (let c = bySlug.get(slug); c?.parent && d <= categories.length; c = bySlug.get(c.parent)) d++;
  return d;
}

describe('dữ liệu seed thật', () => {
  it('giữ cụm định vị đa ngôn ngữ riêng và phân loại các loạt bài ngôn ngữ', () => {
    const bad: string[] = [];
    for (const file of files) {
      const { slug, category, level } = matter(readFileSync(path.join(contentDir, file), 'utf8')).data;
      const language = ['python', 'rust', 'go'].find(lang => String(slug).startsWith(`${lang}-`));
      const expected = String(slug).startsWith('location-') ? 'location-routing'
        : String(slug).startsWith('flutter-') ? 'flutter'
        : language ? `${language}-${level}` : undefined;
      if (expected && category !== expected) bad.push(`${slug}: ${category} → ${expected}`);
    }
    expect(bad).toEqual([]);
  });
  it('slug chủ đề duy nhất, mọi chủ đề cha tồn tại, cây không quá 3 cấp và không có vòng', () => {
    expect(new Set(categories.map((c) => c.slug)).size).toBe(categories.length);
    expect(categories.filter((c) => c.parent && !bySlug.has(c.parent)).map((c) => `${c.slug} → ${c.parent}`)).toEqual([]);
    expect(categories.filter((c) => depth(c.slug) > MAX_DEPTH).map((c) => c.slug)).toEqual([]);
  });

  it('mọi bài có category tồn tại và là chủ đề lá (chủ đề cha không chứa bài)', () => {
    expect(files.length).toBeGreaterThan(0);
    const missing: string[] = [];
    const onParent: string[] = [];
    for (const f of files) {
      const { category } = matter(readFileSync(path.join(contentDir, f), 'utf8')).data as { category?: string };
      if (!category || !bySlug.has(category)) missing.push(`${f} (category: ${category ?? '∅'})`);
      else if (parentSlugs.has(category)) onParent.push(`${f} (category: ${category})`);
    }
    expect(missing, `Bài có chủ đề không tồn tại:\n${missing.join('\n')}`).toEqual([]);
    expect(onParent, `Bài gắn vào chủ đề cha — đổi sang một chủ đề lá:\n${onParent.join('\n')}`).toEqual([]);
  });

  it('mọi lộ trình học có category tồn tại (được phép trỏ tới chủ đề cha)', () => {
    const bad = paths.filter((p) => p.category && !bySlug.has(p.category)).map((p) => `${p.slug} (category: ${p.category})`);
    expect(bad, `Lộ trình trỏ tới chủ đề không tồn tại:\n${bad.join('\n')}`).toEqual([]);
  });
});
