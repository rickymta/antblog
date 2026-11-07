/** Reconcile category metadata and article assignments only. Dry-run unless --apply. */
import 'dotenv/config';
import mongoose from 'mongoose';
import { readFile, readdir, mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import matter from 'gray-matter';
import { Article, Category, type ICategory } from '../src/models/index.js';
import { validateCategoryTree } from '../src/services/categories.js';
import { recountCategories } from '../src/services/counters.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const apply = process.argv.includes('--apply');
if (!process.env.MONGODB_URI) throw new Error('MONGODB_URI is required.');
try {
  await mongoose.connect(process.env.MONGODB_URI, { autoCreate: false, autoIndex: false, serverSelectionTimeoutMS: 5000 });
  const seeds = JSON.parse(await readFile(path.join(root, 'seed/categories.json'), 'utf8')) as Array<Omit<ICategory, '_id'> & { parent?: string }>;
  const [before, articles] = await Promise.all([
    Category.find({}).lean<ICategory[]>(),
    Article.find({}).select('_id slug categoryId').lean(),
  ]);
  const bySlug = new Map(before.map(c => [c.slug, { ...c }]));
  for (const seed of seeds) {
    const old = bySlug.get(seed.slug);
    bySlug.set(seed.slug, { ...seed, _id: old?._id ?? new mongoose.Types.ObjectId(), articleCount: old?.articleCount ?? 0 });
  }
  for (const seed of seeds) bySlug.get(seed.slug)!.parentId = seed.parent ? bySlug.get(seed.parent)!._id : null;
  const proposed = [...bySlug.values()];
  for (const c of proposed) validateCategoryTree(proposed, String(c._id), c.parentId ? String(c.parentId) : null);
  const parents = new Set(proposed.filter(c => c.parentId).map(c => String(c.parentId)));
  const assignments = new Map<string, ICategory>();
  for (const file of await readdir(path.join(root, 'seed/content'))) {
    if (!file.endsWith('.md')) continue;
    const { data } = matter(await readFile(path.join(root, 'seed/content', file), 'utf8'));
    const category = bySlug.get(data.category);
    if (!category || parents.has(String(category._id))) throw new Error(`Invalid leaf category in ${file}`);
    if (assignments.has(data.slug)) throw new Error(`Duplicate article slug: ${data.slug}`);
    assignments.set(data.slug, category);
  }
  const moves = articles.flatMap(a => {
    const target = assignments.get(a.slug);
    return target && String(target._id) !== String(a.categoryId)
      ? [{ id: a._id, slug: a.slug, from: a.categoryId, to: target._id, category: target.slug }] : [];
  });
  const unresolved = articles.filter(a => !assignments.has(a.slug) && parents.has(String(a.categoryId))).map(a => a.slug);
  console.log(JSON.stringify({ mode: apply ? 'apply' : 'dry-run', categories: seeds.map(c => c.slug), moves, unresolved }, null, 2));
  if (apply) {
    if (unresolved.length) throw new Error('CMS-only articles on parent categories require explicit leaf assignments before applying.');
    const dir = path.join(root, '.migration-backups');
    await mkdir(dir, { recursive: true });
    const backup = path.join(dir, `category-content-${Date.now()}.json`);
    await writeFile(backup, JSON.stringify({ categories: before, articles }, null, 2));
    for (const seed of seeds) {
      const { parent: _, ...fields } = seed;
      const node = bySlug.get(seed.slug)!;
      await Category.updateOne({ slug: seed.slug }, {
        $set: { ...fields, parentId: node.parentId },
        $setOnInsert: { _id: node._id },
      }, { upsert: true });
    }
    if (moves.length) await Article.bulkWrite(moves.map(m => ({ updateOne: {
      filter: { _id: m.id, categoryId: m.from }, update: { $set: { categoryId: m.to } },
    } })));
    await recountCategories();
    console.log(`Applied category metadata only. Backup: ${backup}`);
  }
} finally { await mongoose.disconnect(); }
