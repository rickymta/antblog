/** Metadata-only migration: language/level categories and bounded series parts. Dry-run by default. */
import 'dotenv/config';
import mongoose from 'mongoose';
import { readFile, readdir, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import matter from 'gray-matter';
import { Article, Category, LearningPath, type IArticle, type ICategory, type ILearningPath, type Level } from '../src/models/index.js';
import { validateCategoryTree } from '../src/services/categories.js';
import { recountCategories } from '../src/services/counters.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const apply = process.argv.includes('--apply');
const uri = process.env.MONGODB_URI;
if (!uri) throw new Error('MONGODB_URI is required through the environment.');
const languages = ['csharp', 'python', 'rust', 'go'];
type CategorySeed = { slug: string; parent?: string; name: { vi: string; en: string }; description: { vi: string; en: string }; icon: string; color: string; order: number };
type Part = { slug: string; title: string; description: string; level: Level; category: string; status: 'draft' | 'published' };
type Series = { baseSlug: string; title: string; category: string; articleSlugs: string[]; parts: Part[] };

try {
  await mongoose.connect(uri, { autoCreate: false, autoIndex: false, serverSelectionTimeoutMS: 5000 });
  const categorySeed = JSON.parse(await readFile(path.join(root, 'seed/categories.json'), 'utf8')) as CategorySeed[];
  const series = JSON.parse(await readFile(path.join(root, 'seed/series-layout.json'), 'utf8')) as Series[];
  const [beforeCategories, beforePaths, articles] = await Promise.all([
    Category.find({}).lean<ICategory[]>(), LearningPath.find({}).lean<ILearningPath[]>(),
    Article.find({}).select('_id slug categoryId level tags learningPathId').lean<IArticle[]>(),
  ]);
  const bySlug = new Map(beforeCategories.map(c => [c.slug, { ...c }]));
  if (!bySlug.has('programming')) throw new Error('Base programming category is missing; seed base categories first.');
  const relevant = categorySeed.filter(c => languages.includes(c.slug) || languages.includes(c.parent ?? ''));
  const additions: string[] = [];
  for (const c of relevant) {
    if (bySlug.has(c.slug)) continue;
    additions.push(c.slug);
    bySlug.set(c.slug, { ...c, _id: new mongoose.Types.ObjectId(), articleCount: 0 } as unknown as ICategory);
  }
  for (const c of relevant) {
    const node = bySlug.get(c.slug)!; const parent = bySlug.get(c.parent!);
    if (!parent) throw new Error(`Missing category parent for ${c.slug}.`);
    node.parentId = parent._id;
  }
  const proposed = [...bySlug.values()];
  for (const c of proposed) validateCategoryTree(proposed, String(c._id), c.parentId ? String(c.parentId) : null);
  const sourceLanguages = new Map<string, string>();
  for (const filename of await readdir(path.join(root, 'seed/content'))) {
    if (!filename.endsWith('.md')) continue;
    const { data } = matter(await readFile(path.join(root, 'seed/content', filename), 'utf8'));
    const language = languages.find(lang => String(data.category).startsWith(lang + '-'));
    if (language) sourceLanguages.set(String(data.slug), language);
  }
  const oldCategorySlug = new Map(beforeCategories.map(c => [String(c._id), c.slug]));
  const moves = articles.flatMap(article => {
    const current = oldCategorySlug.get(String(article.categoryId)) ?? '';
    const known = sourceLanguages.get(article.slug);
    const language = known ?? (current === 'programming' || relevant.some(c => c.slug === current)
      ? languages.find(lang => article.tags?.includes(lang) || (lang === 'csharp' && article.tags?.includes('dotnet'))) : undefined);
    if (!language) return [];
    const target = bySlug.get(`${language}-${article.level}`);
    if (!target) throw new Error(`No level category for ${article.slug}.`);
    return String(target._id) === String(article.categoryId) ? [] : [{ id: article._id, slug: article.slug, categoryId: target._id, target: target.slug }];
  });
  const articleBySlug = new Map(articles.map(a => [a.slug, a]));
  const existingIds = new Set(articles.map(a => String(a._id)));
  const pathBySlug = new Map(beforePaths.map(p => [p.slug, p]));
  const plans: Array<Part & { id: mongoose.Types.ObjectId; categoryId: mongoose.Types.ObjectId; articleIds: mongoose.Types.ObjectId[] }> = [];
  for (const group of series) {
    const known = group.articleSlugs.map(slug => articleBySlug.get(slug)?._id).filter((id): id is mongoose.Types.ObjectId => Boolean(id));
    // Keep CMS-only articles already attached to this series; do not drop content not present in seed.
    const related = beforePaths.filter(p => p.slug === group.baseSlug || p.slug.startsWith(group.baseSlug + '-phan-'));
    const ids = [...new Map([...known, ...related.flatMap(p => p.articleIds).filter(id => existingIds.has(String(id)))].map(id => [String(id), id])).values()];
    if (!ids.length) continue;
    const count = Math.ceil(ids.length / 10);
    for (let i = 0; i < count; i++) {
      const template = group.parts[i] ?? { ...group.parts[0]!, slug: group.baseSlug + `-phan-${String(i + 1).padStart(2, '0')}`, title: `${group.title} · Phần ${i + 1}`, level: 'advanced' as const };
      const existing = pathBySlug.get(template.slug); const category = bySlug.get(template.category);
      if (!category) throw new Error(`Missing path category ${template.category}.`);
      plans.push({ ...template, status: existing?.status ?? pathBySlug.get(group.baseSlug)?.status ?? template.status,
        id: existing?._id ?? new mongoose.Types.ObjectId(), categoryId: category._id, articleIds: ids.slice(i * 10, (i + 1) * 10) });
    }
  }
  const membership = new Set<string>();
  for (const plan of plans) for (const id of plan.articleIds) {
    if (membership.has(String(id))) throw new Error('An article occurs in two planned parts.');
    membership.add(String(id));
  }
  console.log(JSON.stringify({ mode: apply ? 'apply' : 'dry-run', database: mongoose.connection.name,
    createCategories: additions, categoryMoves: moves.map(m => ({ slug: m.slug, to: m.target })),
    parts: plans.map(p => ({ slug: p.slug, count: p.articleIds.length })) }, null, 2));
  if (apply) {
    const backupDir = path.join(root, '.migration-backups'); await mkdir(backupDir, { recursive: true });
    const backup = path.join(backupDir, `programming-structure-${Date.now()}.json`);
    await writeFile(backup, JSON.stringify({ categories: beforeCategories, paths: beforePaths, articles }, null, 2));
    for (const seed of relevant) {
      const node = bySlug.get(seed.slug)!; const { parent: _parent, ...record } = seed;
      await Category.updateOne({ slug: seed.slug }, { $setOnInsert: { ...record, _id: node._id, articleCount: 0 }, $set: { parentId: node.parentId } }, { upsert: true });
    }
    if (moves.length) await Article.bulkWrite(moves.map(m => ({ updateOne: { filter: { _id: m.id }, update: { $set: { categoryId: m.categoryId } } } })));
    for (const plan of plans) {
      const { id, articleIds, categoryId, slug, title, description, level, status } = plan;
      await LearningPath.updateOne({ slug }, { $setOnInsert: { _id: id }, $set: { title, description, level, categoryId, articleIds, status } }, { upsert: true });
    }
    for (const plan of plans) {
      await Article.updateMany({ _id: { $in: plan.articleIds } }, { $set: { learningPathId: plan.id } });
      await LearningPath.updateMany({ _id: { $ne: plan.id }, articleIds: { $in: plan.articleIds } }, { $pull: { articleIds: { $in: plan.articleIds } } });
    }
    await recountCategories(); console.log(`Applied metadata only. Backup: ${backup}`);
  }
} finally { await mongoose.disconnect(); }
